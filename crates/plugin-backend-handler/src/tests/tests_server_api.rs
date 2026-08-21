//! Tests the plugin API with a real `SqlBackendHandler`. A mock accepts all
//! requests, thus it cannot find incorrect requests from the Lua bridge.

use std::collections::{BTreeMap, HashSet};

use lldap_domain::types::{AttributeName, AttributeValue, Cardinality, GroupName, UserId};
use lldap_domain_handlers::{
    handler::{
        GroupBackendHandler, GroupListerBackendHandler, ReadSchemaBackendHandler, RequestContext,
        UserBackendHandler,
    },
    requests::{CreateGroupRequest, ListGroupsRequest},
};
use lldap_key_value_store::api::store::{KeyValueStore, Scope};
use pretty_assertions::assert_eq;

use crate::tests::utils::{create_user_request, new_real_handler, plugin_config};

async fn recorded(
    kvstore: &lldap_plugin_kv_store::store::PluginKeyValueStore,
    plugin: &str,
    key: &str,
) -> String {
    let value: Option<String> = kvstore
        .fetch(Scope(plugin.to_string()), key.to_string())
        .await
        .unwrap();
    value.unwrap_or_else(|| format!("<{key} was never recorded>"))
}

// Examines all calls before the assert, so that a failure shows all failed
// calls.
async fn assert_all_calls_succeeded(
    kvstore: &lldap_plugin_kv_store::store::PluginKeyValueStore,
    plugin: &str,
    calls: &[&str],
) {
    let mut outcomes = Vec::new();
    for call in calls {
        outcomes.push((*call, recorded(kvstore, plugin, call).await));
    }
    let failed: Vec<&(&str, String)> = outcomes.iter().filter(|(_, ok)| ok != "true").collect();
    assert!(
        failed.is_empty(),
        "plugin API calls did not succeed against a real backend: {failed:?}"
    );
}

#[tokio::test]
async fn test_plugin_can_drive_the_backend_api_end_to_end() {
    let context = RequestContext::empty();
    let (handler, kvstore, _backend) =
        new_real_handler(vec![plugin_config("test_server_api.lua", BTreeMap::new())]).await;

    handler
        .create_user(&context, create_user_request("alice"))
        .await
        .unwrap();

    let calls = [
        "get_schema",
        "schema_has_user_attributes",
        "add_user_attribute",
        "create_group",
        "add_user_to_group",
        "update_user",
        "get_user_details",
        "get_user_groups",
        "get_group_details",
        "list_users",
        "list_groups",
    ];
    assert_all_calls_succeeded(&kvstore, "test_server_api", &calls).await;

    assert_eq!(
        (
            recorded(&kvstore, "test_server_api", "user_colour").await,
            recorded(&kvstore, "test_server_api", "user_email").await,
            recorded(&kvstore, "test_server_api", "user_group_count").await,
            recorded(&kvstore, "test_server_api", "group_display_name").await,
            recorded(&kvstore, "test_server_api", "listed_user_count").await,
            recorded(&kvstore, "test_server_api", "listed_user_id").await,
        ),
        (
            "blue".to_string(),
            "alice@example.com".to_string(),
            "1".to_string(),
            "colour_club".to_string(),
            "1".to_string(),
            "alice".to_string(),
        ),
        "the plugin read back something other than what it wrote"
    );
}

