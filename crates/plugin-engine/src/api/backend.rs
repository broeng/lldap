use std::collections::HashSet;

use async_trait::async_trait;

use lldap_domain::{
    schema::Schema,
    types::{
        AttributeName, Group, GroupDetails, GroupId, LdapObjectClass, User, UserAndGroups, UserId,
    },
};
use lldap_domain_handlers::{
    handler::RequestContext,
    requests::{
        CreateAttributeRequest, CreateGroupRequest, CreateUserRequest, UpdateGroupRequest,
        UpdateUserRequest,
    },
};

use super::types::QueryFilter;

//
// BackendAPI trait exposes functionality provided by
// the LLDAP backend, and exposed to individual plugins.
//
#[async_trait]
pub trait BackendAPI: Clone + Sync + Send {
    //
    // User Listing
    //
    async fn list_users(
        &self,
        context: &RequestContext,
        filters: Option<QueryFilter>,
    ) -> Result<Vec<UserAndGroups>, String>;
    //
    // Group Listing
    //
    async fn list_groups(
        &self,
        context: &RequestContext,
        filters: Option<QueryFilter>,
    ) -> Result<Vec<Group>, String>;
    //
    // Read Schema
    //
    async fn get_schema(&self, context: &RequestContext) -> Result<Schema, String>;
    //
    // Schema
    //
    async fn add_user_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<(), String>;
    async fn add_group_attribute(
        &self,
        context: &RequestContext,
        request: CreateAttributeRequest,
    ) -> Result<(), String>;
    async fn delete_user_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<(), String>;
    async fn delete_group_attribute(
        &self,
        context: &RequestContext,
        name: AttributeName,
    ) -> Result<(), String>;
    async fn add_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String>;
    async fn add_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String>;
    async fn delete_user_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String>;
    async fn delete_group_object_class(
        &self,
        context: &RequestContext,
        name: LdapObjectClass,
    ) -> Result<(), String>;
    //
    // Groups
    //
    async fn get_group_details(
        &self,
        context: &RequestContext,
        group_id: GroupId,
    ) -> Result<GroupDetails, String>;
    async fn update_group(
        &self,
        context: &RequestContext,
        request: UpdateGroupRequest,
    ) -> Result<(), String>;
    async fn create_group(
        &self,
        context: &RequestContext,
        request: CreateGroupRequest,
    ) -> Result<GroupId, String>;
    async fn delete_group(&self, context: &RequestContext, group_id: GroupId)
        -> Result<(), String>;
    //
    // Users
    //
    async fn get_user_details(
        &self,
        context: &RequestContext,
        user_id: UserId,
    ) -> Result<User, String>;
    async fn create_user(
        &self,
        context: &RequestContext,
        request: CreateUserRequest,
    ) -> Result<(), String>;
    async fn update_user(
        &self,
        context: &RequestContext,
        request: UpdateUserRequest,
    ) -> Result<(), String>;
    async fn delete_user(&self, context: &RequestContext, user_id: UserId) -> Result<(), String>;
    async fn add_user_to_group(
        &self,
        context: &RequestContext,
        user_id: UserId,
        group_id: GroupId,
    ) -> Result<(), String>;
    async fn remove_user_from_group(
        &self,
        context: &RequestContext,
        user_id: UserId,
        group_id: GroupId,
    ) -> Result<(), String>;
    async fn get_user_groups(
        &self,
        context: &RequestContext,
        user_id: UserId,
    ) -> Result<HashSet<GroupDetails>, String>;
}
