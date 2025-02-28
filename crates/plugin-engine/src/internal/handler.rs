use std::collections::HashSet;

use async_trait::async_trait;

use ldap3_proto::{
    proto::{LdapBindRequest, LdapExtendedRequest, LdapModifyRequest, LdapOp, LdapSearchRequest},
    LdapSearchResultEntry,
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
        handler::{PluginHandler, PluginHandlerEvents},
        types::PluginContext,
    },
    internal::{
        arguments::{
            bind_request::BindRequestArguments, create_group::CreateGroupArguments,
            create_user::CreateUserArguments, delete_group::DeleteGroupArguments,
            delete_user::DeleteUserArguments, extended_request::ExtendedRequestArguments,
            ldap_search_result_entry::LdapSearchResultEntryArguments,
            modify_request::ModifyRequestArguments, restricted_secret::RestrictedSecret,
            search_result::SearchResultArguments, update_group::UpdateGroupArguments,
            update_password::UpdatePasswordArguments, update_user::UpdateUserArguments,
            user_and_group::UserAndGroupArguments, user_groups::UserGroupsArguments,
        },
        context::{
            parameters::create_attribute::CreateAttributeParams, plugin_context::LuaPluginContext,
        },
        exec::{exec_mutation_handler, exec_mutation_handler_alt, exec_notification_handler},
        secrets,
        types::{
            group::LuaGroupsVec, plugins::Plugin, schema::LuaSchema, secret::Secret,
            user::LuaUserAndGroupsVec,
        },
    },
};

use tracing::{debug, debug_span, instrument, Instrument};

use super::{
    context::parameters::list_query::ListQueryParam,
    types::{group::LuaGroupDetails, user::LuaUser},
};

macro_rules! invoke_andthen_mutation_handler {
    ($self:expr, $ctx:expr, $handler:ident, $fromargt:expr, $toargt:expr, $args:expr) => {{
        if ($self.plugin_registry.$handler.is_empty()) {
            Ok($args)
        } else {
            exec_mutation_handler(
                $ctx,
                $self.kvstore.clone(),
                &$self.plugin_registry.lua,
                &$self.plugin_registry.$handler,
                $fromargt,
            )
            .await
            .and_then($toargt)
        }
    }};
}

macro_rules! invoke_mutation_handler {
    ($self:expr, $ctx:expr, $handler:ident, $fromargt:expr, $toargt:expr, $args:expr) => {{
        invoke_andthen_mutation_handler!(
            $self,
            $ctx,
            $handler,
            $fromargt,
            |a| Ok($toargt(a)),
            $args
        )
    }};
}

macro_rules! invoke_notification_handler {
    ($self:expr, $ctx:expr, $handler:ident, $fromargt:expr, $args:expr) => {{
        if !$self.plugin_registry.$handler.is_empty() {
            exec_notification_handler(
                $ctx,
                $self.kvstore.clone(),
                &$self.plugin_registry.lua,
                &$self.plugin_registry.$handler,
                $fromargt,
            )
            .await?
        }
        Ok(())
    }};
}

