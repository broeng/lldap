use std::{collections::HashSet, marker::PhantomData};

use async_trait::async_trait;
use ldap3_proto::{
    LdapSearchResultEntry,
    proto::{LdapBindRequest, LdapExtendedRequest, LdapModifyRequest, LdapOp, LdapSearchRequest},
};
use lldap_domain::{
    schema::Schema,
    types::{
        AttributeName, Group, GroupDetails, GroupId, LdapObjectClass, User, UserAndGroups, UserId,
    },
};
use lldap_domain_handlers::requests::{
    AddUserToGroupRequest, CreateAttributeRequest, CreateGroupRequest, CreateUserRequest,
    ListGroupsRequest, ListUsersRequest, RemoveUserFromGroupRequest, UpdateGroupRequest,
    UpdateUserRequest,
};
use lldap_key_value_store::api::store::KeyValueStore;

use crate::{
    api::{
        arguments::{ldap_bind_result::BindResult, ldap_search_result::SearchResult},
        backend::BackendAPI,
        types::{PluginConfig, PluginContext},
    },
    internal::{plugins, types::plugins::PluginRegistry},
};

#[async_trait]
pub trait PluginHandlerEvents<API>
where
    API: BackendAPI,
{
    async fn initialize_plugins(&self, context: PluginContext<API>) -> Result<(), String>;

    async fn on_create_user(
        &self,
        context: PluginContext<API>,
        args: CreateUserRequest,
    ) -> Result<CreateUserRequest, String>;

    async fn on_created_user(
        &self,
        context: PluginContext<API>,
        args: CreateUserRequest,
    ) -> Result<(), String>;

    async fn on_update_user(
        &self,
        context: PluginContext<API>,
        args: UpdateUserRequest,
    ) -> Result<UpdateUserRequest, String>;

    async fn on_updated_user(
        &self,
        context: PluginContext<API>,
        args: UpdateUserRequest,
    ) -> Result<(), String>;

    async fn on_delete_user(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<UserId, String>;

    async fn on_deleted_user(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<(), String>;

    async fn on_create_group(
        &self,
        context: PluginContext<API>,
        args: CreateGroupRequest,
    ) -> Result<CreateGroupRequest, String>;

    async fn on_created_group(
        &self,
        context: PluginContext<API>,
        args: CreateGroupRequest,
    ) -> Result<(), String>;

    async fn on_update_group(
        &self,
        context: PluginContext<API>,
        args: UpdateGroupRequest,
    ) -> Result<UpdateGroupRequest, String>;

    async fn on_updated_group(
        &self,
        context: PluginContext<API>,
        args: UpdateGroupRequest,
    ) -> Result<(), String>;

    async fn on_delete_group(
        &self,
        context: PluginContext<API>,
        group_id: GroupId,
    ) -> Result<GroupId, String>;

    async fn on_deleted_group(
        &self,
        context: PluginContext<API>,
        group_id: GroupId,
    ) -> Result<(), String>;

    async fn on_add_user_to_group(
        &self,
        context: PluginContext<API>,
        args: AddUserToGroupRequest,
    ) -> Result<AddUserToGroupRequest, String>;

    async fn on_added_user_to_group(
        &self,
        context: PluginContext<API>,
        args: AddUserToGroupRequest,
    ) -> Result<(), String>;

    async fn on_remove_user_from_group(
        &self,
        context: PluginContext<API>,
        args: RemoveUserFromGroupRequest,
    ) -> Result<RemoveUserFromGroupRequest, String>;

    async fn on_removed_user_from_group(
        &self,
        context: PluginContext<API>,
        args: RemoveUserFromGroupRequest,
    ) -> Result<(), String>;

    async fn on_get_user_details(
        &self,
        context: PluginContext<API>,
        args: UserId,
    ) -> Result<UserId, String>;

    async fn on_get_user_details_result(
        &self,
        context: PluginContext<API>,
        args: User,
    ) -> Result<User, String>;

    async fn on_get_group_details(
        &self,
        context: PluginContext<API>,
        args: GroupId,
    ) -> Result<GroupId, String>;

    async fn on_get_group_details_result(
        &self,
        context: PluginContext<API>,
        args: GroupDetails,
    ) -> Result<GroupDetails, String>;

    async fn on_list_users(
        &self,
        context: PluginContext<API>,
        args: ListUsersRequest,
    ) -> Result<ListUsersRequest, String>;

    async fn on_list_users_result(
        &self,
        context: PluginContext<API>,
        args: Vec<UserAndGroups>,
    ) -> Result<Vec<UserAndGroups>, String>;

    async fn on_list_groups(
        &self,
        context: PluginContext<API>,
        args: ListGroupsRequest,
    ) -> Result<ListGroupsRequest, String>;

    async fn on_list_groups_result(
        &self,
        context: PluginContext<API>,
        args: Vec<Group>,
    ) -> Result<Vec<Group>, String>;

    async fn on_get_schema(
        &self,
        context: PluginContext<API>,
        schema: Schema,
    ) -> Result<Schema, String>;

    async fn on_get_user_groups(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<UserId, String>;

    async fn on_get_user_groups_result(
        &self,
        context: PluginContext<API>,
        groups: HashSet<GroupDetails>,
    ) -> Result<HashSet<GroupDetails>, String>;

    async fn on_add_user_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<CreateAttributeRequest, String>;

    async fn on_added_user_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<(), String>;

    async fn on_add_group_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<CreateAttributeRequest, String>;

    async fn on_added_group_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<(), String>;

    async fn on_delete_user_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<AttributeName, String>;

    async fn on_deleted_user_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<(), String>;

    async fn on_delete_group_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<AttributeName, String>;

    async fn on_deleted_group_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<(), String>;

    async fn on_add_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String>;

    async fn on_added_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String>;

    async fn on_add_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String>;

    async fn on_added_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String>;

    async fn on_delete_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String>;

    async fn on_deleted_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String>;

    async fn on_delete_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String>;

    async fn on_deleted_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String>;

    async fn on_ldap_bind(
        &self,
        context: PluginContext<API>,
        bind_request: LdapBindRequest,
        bind_result: BindResult,
    ) -> Result<BindResult, String>;

    async fn on_ldap_unbind(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<(), String>;

    async fn on_ldap_modify(
        &self,
        context: PluginContext<API>,
        modify_request: LdapModifyRequest,
        modify_result: Vec<LdapOp>,
    ) -> Result<Vec<LdapOp>, String>;

    async fn on_ldap_extended_request(
        &self,
        context: PluginContext<API>,
        request: LdapExtendedRequest,
        result: Vec<LdapOp>,
    ) -> Result<Vec<LdapOp>, String>;

    async fn on_ldap_password_update(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
        password: &[u8],
    ) -> Result<(), String>;

    async fn on_ldap_search_result(
        &self,
        context: PluginContext<API>,
        search_request: LdapSearchRequest,
        search_result: SearchResult,
    ) -> Result<SearchResult, String>;

    async fn on_ldap_root_dse(
        &self,
        context: PluginContext<API>,
        search_result_entry: LdapSearchResultEntry,
    ) -> Result<LdapSearchResultEntry, String>;
}

#[derive(Clone)]
pub struct PluginHandler<KVStore: KeyValueStore + 'static, API: BackendAPI + 'static> {
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) kvstore: KVStore,
    __phantom: PhantomData<API>,
}

impl<KVStore: KeyValueStore + 'static, API: BackendAPI + 'static> PluginHandler<KVStore, API> {
    pub fn new(
        plugins: Vec<PluginConfig>,
        kvstore: KVStore,
    ) -> Result<PluginHandler<KVStore, API>, String> {
        let plugins = plugins::load_plugins(plugins).map_err(|e| e.to_string())?;
        Ok(PluginHandler {
            plugin_registry: plugins,
            kvstore,
            __phantom: PhantomData::default(),
        })
    }
}
