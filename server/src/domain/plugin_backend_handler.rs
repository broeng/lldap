use std::collections::HashSet;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use ldap3_proto::{
    LdapSearchResultEntry,
    proto::{LdapBindRequest, LdapExtendedRequest, LdapModifyRequest, LdapOp, LdapSearchRequest},
};
use lldap_auth::{login, registration, types::UserId};
use lldap_domain::{
    schema::Schema,
    types::{AttributeName, Group, GroupDetails, GroupId, LdapObjectClass, User, UserAndGroups},
};
use lldap_domain_handlers::{
    handler::{
        BackendHandler, BindRequest, GroupBackendHandler, GroupListerBackendHandler, LoginHandler,
        ReadSchemaBackendHandler, RequestContext, SchemaBackendHandler, UserBackendHandler,
        UserListerBackendHandler,
    },
    requests::{
        AddUserToGroupRequest, CreateAttributeRequest, CreateGroupRequest, CreateUserRequest,
        ListGroupsRequest, ListUsersRequest, RemoveUserFromGroupRequest, UpdateGroupRequest,
        UpdateUserRequest,
    },
};
use lldap_domain_model::error::Result;
use lldap_plugin_engine::api::{
    arguments::{ldap_bind_result::BindResult, ldap_search_result::SearchResult},
    handler::{PluginHandler, PluginHandlerEvents},
    types::PluginContext,
};
use lldap_plugin_kv_store::store::PluginKeyValueStore;

use lldap_ldap::{InternalSearchResults, LdapEventHandler, LdapInfo};
use lldap_opaque_handler::OpaqueHandler;
use lldap_sql_backend_handler::SqlBackendHandler;

use crate::{domain::plugin::backend::ServerBackendAPI, tcp_backend_handler::TcpBackendHandler};

use tracing::instrument;

macro_rules! wrap_call_with_plugin_callbacks {
    ($self:expr, $req_ctx:expr, $pre_handler:ident, $fn:ident, $post_handler:ident, $param:expr) => {{
        let ctx = $self.new_plugin_context($req_ctx);
        let args = $self
            .plugin_handler
            .$pre_handler(ctx.clone(), ($param).clone())
            .await
            .unwrap_or($param);
        let res = $self.backend_handler.$fn($req_ctx, args.clone()).await?;
        let _ = $self.plugin_handler.$post_handler(ctx, args).await;
        Ok(res)
    }};
}

macro_rules! wrap_call_with_plugin_callbacks_mutated_retval {
    ($self:expr, $req_ctx:expr, $pre_handler:ident, $fn:ident, $post_handler:ident, $param:expr) => {{
        let ctx = $self.new_plugin_context($req_ctx);
        let args = $self
            .plugin_handler
            .$pre_handler(ctx.clone(), ($param).clone())
            .await
            .unwrap_or($param);
        let res = $self.backend_handler.$fn($req_ctx, args).await?;
        match $self.plugin_handler.$post_handler(ctx, res.clone()).await {
            Ok(mutated_res) => Ok(mutated_res),
            Err(_) => Ok(res), // plugin callbacks failed, return result from underlying
        }
    }};
}

#[derive(Clone)]
pub struct PluginBackendHandler {
    pub(crate) backend_handler: SqlBackendHandler,
    pub(crate) backend_api: &'static ServerBackendAPI<SqlBackendHandler>,
    pub(crate) plugin_handler:
        PluginHandler<PluginKeyValueStore, ServerBackendAPI<SqlBackendHandler>>,
}

impl PluginBackendHandler {
    pub fn new(
        backend_handler: &SqlBackendHandler,
        plugin_handler: PluginHandler<PluginKeyValueStore, ServerBackendAPI<SqlBackendHandler>>,
        ldap_info: &'static LdapInfo,
    ) -> Self {
        let api: &'static ServerBackendAPI<SqlBackendHandler> =
            Box::leak(Box::new(ServerBackendAPI {
                backend_handler: backend_handler.clone(),
                ldap_info: ldap_info,
            }));
        PluginBackendHandler {
            backend_handler: backend_handler.clone(),
            backend_api: api,
            plugin_handler: plugin_handler,
        }
    }
    pub fn new_plugin_context(
        &self,
        context: &RequestContext,
    ) -> PluginContext<ServerBackendAPI<SqlBackendHandler>> {
        PluginContext::new(self.backend_api, context.clone())
    }

