use crate::{
    LdapEventHandler,
    core::{
        error::{LdapError, LdapResult},
        utils::{LdapInfo, get_user_id_from_distinguished_name},
    },
    handler::make_modify_response,
    password::{self},
};
use ldap3_proto::proto::{LdapModify, LdapModifyRequest, LdapModifyType, LdapOp, LdapResultCode};
use lldap_access_control::UserReadableBackendHandler;
use lldap_auth::access_control::ValidationResults;
use lldap_domain::types::UserId;
use lldap_domain_handlers::handler::RequestContext;
use lldap_opaque_handler::OpaqueHandler;

async fn handle_modify_change(
    opaque_handler: &impl OpaqueHandler,
    event_handler: &impl LdapEventHandler,
    user_id: UserId,
    credentials: &ValidationResults,
    user_is_admin: bool,
    change: &LdapModify,
    context: &RequestContext,
) -> LdapResult<()> {
    if !change
        .modification
        .atype
        .eq_ignore_ascii_case("userpassword")
        || change.operation != LdapModifyType::Replace
    {
        return Err(LdapError {
            code: LdapResultCode::UnwillingToPerform,
            message: format!(
                r#"Unsupported operation: `{:?}` for `{}`"#,
                change.operation, change.modification.atype
            ),
        });
    }
    if !credentials.can_change_password(&user_id, user_is_admin) {
        return Err(LdapError {
            code: LdapResultCode::InsufficentAccessRights,
            message: format!(
                r#"User `{}` cannot modify the password of user `{}`"#,
                &credentials.user, &user_id
            ),
        });
    }
    if let [value] = &change.modification.vals.as_slice() {
        password::change_password(opaque_handler, event_handler, user_id, value, context)
            .await
            .map_err(|e| LdapError {
                code: LdapResultCode::Other,
                message: format!("Error while changing the password: {:#?}", e),
            })?;
    } else {
        return Err(LdapError {
            code: LdapResultCode::InvalidAttributeSyntax,
            message: format!(
                r#"Wrong number of values for password attribute: {}"#,
                change.modification.vals.len()
            ),
        });
    }
    Ok(())
}

