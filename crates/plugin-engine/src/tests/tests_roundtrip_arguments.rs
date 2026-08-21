// Roundtrip tests for the event argument and API parameter types that do not contain
// `ldap3_proto` types.

use std::collections::HashSet;

use lldap_domain::types::{
    Attribute, AttributeName, AttributeType, AttributeValue, Cardinality, GroupDetails, GroupId,
    GroupName, UserId, Uuid,
};
use lldap_domain_handlers::handler::UserRequestFilter;
use lldap_domain_handlers::requests::ListUsersRequest;

use crate::api::types::QueryFilter;
use crate::internal::arguments::create_group::CreateGroupArguments;
use crate::internal::arguments::create_user::CreateUserArguments;
use crate::internal::arguments::delete_group::DeleteGroupArguments;
use crate::internal::arguments::delete_user::DeleteUserArguments;
use crate::internal::arguments::update_group::UpdateGroupArguments;
use crate::internal::arguments::update_password::UpdatePasswordArguments;
use crate::internal::arguments::update_user::UpdateUserArguments;
use crate::internal::arguments::user_and_group::UserAndGroupArguments;
use crate::internal::arguments::user_groups::UserGroupsArguments;
use crate::internal::context::parameters::create_attribute::CreateAttributeParams;
use crate::internal::context::parameters::list_query::ListQueryParam;
use crate::tests::roundtrip_utils::assert_roundtrip;

fn single_attribute() -> Attribute {
    Attribute {
        name: AttributeName::new("nickname"),
        value: AttributeValue::String(Cardinality::Singleton("Bob".to_string())),
    }
}

#[tokio::test]
async fn test_roundtrip_user_and_group_arguments() {
    let original = UserAndGroupArguments::new(UserId::from("alice"), GroupId(7));

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_id, "alice")
            assert_eq(value.group_id, 7)
        "#,
    );

    assert_eq!(result.user_id, original.user_id);
    assert_eq!(result.group_id, original.group_id);
}

#[tokio::test]
async fn test_roundtrip_update_password_arguments() {
    let original = UpdatePasswordArguments {
        user_id: "alice".to_string(),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_id, "alice")
        "#,
    );

    assert_eq!(result.user_id, original.user_id);
}

#[tokio::test]
async fn test_roundtrip_user_groups_arguments() {
    let group = GroupDetails {
        group_id: GroupId(1),
        display_name: GroupName::from("Admins".to_string()),
        creation_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        uuid: Uuid::try_from("550e8400-e29b-41d4-a716-446655440010").unwrap(),
        attributes: vec![],
        modified_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    };
    let original_set: HashSet<GroupDetails> = HashSet::from([group]);
    let original = UserGroupsArguments::from(original_set.clone());

    let result = assert_roundtrip(
        original,
        r#"
            assert_eq(#value.user_groups, 1)
            assert_eq(value.user_groups[1].display_name, "Admins")
        "#,
    );

    let result_set: HashSet<GroupDetails> = result.into();
    assert_eq!(result_set, original_set);
}

#[tokio::test]
async fn test_roundtrip_update_user_arguments() {
    let original = UpdateUserArguments {
        user_id: "alice".to_string(),
        email: Some("alice@example.com".to_string()),
        display_name: Some("Alice".to_string()),
        delete_attributes: vec!["old_attr".to_string()],
        insert_attributes: vec![single_attribute()],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_id, "alice")
            assert_eq(value.email, "alice@example.com")
            assert_eq(value.display_name, "Alice")
            assert_eq(lldap.tables:eq(value.delete_attributes, {"old_attr"}), true)
            assert_eq(value.insert_attributes.nickname.string, "Bob")
        "#,
    );

    assert_eq!(result.user_id, original.user_id);
    assert_eq!(result.email, original.email);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(result.delete_attributes, original.delete_attributes);
    assert_eq!(result.insert_attributes, original.insert_attributes);
}

#[tokio::test]
async fn test_roundtrip_update_user_arguments_no_email_no_display_name() {
    let original = UpdateUserArguments {
        user_id: "bob".to_string(),
        email: None,
        display_name: None,
        delete_attributes: vec![],
        insert_attributes: vec![],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.email, nil)
            assert_eq(value.display_name, nil)
        "#,
    );

    assert_eq!(result.email, None);
    assert_eq!(result.display_name, None);
}

