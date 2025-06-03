use async_trait::async_trait;
use ldap3_proto::{
    LdapSearchResultEntry,
    proto::{LdapBindRequest, LdapExtendedRequest, LdapModifyRequest, LdapOp, LdapSearchRequest},
};
use lldap_auth::types::UserId;
use lldap_domain_handlers::handler::RequestContext;
use lldap_plugin_engine::api::arguments::ldap_bind_result::BindResult;

use crate::search::InternalSearchResults;

#[async_trait]
pub trait LdapEventHandler: Send + Sync {
    async fn on_ldap_bind(
        &self,
        context: &RequestContext,
        request: &LdapBindRequest,
        bind_result: BindResult,
    ) -> BindResult;
    async fn on_ldap_unbind(&self, context: &RequestContext, user_id: Option<UserId>) -> ();
    async fn on_ldap_modify(
        &self,
        context: &RequestContext,
        modify_request: LdapModifyRequest,
        modify_result: Vec<LdapOp>,
    ) -> Vec<LdapOp>;
    async fn on_ldap_extended_request(
        &self,
        context: &RequestContext,
        request: LdapExtendedRequest,
        result: Vec<LdapOp>,
    ) -> Vec<LdapOp>;
    async fn on_password_update(
        &self,
        context: &RequestContext,
        user_id: &UserId,
        password: &String,
    ) -> ();
    async fn on_ldap_search_result(
        &self,
        context: &RequestContext,
        request: &LdapSearchRequest,
        search_result: InternalSearchResults,
    ) -> InternalSearchResults;
    async fn on_ldap_root_dse(
        &self,
        context: &RequestContext,
        search_result_entry: LdapSearchResultEntry,
    ) -> LdapSearchResultEntry;
}

#[cfg(test)]
#[derive(Clone)]
pub struct NoopLdapEventHandler;

#[cfg(test)]
impl NoopLdapEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
#[async_trait]
impl LdapEventHandler for NoopLdapEventHandler {
    async fn on_ldap_bind(
        &self,
        _context: &RequestContext,
        _request: &LdapBindRequest,
        bind_result: BindResult,
    ) -> BindResult {
        bind_result
    }
    async fn on_ldap_unbind(&self, _context: &RequestContext, _user_id: Option<UserId>) -> () {
        ()
    }
    async fn on_ldap_modify(
        &self,
        _context: &RequestContext,
        _modify_request: LdapModifyRequest,
        modify_result: Vec<LdapOp>,
    ) -> Vec<LdapOp> {
        modify_result
    }
    async fn on_ldap_extended_request(
        &self,
        _context: &RequestContext,
        _request: LdapExtendedRequest,
        result: Vec<LdapOp>,
    ) -> Vec<LdapOp> {
        result
    }
    async fn on_password_update(
        &self,
        _context: &RequestContext,
        _user_id: &UserId,
        _password: &String,
    ) -> () {
        ()
    }
    async fn on_ldap_search_result(
        &self,
        _context: &RequestContext,
        _request: &LdapSearchRequest,
        search_result: InternalSearchResults,
    ) -> InternalSearchResults {
        search_result
    }
    async fn on_ldap_root_dse(
        &self,
        _context: &RequestContext,
        search_result_entry: LdapSearchResultEntry,
    ) -> LdapSearchResultEntry {
        search_result_entry
    }
}

#[cfg(test)]
mockall::mock! {
    pub TestLdapEventHandler {}
    impl Clone for TestLdapEventHandler {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl LdapEventHandler for TestLdapEventHandler {
        async fn on_ldap_bind(
            &self,
            context: &RequestContext,
            request: &LdapBindRequest,
            bind_result: BindResult,
        ) -> BindResult;
        async fn on_ldap_unbind(&self, context: &RequestContext, user_id: Option<UserId>) -> ();
        async fn on_ldap_modify(
            &self,
            context: &RequestContext,
            modify_request: LdapModifyRequest,
            modify_result: Vec<LdapOp>,
        ) -> Vec<LdapOp>;
        async fn on_ldap_extended_request(
            &self,
            context: &RequestContext,
            request: LdapExtendedRequest,
            result: Vec<LdapOp>,
        ) -> Vec<LdapOp>;
        async fn on_password_update(
            &self,
            context: &RequestContext,
            user_id: &UserId,
            password: &String,
        ) -> ();
        async fn on_ldap_search_result(
            &self,
            context: &RequestContext,
            request: &LdapSearchRequest,
            search_result: InternalSearchResults,
        ) -> InternalSearchResults;
        async fn on_ldap_root_dse(
            &self,
            context: &RequestContext,
            search_result_entry: LdapSearchResultEntry,
        ) -> LdapSearchResultEntry;
    }
}
