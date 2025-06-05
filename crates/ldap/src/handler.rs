use crate::{
    LdapEventHandler, compare,
    core::{
        error::{LdapError, LdapResult},
        utils::{LdapInfo, parse_distinguished_name},
    },
    create, delete, modify,
    password::{self, do_password_modification},
    search::{
        self, is_root_dse_request, is_subschema_entry_request, make_ldap_subschema_entry,
        make_search_error, make_search_request, make_search_success, root_dse_response,
    },
};
use ldap3_proto::proto::{
    LdapAddRequest, LdapBindRequest, LdapBindResponse, LdapCompareRequest, LdapExtendedRequest,
    LdapExtendedResponse, LdapFilter, LdapModifyRequest, LdapOp, LdapPasswordModifyRequest,
    LdapResult as LdapResultOp, LdapResultCode, LdapSearchRequest, OID_PASSWORD_MODIFY, OID_WHOAMI,
};
use lldap_access_control::AccessControlledBackendHandler;
use lldap_auth::access_control::ValidationResults;
use lldap_domain::{public_schema::PublicSchema, types::AttributeName};
use lldap_domain_handlers::handler::{BackendHandler, LoginHandler, ReadSchemaBackendHandler, RequestContext};
use lldap_opaque_handler::OpaqueHandler;
use lldap_plugin_engine::api::arguments::ldap_bind_result::BindResult;
use tracing::{debug, instrument};

use super::delete::make_del_response;

pub(crate) fn make_add_response(code: LdapResultCode, message: String) -> LdapOp {
    LdapOp::AddResponse(LdapResultOp {
        code,
        matcheddn: "".to_string(),
        message,
        referral: vec![],
    })
}

pub(crate) fn make_extended_response(code: LdapResultCode, message: String) -> LdapOp {
    LdapOp::ExtendedResponse(LdapExtendedResponse {
        res: LdapResultOp {
            code,
            matcheddn: "".to_string(),
            message,
            referral: vec![],
        },
        name: None,
        value: None,
    })
}

pub(crate) fn make_modify_response(code: LdapResultCode, message: String) -> LdapOp {
    LdapOp::ModifyResponse(LdapResultOp {
        code,
        matcheddn: "".to_string(),
        message,
        referral: vec![],
    })
}

pub struct LdapHandler<Backend, Events> {
    user_info: Option<ValidationResults>,
    backend_handler: AccessControlledBackendHandler<Backend>,
    event_handler: Events,
    ldap_info: LdapInfo,
    session_uuid: uuid::Uuid,
}

impl<Backend, Events> LdapHandler<Backend, Events> {
    pub fn session_uuid(&self) -> &uuid::Uuid {
        &self.session_uuid
    }
}

impl<Backend: LoginHandler, Events: LdapEventHandler> LdapHandler<Backend, Events> {
    pub fn get_login_handler(&self) -> &(impl LoginHandler + use<Backend, Events>) {
        self.backend_handler.unsafe_get_handler()
    }
}

impl<Backend: OpaqueHandler, Events: LdapEventHandler> LdapHandler<Backend, Events> {
    pub fn get_opaque_handler(&self) -> &(impl OpaqueHandler + use<Backend, Events>) {
        self.backend_handler.unsafe_get_handler()
    }
}

enum Credentials<'s> {
    Bound(&'s ValidationResults),
    Unbound(Vec<LdapOp>),
}

