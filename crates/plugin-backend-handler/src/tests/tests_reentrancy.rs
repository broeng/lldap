use std::collections::BTreeMap;

use lldap_domain_handlers::handler::{
    ReadSchemaBackendHandler, RequestContext, UserBackendHandler,
};
use lldap_key_value_store::api::store::{KeyValueStore, Scope};
use lldap_plugin_engine::api::MAX_PLUGIN_CALL_DEPTH;
use lldap_test_utils::MockTestBackendHandler;
use pretty_assertions::assert_eq;

use crate::tests::utils::{
    create_user_request, expect_get_schema_from_any_context, new_handler, new_memory_store,
    plugin_config,
};

#[tokio::test]
async fn test_plugin_does_not_re_enter_its_own_callback() {
    let context = RequestContext::empty();
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    expect_get_schema_from_any_context(&mut backend);

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![plugin_config("schema_self_recursion.lua", BTreeMap::new())],
    );

    handler.get_schema(&context).await.unwrap();

    let invocations: Option<i64> = kvstore
        .fetch(
            Scope("schema_self_recursion".to_string()),
            "invocations".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(
        invocations,
        Some(1),
        "a plugin's own callback was re-entered by the call it made itself"
    );
}

// Only the plugin that makes the call is skipped. A chain that goes back to a
// plugin through a different plugin must still get to it.
//
// Plugin A lists users with an LDAP filter. To parse the filter, the backend
// reads the schema. A started this schema read, thus A does not get it.
// Plugin B gets the user listing and reads the schema. A gets this schema
// read, because B started it. Thus, the on_get_schema callback of A runs one
// time.
#[tokio::test]
async fn test_plugin_cycle_reaches_first_plugin_only_once() {
    let context = RequestContext::empty();
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    expect_get_schema_from_any_context(&mut backend);
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));
    backend
        .expect_list_users()
        .times(1)
        .returning(|_, _| Ok(Vec::new()));

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![
            plugin_config("cycle_a.lua", BTreeMap::new()),
            plugin_config("cycle_b.lua", BTreeMap::new()),
        ],
    );

    handler
        .create_user(&context, create_user_request("bob"))
        .await
        .unwrap();

    // If the full chain did not run, the count below proves nothing.
    let plugin_a = Scope("cycle_a".to_string());
    let list_users_ok: Option<String> = kvstore
        .fetch(plugin_a.clone(), "list_users_ok".to_string())
        .await
        .unwrap();
    assert_eq!(
        list_users_ok.as_deref(),
        Some("true"),
        "plugin A could not list users, so the chain never started"
    );
    let reached_b: Option<i64> = kvstore
        .fetch(Scope("cycle_b".to_string()), "invocations".to_string())
        .await
        .unwrap();
    assert_eq!(
        reached_b,
        Some(1),
        "plugin B was never reached by plugin A's call"
    );

    let reached_a: Option<i64> = kvstore
        .fetch(plugin_a, "get_schema_invocations".to_string())
        .await
        .unwrap();
    assert_eq!(
        reached_a,
        Some(1),
        "plugin A's on_get_schema should run once, for the call arriving from plugin B"
    );
}

// Each plugin reads the schema in its on_get_schema callback, and the other
// plugin gets that event. Thus, only the depth limit can stop the recursion.
//
// Each plugin runs one time at each depth level, thus MAX_PLUGIN_CALL_DEPTH
// times in total.
#[tokio::test]
async fn test_mutual_plugin_recursion_is_cut_short() {
    let context = RequestContext::empty();
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    expect_get_schema_from_any_context(&mut backend);

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![
            plugin_config("mutual_recursion_a.lua", BTreeMap::new()),
            plugin_config("mutual_recursion_b.lua", BTreeMap::new()),
        ],
    );

    handler
        .get_schema(&context)
        .await
        .expect("the engine should cut the recursion short, not fail the request");

    let plugin_a: Option<i64> = kvstore
        .fetch(
            Scope("mutual_recursion_a".to_string()),
            "invocations".to_string(),
        )
        .await
        .unwrap();
    let plugin_b: Option<i64> = kvstore
        .fetch(
            Scope("mutual_recursion_b".to_string()),
            "invocations".to_string(),
        )
        .await
        .unwrap();
    let expected = Some(MAX_PLUGIN_CALL_DEPTH as i64);
    assert_eq!(
        (plugin_a, plugin_b),
        (expected, expected),
        "recursion between two plugins was not bounded at the engine's depth limit"
    );
}