    pub async fn initialize_plugins(&self) -> std::result::Result<(), String> {
        let ctx = self.new_plugin_context(&RequestContext::empty());
        self.plugin_handler.initialize_plugins(ctx).await
    }
}

#[async_trait]
impl BackendHandler for PluginBackendHandler {}

#[async_trait]
impl ReadSchemaBackendHandler for PluginBackendHandler {
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn get_schema(&self, context: &RequestContext) -> Result<Schema> {
        let schema = self.backend_handler.get_schema(context).await?;
        let ctx = self.new_plugin_context(context);
        match self.plugin_handler.on_get_schema(ctx, schema.clone()).await {
            Ok(mutated_schema) => Ok(mutated_schema),
            Err(_) => Ok(schema),
        }
    }
}

#[async_trait]
impl SchemaBackendHandler for PluginBackendHandler {
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn add_user_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_add_user_attribute,
            add_user_attribute,
            on_added_user_attribute,
            request
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn add_group_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_add_group_attribute,
            add_group_attribute,
            on_added_group_attribute,
            request
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn delete_user_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_user_attribute,
            delete_user_attribute,
            on_deleted_user_attribute,
            name
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn delete_group_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_group_attribute,
            delete_group_attribute,
            on_deleted_group_attribute,
            name
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn add_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_add_user_object_class,
            add_user_object_class,
            on_added_user_object_class,
            name
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn add_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_add_group_object_class,
            add_group_object_class,
            on_added_group_object_class,
            name
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn delete_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_user_object_class,
            delete_user_object_class,
            on_deleted_user_object_class,
            name
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn delete_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_group_object_class,
            delete_group_object_class,
            on_deleted_group_object_class,
            name
        )
    }
}

#[async_trait]
impl GroupBackendHandler for PluginBackendHandler {
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn get_group_details(
        &self,
        context: &RequestContext,
        group_id: GroupId,
    ) -> Result<GroupDetails> {
        wrap_call_with_plugin_callbacks_mutated_retval!(
            self,
            context,
            on_get_group_details,
            get_group_details,
            on_get_group_details_result,
            group_id
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn update_group(
        &self,
        context: &RequestContext,
        request: UpdateGroupRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_update_group,
            update_group,
            on_updated_group,
            request
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn create_group(
        &self,
        context: &RequestContext,
        request: CreateGroupRequest,
    ) -> Result<GroupId> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_create_group,
            create_group,
            on_created_group,
            request
        )
    }
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn delete_group(&self, context: &RequestContext, group_id: GroupId) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_group,
            delete_group,
            on_deleted_group,
            group_id
        )
    }
}

#[async_trait]
impl GroupListerBackendHandler for PluginBackendHandler {
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn list_groups(
        &self,
        context: &RequestContext,
        filters: ListGroupsRequest,
    ) -> Result<Vec<Group>> {
        wrap_call_with_plugin_callbacks_mutated_retval!(
            self,
            context,
            on_list_groups,
            list_groups,
            on_list_groups_result,
            filters
        )
    }
}