impl<Backend: BackendHandler + LoginHandler + OpaqueHandler, Events: LdapEventHandler>
    LdapHandler<Backend, Events>
{
    pub fn new(
        backend_handler: AccessControlledBackendHandler<Backend>,
        event_handler: Events,
        mut ldap_base_dn: String,
        ignored_user_attributes: Vec<AttributeName>,
        ignored_group_attributes: Vec<AttributeName>,
        session_uuid: uuid::Uuid,
    ) -> Self {
        ldap_base_dn.make_ascii_lowercase();
        Self {
            user_info: None,
            backend_handler,
            event_handler,
            ldap_info: LdapInfo {
                base_dn: parse_distinguished_name(&ldap_base_dn).unwrap_or_else(|_| {
                    panic!(
                        "Invalid value for ldap_base_dn in configuration: {}",
                        ldap_base_dn
                    )
                }),
                base_dn_str: ldap_base_dn,
                ignored_user_attributes,
                ignored_group_attributes,
            },
            session_uuid,
        }
    }

    #[cfg(test)]
    pub fn new_for_tests(
        backend_handler: Backend,
        event_handler: Events,
        ldap_base_dn: &str,
    ) -> Self {
        Self::new(
            AccessControlledBackendHandler::new(backend_handler),
            event_handler,
            ldap_base_dn.to_string(),
            vec![],
            vec![],
            uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        )
    }

    fn get_credentials(&self) -> Credentials<'_> {
        match self.user_info.as_ref() {
            Some(user_info) => Credentials::Bound(user_info),
            None => Credentials::Unbound(vec![make_extended_response(
                LdapResultCode::InsufficentAccessRights,
                "No user currently bound".to_string(),
            )]),
        }
    }

    pub async fn do_search_or_dse(
        &self,
        context: &RequestContext,
        request: &LdapSearchRequest,
    ) -> LdapResult<Vec<LdapOp>> {
        if is_root_dse_request(request) {
            debug!("rootDSE request");
            return Ok(vec![
                LdapOp::SearchResultEntry(
                    self.event_handler
                        .on_ldap_root_dse(context, root_dse_response(&self.ldap_info.base_dn_str))
                        .await,
                ),
                make_search_success(),
            ]);
        } else if is_subschema_entry_request(request) {
            // See RFC4512 section 4.4 "Subschema discovery"
            debug!("Schema request");
            let backend_handler = self
                .user_info
                .as_ref()
                .and_then(|u| self.backend_handler.get_schema_only_handler(u))
                .ok_or_else(|| LdapError {
                    code: LdapResultCode::InsufficentAccessRights,
                    message: "No user currently bound".to_string(),
                })?;

            let schema = backend_handler.get_schema(context).await.map_err(|e| LdapError {
                code: LdapResultCode::OperationsError,
                message: format!("Unable to get schema: {:#}", e),
            })?;
            return Ok(vec![
                make_ldap_subschema_entry(PublicSchema::from(schema)),
                make_search_success(),
            ]);
        }
        self.do_search(context, request).await
    }

    #[instrument(skip_all, level = "debug")]
    async fn do_search(
        &self,
        context: &RequestContext,
        request: &LdapSearchRequest,
    ) -> LdapResult<Vec<LdapOp>> {
        let user_info = self.user_info.as_ref().ok_or_else(|| LdapError {
            code: LdapResultCode::InsufficentAccessRights,
            message: "No user currently bound".to_string(),
        })?;
        let backend_handler = self
            .backend_handler
            .get_user_restricted_lister_handler(user_info);
        search::do_search(
            &backend_handler,
            &self.event_handler,
            &self.ldap_info,
            request,
            context,
        )
        .await
    }

    #[instrument(skip_all, level = "debug", fields(dn = %request.dn))]
    pub async fn do_bind(
        &mut self,
        context: &RequestContext,
        request: &LdapBindRequest,
    ) -> Vec<LdapOp> {
        let (code, message) =
            match password::do_bind(&self.ldap_info, request, self.get_login_handler(), context)
                .await
            {
                Ok(user_id) => {
                    self.user_info = self
                        .backend_handler
                        .get_permissions_for_user(context, user_id)
                        .await
                        .ok();
                    debug!("Success!");
                    (LdapResultCode::Success, "".to_string())
                }
                Err(err) => (err.code, err.message),
            };
        let bind_result = self
            .event_handler
            .on_ldap_bind(
                &context,
                &request,
                BindResult {
                    result_code: code,
                    message,
                },
            )
            .await;
        vec![LdapOp::BindResponse(LdapBindResponse {
            res: LdapResultOp {
                code: bind_result.result_code,
                matcheddn: "".to_string(),
                message: bind_result.message,
                referral: vec![],
            },
            saslcreds: None,
        })]
    }

    #[instrument(skip_all, level = "debug")]
    async fn do_extended_request(
        &self,
        context: &RequestContext,
        request: &LdapExtendedRequest,
    ) -> Vec<LdapOp> {
        let result = match request.name.as_str() {
            OID_PASSWORD_MODIFY => match LdapPasswordModifyRequest::try_from(request) {
                Ok(password_request) => {
                    let credentials = match self.get_credentials() {
                        Credentials::Bound(cred) => cred,
                        Credentials::Unbound(err) => return err,
                    };
                    do_password_modification(
                        credentials,
                        &self.ldap_info,
                        &self.backend_handler,
                        &self.event_handler,
                        self.get_opaque_handler(),
                        &password_request,
                        context,
                    )
                    .await
                    .unwrap_or_else(|e: LdapError| vec![make_extended_response(e.code, e.message)])
                }
                Err(e) => vec![make_extended_response(
                    LdapResultCode::ProtocolError,
                    format!("Error while parsing password modify request: {:#?}", e),
                )],
            },
            OID_WHOAMI => {
                let authz_id = self
                    .user_info
                    .as_ref()
                    .map(|user_info| {
                        format!(
                            "dn:uid={},ou=people,{}",
                            user_info.user.as_str(),
                            self.ldap_info.base_dn_str
                        )
                    })
                    .unwrap_or_default();
                vec![make_extended_response(LdapResultCode::Success, authz_id)]
            }
            _ => vec![make_extended_response(
                LdapResultCode::UnwillingToPerform,
                format!("Unsupported extended operation: {}", &request.name),
            )],
        };
        self.event_handler
            .on_ldap_extended_request(context, request.clone(), result)
            .await
    }

    #[instrument(skip_all, level = "debug", fields(dn = %request.dn))]
    pub async fn do_modify_request(
        &self,
        context: &RequestContext,
        request: &LdapModifyRequest,
    ) -> Vec<LdapOp> {
        let credentials = match self.get_credentials() {
            Credentials::Bound(cred) => cred,
            Credentials::Unbound(err) => return err,
        };
        let result = modify::handle_modify_request(
            self.get_opaque_handler(),
            |credentials, user_id| {
                self.backend_handler
                    .get_readable_handler(credentials, &user_id)
            },
            &self.ldap_info,
            credentials,
            request,
            context,
        )
        .await
        .unwrap_or_else(|e: LdapError| vec![make_modify_response(e.code, e.message)]);
        self.event_handler
            .on_ldap_modify(context, request.clone(), result)
            .await
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn create_user_or_group(
        &self,
        context: &RequestContext,
        request: LdapAddRequest,
    ) -> LdapResult<Vec<LdapOp>> {
        let backend_handler = self
            .user_info
            .as_ref()
            .and_then(|u| self.backend_handler.get_admin_handler(u))
            .ok_or_else(|| LdapError {
                code: LdapResultCode::InsufficentAccessRights,
                message: "Unauthorized write".to_string(),
            })?;
        create::create_user_or_group(backend_handler, &self.ldap_info, request, context).await
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn delete_user_or_group(
        &self,
        context: &RequestContext,
        request: String,
    ) -> LdapResult<Vec<LdapOp>> {
        let backend_handler = self
            .user_info
            .as_ref()
            .and_then(|u| self.backend_handler.get_admin_handler(u))
            .ok_or_else(|| LdapError {
                code: LdapResultCode::InsufficentAccessRights,
                message: "Unauthorized write".to_string(),
            })?;
        delete::delete_user_or_group(backend_handler, &self.ldap_info, request, context).await
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn do_compare(
        &self,
        context: &RequestContext,
        request: LdapCompareRequest,
    ) -> LdapResult<Vec<LdapOp>> {
        let req = make_search_request::<String>(
            &self.ldap_info.base_dn_str,
            LdapFilter::Equality("dn".to_string(), request.dn.to_string()),
            vec![request.atype.clone()],
        );
        compare::compare(
            request,
            self.do_search(context, &req).await?,
            &self.ldap_info.base_dn_str,
        )
    }

    pub async fn handle_ldap_message(&mut self, ldap_op: LdapOp) -> Option<Vec<LdapOp>> {
        let context = RequestContext::new(self.user_info.clone());
        Some(match ldap_op {
            LdapOp::BindRequest(request) => self.do_bind(&context, &request).await,
            LdapOp::SearchRequest(request) => self
                .do_search_or_dse(&context, &request)
                .await
                .unwrap_or_else(|e: LdapError| vec![make_search_error(e.code, e.message)]),
            LdapOp::UnbindRequest => {
                let user_id = self.user_info.clone().map(|u| u.user);
                self.event_handler
                    .on_ldap_unbind(&context, user_id.clone())
                    .await;
                debug!(
                    "Unbind request for {}",
                    user_id
                        .as_ref()
                        .map(|u| u.as_str())
                        .unwrap_or("<not bound>"),
                );
                self.user_info = None;
                // No need to notify on unbind (per rfc4511)
                return None;
            }
            LdapOp::ModifyRequest(request) => self.do_modify_request(&context, &request).await,
            LdapOp::ExtendedRequest(request) => self.do_extended_request(&context, &request).await,
            LdapOp::AddRequest(request) => self
                .create_user_or_group(&context, request)
                .await
                .unwrap_or_else(|e: LdapError| vec![make_add_response(e.code, e.message)]),
            LdapOp::DelRequest(request) => self
                .delete_user_or_group(&context, request)
                .await
                .unwrap_or_else(|e: LdapError| vec![make_del_response(e.code, e.message)]),
            LdapOp::CompareRequest(request) => self
                .do_compare(&context, request)
                .await
                .unwrap_or_else(|e: LdapError| vec![make_search_error(e.code, e.message)]),
            op => vec![make_extended_response(
                LdapResultCode::UnwillingToPerform,
                format!("Unsupported operation: {:#?}", op),
            )],
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{events::NoopLdapEventHandler, password::tests::make_bind_success};
    use chrono::TimeZone;
    use ldap3_proto::proto::{LdapBindCred, LdapWhoamiRequest};
    use lldap_domain::{
        types::{GroupDetails, GroupId, UserId},
        uuid,
    };
    use lldap_domain_handlers::handler::*;
    use lldap_test_utils::{MockTestBackendHandler, setup_default_schema};
    use mockall::predicate::eq;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use tokio;

    pub fn make_user_search_request<S: Into<String>>(
        filter: LdapFilter,
        attrs: Vec<S>,
    ) -> LdapSearchRequest {
        make_search_request::<S>("ou=people,Dc=example,dc=com", filter, attrs)
    }

    pub fn make_group_search_request<S: Into<String>>(
        filter: LdapFilter,
        attrs: Vec<S>,
    ) -> LdapSearchRequest {
        make_search_request::<S>("ou=groups,dc=example,dc=com", filter, attrs)
    }

    pub async fn setup_bound_handler_with_group<E: LdapEventHandler>(
        mut mock: MockTestBackendHandler,
        event_handler: E,
        group: &str,
        context: &RequestContext,
    ) -> LdapHandler<MockTestBackendHandler, E> {
        mock.expect_bind()
            .with(
                eq(context.clone()),
                eq(BindRequest {
                    name: UserId::new("test"),
                    password: "pass".to_string(),
                }),
            )
            .return_once(|_, _| Ok(()));
        let group = group.to_string();
        mock.expect_get_user_groups()
            .with(eq(context.clone()), eq(UserId::new("test")))
            .return_once(|_, _| {
                let mut set = HashSet::new();
                set.insert(GroupDetails {
                    group_id: GroupId(42),
                    display_name: group.into(),
                    creation_date: chrono::Utc.timestamp_opt(42, 42).unwrap().naive_utc(),
                    uuid: uuid!("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8"),
                    attributes: Vec::new(),
                });
                Ok(set)
            });
        setup_default_schema(&mut mock, context);
        let mut ldap_handler = LdapHandler::new_for_tests(mock, event_handler, "dc=Example,dc=com");
        let request = LdapBindRequest {
            dn: "uid=test,ou=people,dc=example,dc=coM".to_string(),
            cred: LdapBindCred::Simple("pass".to_string()),
        };
        assert_eq!(
            ldap_handler.do_bind(context, &request).await,
            make_bind_success()
        );
        ldap_handler
    }

    pub async fn setup_bound_readonly_handler<E: LdapEventHandler>(
        mock: MockTestBackendHandler,
        event_handler: E,
        context: &RequestContext,
    ) -> LdapHandler<MockTestBackendHandler, E> {
        setup_bound_handler_with_group(mock, event_handler, "lldap_strict_readonly", context).await
    }

    pub async fn setup_bound_password_manager_handler<E: LdapEventHandler>(
        mock: MockTestBackendHandler,
        event_handler: E,
        context: &RequestContext,
    ) -> LdapHandler<MockTestBackendHandler, E> {
        setup_bound_handler_with_group(mock, event_handler, "lldap_password_manager", context).await
    }

    pub async fn setup_bound_admin_handler<E: LdapEventHandler>(
        mock: MockTestBackendHandler,
        event_handler: E,
        context: &RequestContext,
    ) -> LdapHandler<MockTestBackendHandler, E> {
        setup_bound_handler_with_group(mock, event_handler, "lldap_admin", context).await
    }

    #[tokio::test]
    async fn test_whoami_empty() {
        let mut ldap_handler = LdapHandler::new_for_tests(
            MockTestBackendHandler::new(),
            NoopLdapEventHandler::new(),
            "dc=example,dc=com",
        );
        let request = LdapOp::ExtendedRequest(LdapWhoamiRequest {}.into());
        assert_eq!(
            ldap_handler.handle_ldap_message(request).await,
            Some(vec![make_extended_response(
                LdapResultCode::Success,
                "".to_string(),
            )])
        );
    }

    #[tokio::test]
    async fn test_whoami_bound() {
        let context = RequestContext::empty();
        let mock = MockTestBackendHandler::new();
        let event_mock = NoopLdapEventHandler::new();
        let mut ldap_handler = setup_bound_admin_handler(mock, event_mock, &context).await;
        let request = LdapOp::ExtendedRequest(LdapWhoamiRequest {}.into());
        assert_eq!(
            ldap_handler.handle_ldap_message(request).await,
            Some(vec![make_extended_response(
                LdapResultCode::Success,
                "dn:uid=test,ou=people,dc=example,dc=com".to_string(),
            )])
        );
    }
}