#[async_trait]
impl<KVStore: KeyValueStore + 'static, API: BackendAPI> PluginHandlerEvents<API>
    for PluginHandler<KVStore, API>
{
    #[instrument(skip(self, context), level = "debug", err)]
    async fn initialize_plugins(&self, context: PluginContext<API>) -> Result<(), String> {
        if !self.plugin_registry.init.is_empty() {
            // Execute and notify the registered plugins
            for cb in self.plugin_registry.init.iter() {
                // Obtain reference to plugin being executed
                let plugin_ref: &Plugin = cb.plugin.as_ref();
                // Prepare actual context for plugin
                let ctx = LuaPluginContext {
                    api: context.api,
                    configuration: plugin_ref.configuration.clone(),
                    context: context.request_context.clone(),
                    kvstore: self.kvstore.clone(),
                    kvscope: plugin_ref.kvstore_scope.clone(),
                    lua: self.plugin_registry.lua,
                };
                let plugin_name = plugin_ref.name.clone();
                let span = debug_span!("[Lua Plugin Handler]");
                span.in_scope(|| {
                    debug!(plugin_name);
                });
                let _: () = cb
                    .callback
                    .call_async(ctx.clone())
                    .instrument(span)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    #[instrument(skip(self, context), level = "debug", err, fields(user_id = ?args.user_id.as_str()))]
    async fn on_create_user(
        &self,
        context: PluginContext<API>,
        args: CreateUserRequest,
    ) -> Result<CreateUserRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_create_user,
            CreateUserArguments::from(args),
            CreateUserArguments::into_request,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err, fields(user_id = ?args.user_id.as_str()))]
    async fn on_created_user(
        &self,
        context: PluginContext<API>,
        args: CreateUserRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_created_user,
            CreateUserArguments::from(args),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err, fields(user_id = ?args.user_id.as_str()))]
    async fn on_update_user(
        &self,
        context: PluginContext<API>,
        args: UpdateUserRequest,
    ) -> Result<UpdateUserRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_update_user,
            UpdateUserArguments::from(args),
            UpdateUserArguments::into_request,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err, fields(user_id = ?args.user_id.as_str()))]
    async fn on_updated_user(
        &self,
        context: PluginContext<API>,
        args: UpdateUserRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_updated_user,
            UpdateUserArguments::from(args),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_user(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<UserId, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_user,
            DeleteUserArguments::from(user_id),
            DeleteUserArguments::into_user_id,
            user_id
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_user(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_user,
            DeleteUserArguments::from(user_id),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err, fields(display_name = ?args.display_name.as_str()))]
    async fn on_create_group(
        &self,
        context: PluginContext<API>,
        args: CreateGroupRequest,
    ) -> Result<CreateGroupRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_create_group,
            CreateGroupArguments::from(args),
            CreateGroupArguments::into_request,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err, fields(display_name = ?args.display_name.as_str()))]
    async fn on_created_group(
        &self,
        context: PluginContext<API>,
        args: CreateGroupRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_created_user,
            CreateGroupArguments::from(args),
            args
        )
    }

    #[instrument(skip(self, context, args), level = "debug", err)]
    async fn on_update_group(
        &self,
        context: PluginContext<API>,
        args: UpdateGroupRequest,
    ) -> Result<UpdateGroupRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_update_group,
            UpdateGroupArguments::from(args),
            UpdateGroupArguments::into_request,
            args
        )
    }

    #[instrument(skip(self, context, args), level = "debug")]
    async fn on_updated_group(
        &self,
        context: PluginContext<API>,
        args: UpdateGroupRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_updated_user,
            UpdateGroupArguments::from(args),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_group(
        &self,
        context: PluginContext<API>,
        group_id: GroupId,
    ) -> Result<GroupId, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_group,
            DeleteGroupArguments::from(group_id),
            DeleteGroupArguments::into_group_id,
            group_id
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_group(
        &self,
        context: PluginContext<API>,
        group_id: GroupId,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_user,
            DeleteGroupArguments::from(group_id),
            group_id
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_add_user_to_group(
        &self,
        context: PluginContext<API>,
        args: AddUserToGroupRequest,
    ) -> Result<AddUserToGroupRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_add_user_to_group,
            UserAndGroupArguments::new(args.user_id, args.group_id),
            UserAndGroupArguments::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_added_user_to_group(
        &self,
        context: PluginContext<API>,
        args: AddUserToGroupRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_added_user_to_group,
            UserAndGroupArguments::new(args.user_id, args.group_id),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_remove_user_from_group(
        &self,
        context: PluginContext<API>,
        args: RemoveUserFromGroupRequest,
    ) -> Result<RemoveUserFromGroupRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_remove_user_from_group,
            UserAndGroupArguments::new(args.user_id, args.group_id),
            UserAndGroupArguments::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_removed_user_from_group(
        &self,
        context: PluginContext<API>,
        args: RemoveUserFromGroupRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_removed_user_from_group,
            UserAndGroupArguments::new(args.user_id, args.group_id),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_get_user_details(
        &self,
        context: PluginContext<API>,
        args: UserId,
    ) -> Result<UserId, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_user_details,
            args.into_string(),
            UserId::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_get_user_details_result(
        &self,
        context: PluginContext<API>,
        args: User,
    ) -> Result<User, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_user_details_result,
            LuaUser::from(args),
            LuaUser::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_get_group_details(
        &self,
        context: PluginContext<API>,
        args: GroupId,
    ) -> Result<GroupId, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_group_details,
            args.0,
            |i| GroupId(i),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_get_group_details_result(
        &self,
        context: PluginContext<API>,
        args: GroupDetails,
    ) -> Result<GroupDetails, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_user_details_result,
            LuaGroupDetails::from(args),
            LuaGroupDetails::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_list_users(
        &self,
        context: PluginContext<API>,
        args: ListUsersRequest,
    ) -> Result<ListUsersRequest, String> {
        invoke_andthen_mutation_handler!(
            self,
            context,
            on_list_users,
            ListQueryParam::from(args),
            ListQueryParam::try_into_user_request,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_list_users_result(
        &self,
        context: PluginContext<API>,
        args: Vec<UserAndGroups>,
    ) -> Result<Vec<UserAndGroups>, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_list_users_result,
            LuaUserAndGroupsVec::from(args),
            LuaUserAndGroupsVec::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_list_groups(
        &self,
        context: PluginContext<API>,
        args: ListGroupsRequest,
    ) -> Result<ListGroupsRequest, String> {
        invoke_andthen_mutation_handler!(
            self,
            context,
            on_list_groups,
            ListQueryParam::from(args),
            ListQueryParam::try_into_group_request,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_list_groups_result(
        &self,
        context: PluginContext<API>,
        args: Vec<Group>,
    ) -> Result<Vec<Group>, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_list_groups_result,
            LuaGroupsVec::from(args),
            LuaGroupsVec::into,
            args
        )
    }

    #[instrument(skip_all, level = "debug", err)]
    async fn on_get_schema(
        &self,
        context: PluginContext<API>,
        schema: Schema,
    ) -> Result<Schema, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_schema,
            LuaSchema::from(schema),
            LuaSchema::into,
            schema
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_get_user_groups(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<UserId, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_user_groups,
            user_id.into_string(),
            UserId::from,
            user_id
        )
    }

    #[instrument(skip_all, level = "debug", err)]
    async fn on_get_user_groups_result(
        &self,
        context: PluginContext<API>,
        groups: HashSet<GroupDetails>,
    ) -> Result<HashSet<GroupDetails>, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_get_user_groups_result,
            UserGroupsArguments::from(groups),
            UserGroupsArguments::into,
            groups
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_add_user_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<CreateAttributeRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_add_user_attribute,
            CreateAttributeParams::from(args),
            CreateAttributeParams::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_added_user_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_added_user_attribute,
            CreateAttributeParams::from(args),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_add_group_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<CreateAttributeRequest, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_add_group_attribute,
            CreateAttributeParams::from(args),
            CreateAttributeParams::into,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_added_group_attribute(
        &self,
        context: PluginContext<API>,
        args: CreateAttributeRequest,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_added_group_attribute,
            CreateAttributeParams::from(args),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_user_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<AttributeName, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_user_attribute,
            args.into_string(),
            AttributeName::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_user_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_user_attribute,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_group_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<AttributeName, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_group_attribute,
            args.into_string(),
            AttributeName::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_group_attribute(
        &self,
        context: PluginContext<API>,
        args: AttributeName,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_group_attribute,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_add_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_add_user_object_class,
            args.into_string(),
            LdapObjectClass::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_added_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_added_user_object_class,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_add_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_add_group_object_class,
            args.into_string(),
            LdapObjectClass::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_added_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_added_group_object_class,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_user_object_class,
            args.into_string(),
            LdapObjectClass::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_user_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_user_object_class,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_delete_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<LdapObjectClass, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_delete_group_object_class,
            args.into_string(),
            LdapObjectClass::from,
            args
        )
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_deleted_group_object_class(
        &self,
        context: PluginContext<API>,
        args: LdapObjectClass,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_deleted_group_object_class,
            args.into_string(),
            args
        )
    }

    #[instrument(skip(self, context, bind_request), level = "debug", err)]
    async fn on_ldap_bind(
        &self,
        context: PluginContext<API>,
        bind_request: LdapBindRequest,
        bind_result: BindResult,
    ) -> Result<BindResult, String> {
        if self.plugin_registry.on_ldap_bind.is_empty() {
            Ok(bind_result)
        } else {
            let secret = Secret::try_from_bind_request(&bind_request).map(RestrictedSecret::new);
            exec_mutation_handler_alt(
                context,
                self.kvstore.clone(),
                &self.plugin_registry.lua,
                &self.plugin_registry.on_ldap_bind,
                BindRequestArguments {
                    bind_result,
                    bind_request: secrets::strip_bind_password(bind_request),
                },
                secret,
                apply_permission_to_secret,
            )
            .await
            .map(|r| r.bind_result)
        }
    }

    #[instrument(skip(self, context), level = "debug", err)]
    async fn on_ldap_unbind(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
    ) -> Result<(), String> {
        invoke_notification_handler!(
            self,
            context,
            on_ldap_unbind,
            user_id.into_string(),
            user_id
        )
    }

    #[instrument(skip_all(), level = "debug", err)]
    async fn on_ldap_modify(
        &self,
        context: PluginContext<API>,
        modify_request: LdapModifyRequest,
        modify_result: Vec<LdapOp>,
    ) -> Result<Vec<LdapOp>, String> {
        if self.plugin_registry.on_ldap_modify.is_empty() {
            Ok(modify_result)
        } else {
            let secret =
                Secret::try_from_modify_request(&modify_request).map(RestrictedSecret::new);
            exec_mutation_handler_alt(
                context,
                self.kvstore.clone(),
                &self.plugin_registry.lua,
                &self.plugin_registry.on_ldap_modify,
                ModifyRequestArguments {
                    modify_result,
                    modify_request: secrets::strip_modify_password(modify_request),
                },
                secret,
                apply_permission_to_secret,
            )
            .await
            .map(|r| r.modify_result)
        }
    }

    #[instrument(skip_all(), level = "debug", err)]
    async fn on_ldap_extended_request(
        &self,
        context: PluginContext<API>,
        request: LdapExtendedRequest,
        result: Vec<LdapOp>,
    ) -> Result<Vec<LdapOp>, String> {
        if self.plugin_registry.on_ldap_extended_request.is_empty() {
            Ok(result)
        } else {
            let secret = Secret::try_from_extended_request(&request).map(RestrictedSecret::new);
            exec_mutation_handler_alt(
                context,
                self.kvstore.clone(),
                &self.plugin_registry.lua,
                &self.plugin_registry.on_ldap_extended_request,
                ExtendedRequestArguments {
                    extended_result: result,
                    extended_request: secrets::strip_change_password_extended(request),
                },
                secret,
                apply_permission_to_secret,
            )
            .await
            .map(|r| r.extended_result)
        }
    }

    #[instrument(skip(self, context, password), level = "debug", err)]
    async fn on_ldap_password_update(
        &self,
        context: PluginContext<API>,
        user_id: UserId,
        password: String,
    ) -> Result<(), String> {
        if !self.plugin_registry.on_ldap_extended_request.is_empty() {
            let secret = Some(RestrictedSecret::new(password.into()));
            let _ = exec_mutation_handler_alt(
                context,
                self.kvstore.clone(),
                &self.plugin_registry.lua,
                &self.plugin_registry.on_ldap_password_update,
                UpdatePasswordArguments {
                    user_id: user_id.into_string(),
                },
                secret,
                apply_permission_to_secret,
            )
            .await?;
        }
        Ok(())
    }

    #[instrument(skip_all(), level = "debug", err)]
    async fn on_ldap_search_result(
        &self,
        context: PluginContext<API>,
        search_request: LdapSearchRequest,
        search_result: SearchResult,
    ) -> Result<SearchResult, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_ldap_search_result,
            SearchResultArguments::new(search_result, search_request),
            |a: SearchResultArguments| a.search_result.into(),
            search_result
        )
    }

    #[instrument(skip_all(), level = "debug", err)]
    async fn on_ldap_root_dse(
        &self,
        context: PluginContext<API>,
        search_result_entry: LdapSearchResultEntry,
    ) -> Result<LdapSearchResultEntry, String> {
        invoke_mutation_handler!(
            self,
            context,
            on_ldap_root_dse,
            LdapSearchResultEntryArguments::new(search_result_entry),
            |r: LdapSearchResultEntryArguments| r.search_result_entry,
            search_result_entry
        )
    }
}

fn apply_permission_to_secret(
    rs: &Option<RestrictedSecret>,
    plugin: &Plugin,
) -> Option<RestrictedSecret> {
    rs.clone()
        .map(|s| s.with_permissions(plugin.permissions.secrets.clone()))
}