#[tokio::test]
async fn test_roundtrip_create_group_arguments() {
    let original = CreateGroupArguments {
        display_name: "New Group".to_string(),
        attributes: vec![single_attribute()],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.display_name, "New Group")
            assert_eq(value.attributes.nickname.string, "Bob")
        "#,
    );

    assert_eq!(result.display_name, original.display_name);
    assert_eq!(result.attributes, original.attributes);
}

#[tokio::test]
async fn test_roundtrip_delete_user_arguments() {
    let original = DeleteUserArguments::from(UserId::from("alice"));

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_id, "alice")
        "#,
    );

    assert_eq!(result.user_id, original.user_id);
}

#[tokio::test]
async fn test_roundtrip_delete_group_arguments() {
    let original = DeleteGroupArguments::from(GroupId(42));

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.group_id, 42)
        "#,
    );

    assert_eq!(result.group_id, original.group_id);
}

#[tokio::test]
async fn test_roundtrip_update_group_arguments() {
    let original = UpdateGroupArguments {
        group_id: 3,
        display_name: Some("Renamed".to_string()),
        delete_attributes: vec!["old".to_string()],
        insert_attributes: vec![single_attribute()],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.group_id, 3)
            assert_eq(value.display_name, "Renamed")
            assert_eq(lldap.tables:eq(value.delete_attributes, {"old"}), true)
            assert_eq(value.insert_attributes.nickname.string, "Bob")
        "#,
    );

    assert_eq!(result.group_id, original.group_id);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(result.delete_attributes, original.delete_attributes);
    assert_eq!(result.insert_attributes, original.insert_attributes);
}

#[tokio::test]
async fn test_roundtrip_update_group_arguments_no_display_name() {
    let original = UpdateGroupArguments {
        group_id: 5,
        display_name: None,
        delete_attributes: vec![],
        insert_attributes: vec![],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.group_id, 5)
            assert_eq(value.display_name, nil)
        "#,
    );

    assert_eq!(result.display_name, None);
}

#[tokio::test]
async fn test_roundtrip_create_user_arguments() {
    let original = CreateUserArguments {
        user_id: "alice".to_string(),
        email: "alice@example.com".to_string(),
        display_name: Some("Alice".to_string()),
        attributes: vec![single_attribute()],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_id, "alice")
            assert_eq(value.email, "alice@example.com")
            assert_eq(value.display_name, "Alice")
            assert_eq(value.attributes.nickname.string, "Bob")
        "#,
    );

    assert_eq!(result.user_id, original.user_id);
    assert_eq!(result.email, original.email);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(result.attributes, original.attributes);
}

#[tokio::test]
async fn test_roundtrip_create_attribute_params() {
    let original = CreateAttributeParams {
        name: "nickname".to_string(),
        attribute_type: AttributeType::String,
        is_list: false,
        is_visible: true,
        is_editable: true,
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.name, "nickname")
            assert_eq(value.attribute_type, "String")
            assert_eq(value.is_list, false)
            assert_eq(value.is_visible, true)
            assert_eq(value.is_editable, true)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_query_filter_user() {
    let original = QueryFilter::UserFilter(ListUsersRequest {
        filter: Some(UserRequestFilter::UserId(UserId::from("alice"))),
        need_groups: true,
    });

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.userQuery.filter.userId, "alice")
            assert_eq(value.userQuery.need_groups, true)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_query_filter_ldap() {
    let original = QueryFilter::LdapFilter("(cn=admin)".to_string());

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.ldapQuery, "(cn=admin)")
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_list_query_param() {
    let original = ListQueryParam {
        filter: Some(QueryFilter::UserFilter(ListUsersRequest {
            filter: Some(UserRequestFilter::UserId(UserId::from("alice"))),
            need_groups: false,
        })),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.filter.userQuery.filter.userId, "alice")
            assert_eq(value.filter.userQuery.need_groups, false)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_list_query_param_none() {
    let original = ListQueryParam { filter: None };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.filter, LUA_NULL)
        "#,
    );

    assert_eq!(result, original);
}
