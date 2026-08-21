//! Each plugin has its own Lua state. These tests make sure that a plugin
//! cannot change the Lua state of a different plugin.

use std::collections::BTreeMap;

use lldap_domain_handlers::handler::{RequestContext, UserBackendHandler};
use lldap_key_value_store::api::store::{KeyValueStore, Scope};
use lldap_test_utils::MockTestBackendHandler;
use pretty_assertions::assert_eq;

use crate::tests::utils::{create_user_request, new_handler, new_memory_store, plugin_config};

#[tokio::test]
async fn test_plugins_cannot_share_global_values() {
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![
            plugin_config("global_writer.lua", BTreeMap::new()),
            plugin_config("global_reader.lua", BTreeMap::new()),
        ],
    );

    handler
        .create_user(&RequestContext::empty(), create_user_request("bob"))
        .await
        .unwrap();

    // If the writer did not set the global, "nil" below proves nothing.
    let wrote: Option<String> = kvstore
        .fetch(
            Scope("global_writer".to_string()),
            "wrote_global".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(
        wrote.as_deref(),
        Some("true"),
        "the writer plugin did not run, or was not allowed to assign a global"
    );

    let observed: Option<String> = kvstore
        .fetch(
            Scope("global_reader".to_string()),
            "observed_global".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(
        observed.as_deref(),
        Some("nil"),
        "a plugin read the global of a different plugin"
    );
}

// The test does not require that the writes fail. Each plugin has its own Lua
// state, thus a write can succeed without an effect on other plugins.
#[tokio::test]
async fn test_plugins_cannot_overwrite_shared_libraries() {
    let kvstore = new_memory_store().await;
    let mut backend = MockTestBackendHandler::new();
    backend
        .expect_create_user()
        .times(1)
        .returning(|_, _| Ok(()));

    let handler = new_handler(
        backend,
        kvstore.clone(),
        vec![
            plugin_config("lib_tamperer.lua", BTreeMap::new()),
            plugin_config("lib_victim.lua", BTreeMap::new()),
        ],
    );

    handler
        .create_user(&RequestContext::empty(), create_user_request("bob"))
        .await
        .unwrap();

    // If the tamperer did not run, the victim results below prove nothing.
    let ran: Option<String> = kvstore
        .fetch(Scope("lib_tamperer".to_string()), "ran".to_string())
        .await
        .unwrap();
    assert_eq!(ran.as_deref(), Some("yes"), "the tamperer plugin never ran");

    let victim = Scope("lib_victim".to_string());
    let stdlib_result: Option<String> = kvstore
        .fetch(victim.clone(), "stdlib_result".to_string())
        .await
        .unwrap();
    let log_usable: Option<String> = kvstore
        .fetch(victim, "lldap_log_usable".to_string())
        .await
        .unwrap();

    // Examine both libraries in one assert, so that a failure shows both
    // results.
    let tamperer = Scope("lib_tamperer".to_string());
    let stdlib_write: Option<String> = kvstore
        .fetch(tamperer.clone(), "stdlib_write_allowed".to_string())
        .await
        .unwrap();
    let lldap_write: Option<String> = kvstore
        .fetch(tamperer, "lldap_write_allowed".to_string())
        .await
        .unwrap();
    assert_eq!(
        (stdlib_result.as_deref(), log_usable.as_deref()),
        (Some("OK"), Some("true")),
        "a plugin broke shared libraries for every other plugin \
         (writes permitted: stdlib={stdlib_write:?}, lldap={lldap_write:?})"
    );
}