// Examines the storage directly, not the values that the plugin records.
#[tokio::test]
async fn test_plugin_api_calls_reach_real_storage() {
    let context = RequestContext::empty();
    let (handler, _kvstore, backend) =
        new_real_handler(vec![plugin_config("test_server_api.lua", BTreeMap::new())]).await;

    handler
        .create_user(&context, create_user_request("alice"))
        .await
        .unwrap();

    let schema = backend.get_schema(&context).await.unwrap();
    let attribute_names: HashSet<&str> = schema
        .user_attributes
        .attributes
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert!(
        attribute_names.contains("favourite_colour"),
        "the attribute the plugin added is missing from the schema: {attribute_names:?}"
    );

    let groups = backend
        .list_groups(&context, ListGroupsRequest { filter: None })
        .await
        .unwrap();
    let group_names: Vec<GroupName> = groups.iter().map(|g| g.display_name.clone()).collect();
    assert!(
        group_names.contains(&GroupName::from("colour_club")),
        "the group the plugin created is missing: {group_names:?}"
    );

    let user = backend
        .get_user_details(&context, UserId::from("alice"))
        .await
        .unwrap();
    let colour = user
        .attributes
        .iter()
        .find(|a| a.name == AttributeName::from("favourite_colour"))
        .map(|a| a.value.clone());
    assert_eq!(
        colour,
        Some(AttributeValue::String(Cardinality::Singleton(
            "blue".to_string()
        ))),
        "the attribute the plugin set did not reach storage"
    );

    let memberships = backend
        .get_user_groups(&context, UserId::from("alice"))
        .await
        .unwrap();
    let membership_names: Vec<GroupName> =
        memberships.iter().map(|g| g.display_name.clone()).collect();
    assert_eq!(
        membership_names,
        vec![GroupName::from("colour_club")],
        "the group membership the plugin added did not reach storage"
    );
}

#[tokio::test]
async fn test_plugin_can_update_and_remove_users_and_their_attributes() {
    let context = RequestContext::empty();
    let (handler, kvstore, backend) = new_real_handler(vec![plugin_config(
        "test_user_operations.lua",
        BTreeMap::new(),
    )])
    .await;

    handler
        .create_user(&context, create_user_request("alice"))
        .await
        .unwrap();

    assert_all_calls_succeeded(
        &kvstore,
        "test_user_operations",
        &[
            "create_user",
            "add_user_attribute",
            "add_user_attribute_second",
            "update_user",
            "create_group",
            "add_user_to_group",
            "update_user_delete_attributes",
            "remove_user_from_group",
            "delete_user_attribute",
            "delete_user",
        ],
    )
    .await;

    assert_eq!(
        (
            recorded(&kvstore, "test_user_operations", "note_after_update").await,
            recorded(&kvstore, "test_user_operations", "note_value_gone").await,
            recorded(&kvstore, "test_user_operations", "marker_still_set").await,
            recorded(&kvstore, "test_user_operations", "groups_before_removal").await,
            recorded(&kvstore, "test_user_operations", "groups_after_removal").await,
            recorded(&kvstore, "test_user_operations", "marker_gone_from_user").await,
            recorded(
                &kvstore,
                "test_user_operations",
                "details_after_delete_errored"
            )
            .await,
        ),
        (
            "noted".to_string(),
            "true".to_string(),
            "true".to_string(),
            "1".to_string(),
            "0".to_string(),
            "true".to_string(),
            "true".to_string(),
        ),
        "the plugin did not see its updates and removals take effect"
    );

    assert!(
        backend
            .get_user_details(&context, UserId::from("bob"))
            .await
            .is_err(),
        "the user the plugin deleted is still in storage"
    );
    let schema = backend.get_schema(&context).await.unwrap();
    let attribute_names: HashSet<&str> = schema
        .user_attributes
        .attributes
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert!(
        !attribute_names.contains("temp_marker"),
        "the attribute the plugin deleted is still in the schema: {attribute_names:?}"
    );
    assert!(
        attribute_names.contains("temp_note"),
        "dropping an attribute's value should not undeclare it: {attribute_names:?}"
    );
    let groups = backend
        .list_groups(&context, ListGroupsRequest { filter: None })
        .await
        .unwrap();
    let group_names: Vec<GroupName> = groups.iter().map(|g| g.display_name.clone()).collect();
    assert!(
        group_names.contains(&GroupName::from("removal_club")),
        "removing the membership should not have removed the group: {group_names:?}"
    );
}

