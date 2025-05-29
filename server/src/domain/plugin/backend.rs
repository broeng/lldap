use std::collections::HashSet;

use async_trait::async_trait;

use lldap_auth::types::UserId;
use lldap_plugin_engine::api::{backend::BackendAPI, types::QueryFilter};

use lldap_domain::{
    schema::Schema,
    types::{AttributeName, Group, GroupDetails, GroupId, LdapObjectClass, User, UserAndGroups},
};
use lldap_domain_handlers::{
    handler::{BackendHandler, GroupRequestFilter, RequestContext, UserRequestFilter},
    requests::{
        AddUserToGroupRequest, CreateAttributeRequest, CreateGroupRequest, CreateUserRequest,
        ListGroupsRequest, ListUsersRequest, RemoveUserFromGroupRequest, UpdateGroupRequest,
        UpdateUserRequest,
    },
};

use crate::domain::{
    ldap::{group::convert_group_filter, user::convert_user_filter, utils::LdapInfo},
    schema::PublicSchema,
};

use ldap3_proto::{filter, proto::LdapFilter};
use tracing::instrument;

#[derive(Clone)]
pub struct ServerBackendAPI<Handler: BackendHandler> {
    pub backend_handler: Handler,
    pub ldap_info: LdapInfo,
}

#[async_trait]
impl<B: BackendHandler + Clone> BackendAPI for ServerBackendAPI<B> {
    //
    // Read Schema
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn get_schema(&self, context: &RequestContext) -> Result<Schema, String> {
        self.backend_handler
            .get_schema(context)
            .await
            .map_err(|e| e.to_string())
    }
    //
    // User Listing
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn list_users(
        &self,
        context: &RequestContext,
        filters: Option<QueryFilter>,
    ) -> Result<Vec<UserAndGroups>, String> {
        let user_filter: Option<UserRequestFilter> = match filters {
            Some(f) => match f {
                QueryFilter::LdapFilter(s) => Some(parse_user_filter(
                    self.get_schema(context).await?,
                    &self.ldap_info,
                    s,
                )?),
                QueryFilter::UserFilter(u) => u.filter,
                QueryFilter::GroupFilter(_) => {
                    Err("Group filter cannot be used for listing users".to_string())?
                }
            },
            None => None,
        };
        self.backend_handler
            .list_users(
                context,
                ListUsersRequest {
                    filter: user_filter,
                    need_groups: true,
                },
            )
            .await
            .map_err(|e| e.to_string())
    }
    //
    // Group Listing
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn list_groups(
        &self,
        context: &RequestContext,
        filters: Option<QueryFilter>,
    ) -> Result<Vec<Group>, String> {
        let group_filter: Option<GroupRequestFilter> = match filters {
            Some(f) => match f {
                QueryFilter::LdapFilter(s) => Some(parse_group_filter(
                    self.get_schema(context).await?,
                    &self.ldap_info,
                    s,
                )?),
                QueryFilter::GroupFilter(g) => g.filter,
                QueryFilter::UserFilter(_) => {
                    Err("User filter cannot be used for listing groups".to_string())?
                }
            },
            None => None,
        };
        self.backend_handler
            .list_groups(
                context,
                ListGroupsRequest {
                    filter: group_filter,
                },
            )
            .await
            .map_err(|e| e.to_string())
    }
    //
    // Schema
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn add_user_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<(), String> {
        self.backend_handler
            .add_user_attribute(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn add_group_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<(), String> {
        self.backend_handler
            .add_group_attribute(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_user_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<(), String> {
        self.backend_handler
            .delete_user_attribute(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_group_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<(), String> {
        self.backend_handler
            .delete_group_attribute(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn add_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String> {
        self.backend_handler
            .add_user_object_class(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn add_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String> {
        self.backend_handler
            .add_group_object_class(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String> {
        self.backend_handler
            .delete_user_object_class(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String> {
        self.backend_handler
            .delete_group_object_class(context, name)
            .await
            .map_err(|e| e.to_string())
    }
    //
    // Groups
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn get_group_details(
        &self,
        context: &RequestContext,
        group_id: GroupId,
    ) -> Result<GroupDetails, String> {
        self.backend_handler
            .get_group_details(context, group_id)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn update_group(
        &self,
        context: &RequestContext,
        request: UpdateGroupRequest,
    ) -> Result<(), String> {
        self.backend_handler
            .update_group(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn create_group(
        &self,
        context: &RequestContext,
        request: CreateGroupRequest,
    ) -> Result<GroupId, String> {
        self.backend_handler
            .create_group(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_group(
        &self,
        context: &RequestContext,
        group_id: GroupId,
    ) -> Result<(), String> {
        self.backend_handler
            .delete_group(context, group_id)
            .await
            .map_err(|e| e.to_string())
    }
    //
    // Users
    //
    #[instrument(skip(self, context), level = "debug", err)]
    async fn get_user_details(
        &self,
        context: &RequestContext,
        user_id: UserId,
    ) -> Result<User, String> {
        self.backend_handler
            .get_user_details(context, user_id)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn create_user(
        &self,
        context: &RequestContext,
        request: CreateUserRequest,
    ) -> Result<(), String> {
        self.backend_handler
            .create_user(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn update_user(
        &self,
        context: &RequestContext,
        request: UpdateUserRequest,
    ) -> Result<(), String> {
        self.backend_handler
            .update_user(context, request)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn delete_user(&self, context: &RequestContext, user_id: UserId) -> Result<(), String> {
        self.backend_handler
            .delete_user(context, user_id)
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn add_user_to_group(
        &self,
        context: &RequestContext,
        user_id: UserId,
        group_id: GroupId,
    ) -> Result<(), String> {
        self.backend_handler
            .add_user_to_group(context, AddUserToGroupRequest { user_id, group_id })
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn remove_user_from_group(
        &self,
        context: &RequestContext,
        user_id: UserId,
        group_id: GroupId,
    ) -> Result<(), String> {
        self.backend_handler
            .remove_user_from_group(context, RemoveUserFromGroupRequest { user_id, group_id })
            .await
            .map_err(|e| e.to_string())
    }
    #[instrument(skip(self, context), level = "debug", err)]
    async fn get_user_groups(
        &self,
        context: &RequestContext,
        user_id: UserId,
    ) -> Result<HashSet<GroupDetails>, String> {
        self.backend_handler
            .get_user_groups(context, user_id)
            .await
            .map_err(|e| e.to_string())
    }
}

fn parse_user_filter(
    schema: Schema,
    ldap_info: &LdapInfo,
    filter: String,
) -> Result<UserRequestFilter, String> {
    let ldap_filter: LdapFilter =
        filter::parse_ldap_filter_str(filter.as_str()).map_err(|e| e.to_string())?;
    let pub_schema = PublicSchema::from(schema);
    convert_user_filter(ldap_info, &ldap_filter, &pub_schema).map_err(|e| e.to_string())
}

fn parse_group_filter(
    schema: Schema,
    ldap_info: &LdapInfo,
    filter: String,
) -> Result<GroupRequestFilter, String> {
    let ldap_filter: LdapFilter =
        filter::parse_ldap_filter_str(filter.as_str()).map_err(|e| e.to_string())?;
    let pub_schema = PublicSchema::from(schema);
    convert_group_filter(ldap_info, &ldap_filter, &pub_schema).map_err(|e| e.to_string())
}
