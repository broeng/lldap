use lldap_auth::{
    access_control::{Permission, ValidationResults},
    types::UserId,
};
use lldap_domain_handlers::handler::RequestContext;
use lldap_plugin_kv_store::store::PluginKeyValueStore;

use crate::{
    api::{
        handler::{PluginHandler, PluginHandlerEvents},
        permissions::{Permissions, SecretPermissions},
        types::{PluginConfig, PluginContext, PluginSource},
    },
    tests::{
        lua_scripts::make_init_script, mock_backend::MockTestServerBackendAPI, utils::load_fixture,
    },
};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub async fn new_memory_store() -> PluginKeyValueStore {
    let dbconn = load_fixture().await;
    PluginKeyValueStore::new(dbconn)
}

pub fn plugin_config_from_file(path: &str) -> PluginConfig {
    let path = PathBuf::from(path);
    PluginConfig::from_file(
        path,
        Some("default".to_string()),
        Permissions {
            secrets: SecretPermissions::AllowAnyHash,
        },
        BTreeMap::new(),
        100,
    )
    .unwrap()
}

pub fn plugin_config_from_str(source: String) -> PluginConfig {
    PluginConfig {
        plugin_source: PluginSource::ScriptSource(source),
        priority: 100,
        kvscope: Some("default".to_string()),
        permissions: Permissions {
            secrets: SecretPermissions::AllowAnyHash,
        },
        configuration: BTreeMap::new(),
    }
}

pub fn new_context_from(
    request_context: RequestContext,
) -> PluginContext<MockTestServerBackendAPI> {
    // this will leak in tests, in actual use there will be a single instance.
    let api: &'static MockTestServerBackendAPI =
        Box::leak(Box::new(MockTestServerBackendAPI::new()));
    PluginContext::new(api, request_context)
}

pub fn new_admin_context() -> PluginContext<MockTestServerBackendAPI> {
    new_context_from(RequestContext::new(Some(ValidationResults {
        user: UserId::from("admin"),
        permission: Permission::Admin,
    })))
}

pub async fn run_plugin_init(
    kvstore: PluginKeyValueStore,
    init_script: &str,
) -> Result<(), String> {
    let plugins = vec![plugin_config_from_str(make_init_script(init_script))];
    let context = new_admin_context();
    let plugin_handler = PluginHandler::new(plugins, kvstore).unwrap();
    plugin_handler.initialize_plugins(context).await
}
