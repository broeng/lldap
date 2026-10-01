use async_trait::async_trait;
use lldap_domain::{
    schema::{AttributeList, AttributeSchema, Schema},
    types::{
        AttributeName, AttributeType, Group, GroupDetails, GroupId, LdapObjectClass, User,
        UserAndGroups, UserId,
    },
};
use lldap_domain_handlers::{
    handler::*,
    requests::{
        AddUserToGroupRequest, CreateAttributeRequest, CreateGroupRequest, CreateUserRequest,
        ListGroupsRequest, ListUsersRequest, RemoveUserFromGroupRequest, UpdateGroupRequest,
        UpdateUserRequest,
    },
};
use lldap_domain_model::error::Result;
use lldap_opaque_handler::{OpaqueHandler, login, registration};
use std::collections::HashSet;

use mockall::predicate::eq;

mockall::mock! {
    pub TestBackendHandler{}
    impl Clone for TestBackendHandler {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl LoginHandler for TestBackendHandler {
        async fn bind(&self, context: &RequestContext, request: BindRequest) -> Result<()>;
    }
    #[async_trait]
    impl GroupListerBackendHandler for TestBackendHandler {
        async fn list_groups(&self, context: &RequestContext, filters: ListGroupsRequest) -> Result<Vec<Group>>;
    }
    #[async_trait]
    impl GroupBackendHandler for TestBackendHandler {
        async fn get_group_details(&self, context: &RequestContext, group_id: GroupId) -> Result<GroupDetails>;
        async fn update_group(&self, context: &RequestContext, request: UpdateGroupRequest) -> Result<()>;
        async fn create_group(&self, context: &RequestContext, request: CreateGroupRequest) -> Result<GroupId>;
        async fn delete_group(&self, context: &RequestContext, group_id: GroupId) -> Result<()>;
    }
    #[async_trait]
    impl UserListerBackendHandler for TestBackendHandler {
        async fn list_users(&self, context: &RequestContext, filters: ListUsersRequest) -> Result<Vec<UserAndGroups>>;
    }
    #[async_trait]
    impl UserBackendHandler for TestBackendHandler {
        async fn get_user_details(&self, context: &RequestContext, user_id: UserId) -> Result<User>;
        async fn create_user(&self, context: &RequestContext, request: CreateUserRequest) -> Result<()>;
        async fn update_user(&self, context: &RequestContext, request: UpdateUserRequest) -> Result<()>;
        async fn delete_user(&self, context: &RequestContext, user_id: UserId) -> Result<()>;
        async fn get_user_groups(&self, context: &RequestContext, user_id: UserId) -> Result<HashSet<GroupDetails>>;
        async fn add_user_to_group(&self, context: &RequestContext, request: AddUserToGroupRequest) -> Result<()>;
        async fn remove_user_from_group(&self, context: &RequestContext, request: RemoveUserFromGroupRequest) -> Result<()>;
    }
    #[async_trait]
    impl ReadSchemaBackendHandler for TestBackendHandler {
        async fn get_schema(&self, context: &RequestContext) -> Result<Schema>;
    }
    #[async_trait]
    impl SchemaBackendHandler for TestBackendHandler {
        async fn add_user_attribute(&self, context: &RequestContext, request: CreateAttributeRequest) -> Result<()>;
        async fn add_group_attribute(&self, context: &RequestContext, request: CreateAttributeRequest) -> Result<()>;
        async fn delete_user_attribute(&self, context: &RequestContext, name: AttributeName) -> Result<()>;
        async fn delete_group_attribute(&self, context: &RequestContext, name: AttributeName) -> Result<()>;
        async fn add_user_object_class(&self, context: &RequestContext, request: LdapObjectClass) -> Result<()>;
        async fn add_group_object_class(&self, context: &RequestContext, request: LdapObjectClass) -> Result<()>;
        async fn delete_user_object_class(&self, context: &RequestContext, name: LdapObjectClass) -> Result<()>;
        async fn delete_group_object_class(&self, context: &RequestContext, name: LdapObjectClass) -> Result<()>;
    }
    #[async_trait]
    impl BackendHandler for TestBackendHandler {}
    #[async_trait]
    impl OpaqueHandler for TestBackendHandler {
        async fn login_start(
            &self,
            request: login::ClientLoginStartRequest
        ) -> Result<login::ServerLoginStartResponse>;
        async fn login_finish(&self, request: login::ClientLoginFinishRequest) -> Result<UserId>;
        async fn registration_start(
            &self,
            request: registration::ClientRegistrationStartRequest
        ) -> Result<registration::ServerRegistrationStartResponse>;
        async fn registration_finish(
            &self,
            request: registration::ClientRegistrationFinishRequest
        ) -> Result<()>;
    }
}

pub fn setup_default_schema(mock: &mut MockTestBackendHandler, context: &RequestContext) {
    mock.expect_get_schema()
        .with(eq(context.clone()))
        .returning(|_| {
            Ok(Schema {
                user_attributes: AttributeList {
                    attributes: vec![
                        AttributeSchema {
                            name: "avatar".into(),
                            attribute_type: AttributeType::JpegPhoto,
                            is_list: false,
                            is_visible: true,
                            is_editable: true,
                            is_hardcoded: true,
                            is_readonly: false,
                        },
                        AttributeSchema {
                            name: "first_name".into(),
                            attribute_type: AttributeType::String,
                            is_list: false,
                            is_visible: true,
                            is_editable: true,
                            is_hardcoded: true,
                            is_readonly: false,
                        },
                        AttributeSchema {
                            name: "last_name".into(),
                            attribute_type: AttributeType::String,
                            is_list: false,
                            is_visible: true,
                            is_editable: true,
                            is_hardcoded: true,
                            is_readonly: false,
                        },
                    ],
                },
                group_attributes: AttributeList {
                    attributes: Vec::new(),
                },
                extra_user_object_classes: vec![LdapObjectClass::from("customUserClass")],
                extra_group_object_classes: Vec::new(),
            })
        });
}