pub(crate) async fn handle_modify_request<'cred, UserBackendHandler>(
    opaque_handler: &impl OpaqueHandler,
    event_handler: &impl LdapEventHandler,
    get_readable_handler: impl FnOnce(
        &'cred ValidationResults,
        UserId,
    ) -> Option<&'cred UserBackendHandler>,
    ldap_info: &LdapInfo,
    credentials: &'cred ValidationResults,
    request: &LdapModifyRequest,
    context: &RequestContext,
) -> LdapResult<Vec<LdapOp>>
where
    // Note: ideally, get_readable_handler would take UserId by reference, but I couldn't make the lifetimes work.
    UserBackendHandler: UserReadableBackendHandler + 'cred,
{
    match get_user_id_from_distinguished_name(
        &request.dn,
        &ldap_info.base_dn,
        &ldap_info.base_dn_str,
    ) {
        Ok(uid) => {
            let user_is_admin = get_readable_handler(credentials, uid.clone())
                .ok_or_else(|| LdapError {
                    code: LdapResultCode::InsufficentAccessRights,
                    message: format!(
                        "User `{}` cannot modify user `{}`",
                        credentials.user.as_str(),
                        uid.as_str()
                    ),
                })?
                .get_user_groups(context, uid.clone())
                .await
                .map_err(|e| LdapError {
                    code: LdapResultCode::OperationsError,
                    message: format!("Internal error while requesting user's groups: {:#?}", e),
                })?
                .iter()
                .any(|g| g.display_name == "lldap_admin".into());
            for change in &request.changes {
                handle_modify_change(
                    opaque_handler,
                    event_handler,
                    uid.clone(),
                    credentials,
                    user_is_admin,
                    change,
                    context,
                )
                .await?
            }
            Ok(vec![make_modify_response(
                LdapResultCode::Success,
                String::new(),
            )])
        }
        Err(e) => Err(LdapError {
            code: LdapResultCode::InvalidDNSyntax,
            message: format!("Invalid username: {}", e),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        events::NoopLdapEventHandler,
        handler::tests::{
            setup_bound_admin_handler, setup_bound_handler_with_group,
            setup_bound_password_manager_handler,
        },
        password::tests::expect_password_change,
    };
    use chrono::TimeZone;
    use ldap3_proto::proto::LdapResult as LdapResultOp;
    use lldap_domain::{
        types::{GroupDetails, GroupId, GroupName, UserId},
        uuid,
    };
    use lldap_test_utils::MockTestBackendHandler;
    use mockall::predicate::eq;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    fn setup_target_user_groups(
        mock: &mut MockTestBackendHandler,
        target_user: &str,
        groups: Vec<&'static str>,
        context: &RequestContext,
    ) {
        mock.expect_get_user_groups()
            .times(1)
            .with(eq(context.clone()), eq(UserId::from(target_user)))
            .return_once(move |_, _| {
                let mut g = HashSet::<GroupDetails>::new();
                for group in groups {
                    g.insert(GroupDetails {
                        group_id: GroupId(42),
                        display_name: GroupName::from(group),
                        creation_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
                        uuid: uuid!("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"),
                        attributes: Vec::new(),
                    });
                }
                Ok(g)
            });
    }

    fn make_password_modify_request(target_user: &str) -> LdapModifyRequest {
        LdapModifyRequest {
            dn: format!("uid={},ou=people,dc=example,dc=com", target_user),
            changes: vec![LdapModify {
                operation: LdapModifyType::Replace,
                modification: ldap3_proto::LdapPartialAttribute {
                    atype: "userPassword".to_string(),
                    vals: vec![b"tommy".to_vec()],
                },
            }],
        }
    }

    fn make_modify_success_response() -> Vec<LdapOp> {
        vec![LdapOp::ModifyResponse(LdapResultOp {
            code: LdapResultCode::Success,
            matcheddn: "".to_string(),
            message: "".to_string(),
            referral: vec![],
        })]
    }

    fn make_modify_failure_response(code: LdapResultCode, message: &str) -> Vec<LdapOp> {
        vec![LdapOp::ModifyResponse(LdapResultOp {
            code,
            matcheddn: "".to_string(),
            message: message.to_string(),
            referral: vec![],
        })]
    }

    #[tokio::test]
    async fn test_modify_password_of_regular_as_admin() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "bob", Vec::new(), &context);
        expect_password_change(&mut mock, "bob");
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let request = make_password_modify_request("bob");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_success_response()
        );
    }

    #[tokio::test]
    async fn test_modify_password_of_regular_as_regular() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "test", Vec::new(), &context);
        expect_password_change(&mut mock, "test");
        let ldap_handler =
            setup_bound_handler_with_group(mock, event_mock, "regular", &context).await;
        let request = make_password_modify_request("test");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_success_response()
        );
    }

    #[tokio::test]
    async fn test_modify_password_of_regular_as_password_manager() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "bob", Vec::new(), &context);
        expect_password_change(&mut mock, "bob");
        let ldap_handler = setup_bound_password_manager_handler(mock, event_mock, &context).await;
        let request = make_password_modify_request("bob");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_success_response()
        );
    }

    #[tokio::test]
    async fn test_modify_password_of_admin_as_password_manager() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "bob", vec!["lldap_admin"], &context);
        let ldap_handler = setup_bound_password_manager_handler(mock, event_mock, &context).await;
        let request = make_password_modify_request("bob");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_failure_response(
                LdapResultCode::InsufficentAccessRights,
                "User `test` cannot modify the password of user `bob`"
            )
        );
    }

    #[tokio::test]
    async fn test_modify_password_of_other_regular_as_regular() {
        let context = RequestContext::empty();
        let event_mock = NoopLdapEventHandler::new();
        let ldap_handler = setup_bound_handler_with_group(
            MockTestBackendHandler::new(),
            event_mock,
            "regular",
            &context,
        )
        .await;
        let request = make_password_modify_request("bob");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_failure_response(
                LdapResultCode::InsufficentAccessRights,
                "User `test` cannot modify user `bob`"
            )
        );
    }

    #[tokio::test]
    async fn test_modify_password_of_admin_as_admin() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "test", vec!["lldap_admin"], &context);
        expect_password_change(&mut mock, "test");
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let request = make_password_modify_request("test");
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_success_response()
        );
    }

    #[tokio::test]
    async fn test_modify_password_invalid_number_of_values() {
        let context = RequestContext::empty();
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        setup_target_user_groups(&mut mock, "bob", Vec::new(), &context);
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let request = {
            let target_user = "bob";
            LdapModifyRequest {
                dn: format!("uid={},ou=people,dc=example,dc=com", target_user),
                changes: vec![LdapModify {
                    operation: LdapModifyType::Replace,
                    modification: ldap3_proto::LdapPartialAttribute {
                        atype: "userPassword".to_string(),
                        vals: vec![b"tommy".to_vec(), b"other_value".to_vec()],
                    },
                }],
            }
        };
        assert_eq!(
            ldap_handler.do_modify_request(&context, &request).await,
            make_modify_failure_response(
                LdapResultCode::InvalidAttributeSyntax,
                "Wrong number of values for password attribute: 2"
            )
        );
    }
}