#[async_trait]
impl UserBackendHandler for PluginBackendHandler {
    #[instrument(skip_all, level = "debug", ret, fields(user_id = ?user_id.as_str()))]
    async fn get_user_details(&self, context: &RequestContext, user_id: UserId) -> Result<User> {
        wrap_call_with_plugin_callbacks_mutated_retval!(
            self,
            context,
            on_get_user_details,
            get_user_details,
            on_get_user_details_result,
            user_id
        )
    }
    #[instrument(skip(self), level = "debug", err, fields(user_id = ?request.user_id.as_str()))]
    async fn create_user(
        &self,
        context: &RequestContext,
        request: CreateUserRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_create_user,
            create_user,
            on_created_user,
            request
        )
    }
    #[instrument(skip(self), level = "debug", err, fields(user_id = ?request.user_id.as_str()))]
    async fn update_user(
        &self,
        context: &RequestContext,
        request: UpdateUserRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_update_user,
            update_user,
            on_updated_user,
            request
        )
    }
    #[instrument(skip_all, level = "debug", err, fields(user_id = ?user_id.as_str()))]
    async fn delete_user(&self, context: &RequestContext, user_id: UserId) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_delete_user,
            delete_user,
            on_deleted_user,
            user_id
        )
    }
    #[instrument(skip_all, level = "debug", err, fields(user_id = ?request.user_id.as_str(), request.group_id))]
    async fn add_user_to_group(
        &self,
        context: &RequestContext,
        request: AddUserToGroupRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_add_user_to_group,
            add_user_to_group,
            on_added_user_to_group,
            request
        )
    }
    #[instrument(skip_all, level = "debug", err, fields(user_id = ?request.user_id.as_str(), request.group_id))]
    async fn remove_user_from_group(
        &self,
        context: &RequestContext,
        request: RemoveUserFromGroupRequest,
    ) -> Result<()> {
        wrap_call_with_plugin_callbacks!(
            self,
            context,
            on_remove_user_from_group,
            remove_user_from_group,
            on_removed_user_from_group,
            request
        )
    }
    #[instrument(skip_all, level = "debug", ret, fields(user_id = ?user_id.as_str()))]
    async fn get_user_groups(
        &self,
        context: &RequestContext,
        user_id: UserId,
    ) -> Result<HashSet<GroupDetails>> {
        wrap_call_with_plugin_callbacks_mutated_retval!(
            self,
            context,
            on_get_user_groups,
            get_user_groups,
            on_get_user_groups_result,
            user_id
        )
    }
}

#[async_trait]
impl UserListerBackendHandler for PluginBackendHandler {
    #[instrument(skip(self), level = "debug", ret, err)]
    async fn list_users(
        &self,
        context: &RequestContext,
        filters: ListUsersRequest,
    ) -> Result<Vec<UserAndGroups>> {
        wrap_call_with_plugin_callbacks_mutated_retval!(
            self,
            context,
            on_list_users,
            list_users,
            on_list_users_result,
            filters
        )
    }
}

#[async_trait]
impl LoginHandler for PluginBackendHandler {
    async fn bind(&self, context: &RequestContext, request: BindRequest) -> Result<()> {
        self.backend_handler.bind(context, request).await
    }
}

#[async_trait]
impl OpaqueHandler for PluginBackendHandler {
    async fn login_start(
        &self,
        request: login::ClientLoginStartRequest,
    ) -> Result<login::ServerLoginStartResponse> {
        self.backend_handler.login_start(request).await
    }
    async fn login_finish(&self, request: login::ClientLoginFinishRequest) -> Result<UserId> {
        self.backend_handler.login_finish(request).await
    }
    async fn registration_start(
        &self,
        request: registration::ClientRegistrationStartRequest,
    ) -> Result<registration::ServerRegistrationStartResponse> {
        self.backend_handler.registration_start(request).await
    }
    async fn registration_finish(
        &self,
        request: registration::ClientRegistrationFinishRequest,
    ) -> Result<()> {
        self.backend_handler.registration_finish(request).await
    }
}