#[tokio::test]
async fn test_plugin_can_update_and_remove_groups_and_their_attributes() {
    let context = RequestContext::empty();
    let (handler, kvstore, backend) = new_real_handler(vec![plugin_config(
        "test_group_operations.lua",
        BTreeMap::new(),
    )])
    .await;

    handler
        .create_group(
            &context,
            CreateGroupRequest {
                display_name: GroupName::from("trigger_group"),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_all_calls_succeeded(
        &kvstore,
        "test_group_operations",
        &[
            "create_group",
            "add_group_attribute",
            "add_group_attribute_second",
            "update_group",
            "get_group_details",
            "update_group_delete_attributes",
            "delete_group_attribute",
            "delete_group",
        ],
    )
    .await;

    assert_eq!(
        (
            recorded(&kvstore, "test_group_operations", "name_after_update").await,
            recorded(&kvstore, "test_group_operations", "colour_after_update").await,
            recorded(&kvstore, "test_group_operations", "colour_gone_from_group").await,
            recorded(
                &kvstore,
                "test_group_operations",
                "details_after_delete_errored"
            )
            .await,
        ),
        (
            "workshop_renamed".to_string(),
            "green".to_string(),
            "true".to_string(),
            "true".to_string(),
        ),
        "the plugin did not see its group changes take effect"
    );

    let groups = backend
        .list_groups(&context, ListGroupsRequest { filter: None })
        .await
        .unwrap();
    let group_names: Vec<GroupName> = groups.iter().map(|g| g.display_name.clone()).collect();
    assert_eq!(
        group_names,
        vec![GroupName::from("trigger_group")],
        "the group the plugin created should be gone, and the one that triggered it left alone"
    );

    let schema = backend.get_schema(&context).await.unwrap();
    let group_attributes: HashSet<&str> = schema
        .group_attributes
        .attributes
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    assert!(
        !group_attributes.contains("team_colour"),
        "the group attribute the plugin deleted is still in the schema: {group_attributes:?}"
    );
    assert!(
        group_attributes.contains("team_motto"),
        "dropping an attribute's value should not undeclare it: {group_attributes:?}"
    );
}

#[tokio::test]
async fn test_plugin_search_filters_agree_across_both_query_forms() {
    let context = RequestContext::empty();
    let (handler, kvstore, _backend) =
        new_real_handler(vec![plugin_config("test_search.lua", BTreeMap::new())]).await;

    handler
        .create_group(
            &context,
            CreateGroupRequest {
                display_name: GroupName::from("trigger_group"),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_all_calls_succeeded(
        &kvstore,
        "test_search",
        &[
            "create_user",
            "create_group",
            "add_user_to_group",
            "add_user_object_class",
            "add_user_object_class_extra",
            "add_group_object_class",
            "need_groups_ok",
            "delete_user_object_class",
            "delete_group_object_class",
        ],
    )
    .await;

    // (query, expected rows). There is one user, and two groups: the group that
    // starts the plugin, and the group that the plugin creates.
    let expected = [
        ("user_by_uid", "1"),
        ("user_by_missing_uid", "0"),
        ("user_by_email", "1"),
        ("user_by_member_of", "1"),
        ("user_by_missing_member_of", "0"),
        ("user_by_default_object_class", "1"),
        ("user_by_added_object_class", "1"),
        ("user_by_unknown_object_class", "0"),
        ("user_by_and", "1"),
        ("user_by_or", "1"),
        ("user_by_not", "1"),
        ("user_by_uid_prefix", "1"),
        ("group_by_name", "1"),
        ("group_by_missing_name", "0"),
        ("group_by_member", "1"),
        ("group_by_added_object_class", "2"),
        ("group_by_unknown_object_class", "0"),
        ("group_by_name_prefix", "1"),
        // After the plugin deletes the object classes.
        ("user_by_deleted_object_class", "0"),
        ("group_by_deleted_object_class", "0"),
    ];

    let mut wrong = Vec::new();
    for (query, rows) in expected {
        let via_ldap = recorded(&kvstore, "test_search", &format!("{query}_ldap")).await;
        let structured = recorded(&kvstore, "test_search", &format!("{query}_structured")).await;
        if via_ldap != rows || structured != rows {
            wrong.push(format!(
                "{query}: expected {rows}, ldap gave {via_ldap}, structured gave {structured}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "queries disagreed with each other or with the expected rows:\n  {}",
        wrong.join("\n  ")
    );

    assert_eq!(
        recorded(&kvstore, "test_search", "need_groups_count").await,
        "1",
        "a listing asking for groups did not carry the membership"
    );
}
