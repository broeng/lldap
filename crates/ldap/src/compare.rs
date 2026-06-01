use crate::core::error::{LdapError, LdapResult};
use ldap3_proto::proto::{LdapCompareRequest, LdapOp, LdapResult as LdapResultOp, LdapResultCode};
use lldap_domain::types::AttributeName;

pub fn compare(
    request: LdapCompareRequest,
    search_results: Vec<LdapOp>,
    base_dn_str: &str,
) -> LdapResult<Vec<LdapOp>> {
    if search_results.len() > 2 {
        // SearchResultEntry + SearchResultDone
        return Err(LdapError {
            code: LdapResultCode::OperationsError,
            message: "Too many search results".to_string(),
        });
    }
    let requested_attribute = AttributeName::from(&request.atype);
    match search_results.first() {
        Some(LdapOp::SearchResultEntry(entry)) => {
            let available = entry.attributes.iter().any(|attr| {
                AttributeName::from(&attr.atype) == requested_attribute
                    && attr.vals.contains(&request.val)
            });
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: if available {
                    LdapResultCode::CompareTrue
                } else {
                    LdapResultCode::CompareFalse
                },
                matcheddn: request.dn,
                message: "".to_string(),
                referral: vec![],
            })])
        }
        Some(LdapOp::SearchResultDone(_)) => Ok(vec![LdapOp::CompareResult(LdapResultOp {
            code: LdapResultCode::NoSuchObject,
            matcheddn: base_dn_str.to_string(),
            message: "".to_string(),
            referral: vec![],
        })]),
        None => Err(LdapError {
            code: LdapResultCode::OperationsError,
            message: "Search request returned nothing".to_string(),
        }),
        _ => Err(LdapError {
            code: LdapResultCode::OperationsError,
            message: "Unexpected results from search".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{events::NoopLdapEventHandler, handler::tests::setup_bound_admin_handler};
    use chrono::TimeZone;
    use lldap_domain::{
        types::{Group, GroupId, User, UserAndGroups, UserId},
        uuid,
    };
    use lldap_domain_handlers::handler::{GroupRequestFilter, RequestContext, UserRequestFilter};
    use lldap_test_utils::MockTestBackendHandler;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_compare_user() {
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let context = RequestContext::empty();
        mock.expect_list_users().returning(|_, g| {
            assert_eq!(
                g.filter,
                Some(UserRequestFilter::UserId(UserId::new("bob")))
            );
            assert!(g.need_groups);
            Ok(vec![UserAndGroups {
                user: User {
                    user_id: UserId::new("bob"),
                    email: "bob@bobmail.bob".into(),
                    ..Default::default()
                },
                groups: None,
            }])
        });
        mock.expect_list_groups().returning(|_, _| Ok(vec![]));
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let dn = "uid=bob,ou=people,dc=example,dc=com";
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "uid".to_owned(),
            val: b"bob".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::CompareTrue,
                matcheddn: dn.to_string(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
        // Non-canonical attribute.
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "eMail".to_owned(),
            val: b"bob@bobmail.bob".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::CompareTrue,
                matcheddn: dn.to_string(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
    }

    #[tokio::test]
    async fn test_compare_group() {
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let context = RequestContext::empty();
        mock.expect_list_users().returning(|_, _| Ok(vec![]));
        mock.expect_list_groups().returning(|_, f| {
            assert_eq!(
                f.filter,
                Some(GroupRequestFilter::DisplayName("group".into()))
            );
            Ok(vec![Group {
                id: GroupId(1),
                display_name: "group".into(),
                creation_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
                users: vec![UserId::new("bob")],
                uuid: uuid!("04ac75e0-2900-3e21-926c-2f732c26b3fc"),
                attributes: Vec::new(),
                modified_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
            }])
        });
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let dn = "uid=group,ou=groups,dc=example,dc=com";
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "uid".to_owned(),
            val: b"group".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::CompareTrue,
                matcheddn: dn.to_string(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
    }

    #[tokio::test]
    async fn test_compare_not_found() {
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let context = RequestContext::empty();
        mock.expect_list_users().returning(|_, g| {
            assert_eq!(
                g.filter,
                Some(UserRequestFilter::UserId(UserId::new("bob")))
            );
            assert!(g.need_groups);
            Ok(vec![])
        });
        mock.expect_list_groups().returning(|_, _| Ok(vec![]));
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let dn = "uid=bob,ou=people,dc=example,dc=com";
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "uid".to_owned(),
            val: b"bob".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::NoSuchObject,
                matcheddn: "dc=example,dc=com".to_owned(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
    }

    #[tokio::test]
    async fn test_compare_no_match() {
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let context = RequestContext::empty();
        mock.expect_list_users().returning(|_, g| {
            assert_eq!(
                g.filter,
                Some(UserRequestFilter::UserId(UserId::new("bob")))
            );
            assert!(g.need_groups);
            Ok(vec![UserAndGroups {
                user: User {
                    user_id: UserId::new("bob"),
                    email: "bob@bobmail.bob".into(),
                    ..Default::default()
                },
                groups: None,
            }])
        });
        mock.expect_list_groups().returning(|_, _| Ok(vec![]));
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let dn = "uid=bob,ou=people,dc=example,dc=com";
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "mail".to_owned(),
            val: b"bob@bob".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::CompareFalse,
                matcheddn: dn.to_string(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
    }

    #[tokio::test]
    async fn test_compare_group_member() {
        let mut mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let context = RequestContext::empty();
        mock.expect_list_users().returning(|_, _| Ok(vec![]));
        mock.expect_list_groups().returning(|_, f| {
            assert_eq!(
                f.filter,
                Some(GroupRequestFilter::DisplayName("group".into()))
            );
            Ok(vec![Group {
                id: GroupId(1),
                display_name: "group".into(),
                creation_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
                users: vec![UserId::new("bob")],
                uuid: uuid!("04ac75e0-2900-3e21-926c-2f732c26b3fc"),
                attributes: Vec::new(),
                modified_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
            }])
        });
        let ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let dn = "uid=group,ou=groups,dc=example,dc=com";
        let request = LdapCompareRequest {
            dn: dn.to_string(),
            atype: "uniqueMember".to_owned(),
            val: b"uid=bob,ou=people,dc=example,dc=com".to_vec(),
        };
        assert_eq!(
            ldap_handler.do_compare(&context, request).await,
            Ok(vec![LdapOp::CompareResult(LdapResultOp {
                code: LdapResultCode::CompareTrue,
                matcheddn: dn.to_owned(),
                message: "".to_string(),
                referral: vec![],
            })])
        );
    }
}