#[async_trait]
impl TcpBackendHandler for PluginBackendHandler {
    async fn get_jwt_blacklist(&self) -> anyhow::Result<HashSet<u64>> {
        self.backend_handler.get_jwt_blacklist().await
    }
    async fn create_refresh_token(&self, user: &UserId) -> Result<(String, chrono::Duration)> {
        self.backend_handler.create_refresh_token(user).await
    }
    async fn register_jwt(
        &self,
        user: &UserId,
        jwt_hash: u64,
        expiry_date: NaiveDateTime,
    ) -> Result<()> {
        self.backend_handler
            .register_jwt(user, jwt_hash, expiry_date)
            .await
    }
    async fn check_token(&self, refresh_token_hash: u64, user: &UserId) -> Result<bool> {
        self.backend_handler
            .check_token(refresh_token_hash, user)
            .await
    }
    async fn blacklist_jwts(&self, user: &UserId) -> Result<HashSet<u64>> {
        self.backend_handler.blacklist_jwts(user).await
    }
    async fn delete_refresh_token(&self, refresh_token_hash: u64) -> Result<()> {
        self.backend_handler
            .delete_refresh_token(refresh_token_hash)
            .await
    }
    async fn start_password_reset(&self, user: &UserId) -> Result<Option<String>> {
        self.backend_handler.start_password_reset(user).await
    }
    async fn get_user_id_for_password_reset_token(&self, token: &str) -> Result<UserId> {
        self.backend_handler
            .get_user_id_for_password_reset_token(token)
            .await
    }
    async fn delete_password_reset_token(&self, token: &str) -> Result<()> {
        self.backend_handler
            .delete_password_reset_token(token)
            .await
    }
}

#[async_trait]
impl LdapEventHandler for PluginBackendHandler {
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_bind(
        &self,
        context: &RequestContext,
        request: &LdapBindRequest,
        bind_result: BindResult,
    ) -> BindResult {
        let context = self.new_plugin_context(context);
        self.plugin_handler
            .on_ldap_bind(context, request.clone(), bind_result.clone())
            .await
            .unwrap_or(bind_result)
    }
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_unbind(&self, context: &RequestContext, user_id: Option<UserId>) -> () {
        if let Some(uid) = user_id {
            let context = self.new_plugin_context(context);
            let _ = self.plugin_handler.on_ldap_unbind(context, uid).await;
        }
    }
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_modify(
        &self,
        context: &RequestContext,
        modify_request: LdapModifyRequest,
        modify_result: Vec<LdapOp>,
    ) -> Vec<LdapOp> {
        let context = self.new_plugin_context(context);
        self.plugin_handler
            .on_ldap_modify(context, modify_request.clone(), modify_result.clone())
            .await
            .unwrap_or(modify_result)
    }
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_extended_request(
        &self,
        context: &RequestContext,
        request: LdapExtendedRequest,
        result: Vec<LdapOp>,
    ) -> Vec<LdapOp> {
        let context = self.new_plugin_context(context);
        self.plugin_handler
            .on_ldap_extended_request(context, request.clone(), result.clone())
            .await
            .unwrap_or(result)
    }
    #[instrument(skip(self, password), level = "debug")]
    async fn on_password_update(
        &self,
        context: &RequestContext,
        user_id: &UserId,
        password: &[u8],
    ) -> () {
        let context = self.new_plugin_context(context);
        let _ = self
            .plugin_handler
            .on_ldap_password_update(context, user_id.clone(), password)
            .await;
    }
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_search_result(
        &self,
        context: &RequestContext,
        request: &LdapSearchRequest,
        search_result: InternalSearchResults,
    ) -> InternalSearchResults {
        let context = self.new_plugin_context(context);
        self.plugin_handler
            .on_ldap_search_result(context, request.clone(), search_result.clone().into())
            .await
            .map(SearchResult::into)
            .unwrap_or(search_result)
    }
    #[instrument(skip_all(), level = "debug")]
    async fn on_ldap_root_dse(
        &self,
        context: &RequestContext,
        search_result_entry: LdapSearchResultEntry,
    ) -> LdapSearchResultEntry {
        let context = self.new_plugin_context(context);
        self.plugin_handler
            .on_ldap_root_dse(context, search_result_entry.clone())
            .await
            .unwrap_or(search_result_entry)
    }
}
