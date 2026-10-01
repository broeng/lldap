use std::collections::BTreeMap;

use lldap_domain::types::{GroupId, UserId};
use lldap_domain_handlers::handler::{
    PluginInvocation, RequestContext, UserBackendHandler, UserRequestFilter,
};
use lldap_key_value_store::api::store::{KeyValueStore, Scope};
use lldap_test_utils::MockTestBackendHandler;
use pretty_assertions::assert_eq;

use crate::tests::utils::{
    create_user_request, expect_get_schema_from_any_context, new_handler, new_memory_store,
    plugin_config,
};

#[tokio::test]
async fn test_plugin_created_group_is_dispatched_to_other_plugins() {
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));
    backend
        .expect_create_group()
        .times(1)
        .withf(|_, request| request.display_name.as_str() == "derived_from_bob")
        .returning(|_, _| Ok(GroupId(7)));

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![
            plugin_config("creator.lua", BTreeMap::new()),
            plugin_config("recorder.lua", BTreeMap::new()),
        ],
    );

    handler
        .create_user(&RequestContext::empty(), create_user_request("bob"))
        .await
        .unwrap();

    // The recorder plugin only listens for on_created_group. Thus, this value
    // shows that the call from the creator plugin dispatched the event.
    let recorded: Option<String> = kvstore
        .fetch(
            Scope("recorder".to_string()),
            "last_created_group".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(recorded.as_deref(), Some("derived_from_bob"));
}

#[tokio::test]
async fn test_no_group_event_without_a_plugin_creating_one() {
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![plugin_config("recorder.lua", BTreeMap::new())],
    );

    handler
        .create_user(&RequestContext::empty(), create_user_request("bob"))
        .await
        .unwrap();

    let recorded: Option<String> = kvstore
        .fetch(
            Scope("recorder".to_string()),
            "last_created_group".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(recorded, None);
}

// Calls from a plugin keep the validation results of the original request, so
// that the backend uses the permissions of the original user.
#[tokio::test]
async fn test_plugin_initiated_call_keeps_request_context() {
    let context = RequestContext::admin("admin");
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));
    let expected = context.clone();
    backend
        .expect_create_group()
        .times(1)
        .withf(move |request_context, _| {
            request_context.validation_results == expected.validation_results
                && request_context.plugin_stack == vec![PluginInvocation::new("creator")]
        })
        .returning(|_, _| Ok(GroupId(7)));

    let handler = new_handler(
        backend,
        kvstore,
        vec![plugin_config("creator.lua", BTreeMap::new())],
    );

    handler
        .create_user(&context, create_user_request("bob"))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_plugin_list_users_passes_need_groups_to_backend() {
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    expect_get_schema_from_any_context(&mut backend);
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));
    // A user query carries its own need_groups value.
    backend
        .expect_list_users()
        .times(1)
        .withf(|_, request| {
            request.filter == Some(UserRequestFilter::UserId(UserId::new("bob")))
                && !request.need_groups
        })
        .returning(|_, _| Ok(Vec::new()));
    // An LDAP query has no need_groups value, thus groups are included.
    backend
        .expect_list_users()
        .times(1)
        .withf(|_, request| {
            request.filter == Some(UserRequestFilter::UserId(UserId::new("bob")))
                && request.need_groups
        })
        .returning(|_, _| Ok(Vec::new()));

    let handler = new_handler(
        backend,
        kvstore,
        vec![plugin_config("user_lister.lua", BTreeMap::new())],
    );

    handler
        .create_user(&RequestContext::empty(), create_user_request("bob"))
        .await
        .unwrap();
}
