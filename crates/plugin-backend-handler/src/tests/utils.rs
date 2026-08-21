use std::collections::BTreeMap;
use std::path::PathBuf;

use lldap_auth::opaque::server::generate_random_private_key;
use lldap_domain::schema::{AttributeList, Schema};
use lldap_domain::types::{Email, UserId};
use lldap_domain_handlers::requests::CreateUserRequest;
use lldap_ldap::LdapInfo;
use lldap_plugin_engine::api::{
    handler::PluginHandler,
    permissions::{Permissions, SecretPermissions},
    types::PluginConfig,
};
use lldap_plugin_kv_store::{migration::create_plugin_kv_table, store::PluginKeyValueStore};
use lldap_sql_backend_handler::{SqlBackendHandler, sql_tables};
use lldap_test_utils::MockTestBackendHandler;
use sea_orm::{Database, DatabaseConnection, TransactionTrait};

use crate::handler::PluginBackendHandler;

pub type MockedHandler = PluginBackendHandler<MockTestBackendHandler>;

pub async fn new_memory_store() -> PluginKeyValueStore {
    // Each connection to `sqlite::memory:` has its own database, thus the pool
    // must have only one connection.
    let mut sql_opt = sea_orm::ConnectOptions::new("sqlite::memory:".to_string());
    sql_opt.max_connections(1);
    let sql_pool: DatabaseConnection = Database::connect(sql_opt).await.unwrap();
    sql_pool
        .transaction(|transaction| {
            Box::pin(async move { create_plugin_kv_table(transaction).await })
        })
        .await
        .unwrap();
    PluginKeyValueStore::new(sql_pool)
}

// The key/value scope is not set, thus it is the plugin name. Tests use that
// name to read the recorded values.
pub fn plugin_config(file_name: &str, configuration: BTreeMap<String, String>) -> PluginConfig {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "src",
        "tests",
        "plugins",
        file_name,
    ]
    .iter()
    .collect();
    PluginConfig::from_file(
        path,
        None,
        Permissions {
            secrets: SecretPermissions::AllowAnyHash,
        },
        configuration,
        100,
    )
    .unwrap()
}

// Use this instead of `lldap_test_utils::setup_default_schema`, which
// requires one specific request context. Calls from plugins have a different
// context, because it also contains the plugin call stack.
pub fn expect_get_schema_from_any_context(backend: &mut MockTestBackendHandler) {
    backend.expect_get_schema().returning(|_| {
        Ok(Schema {
            user_attributes: AttributeList {
                attributes: Vec::new(),
            },
            group_attributes: AttributeList {
                attributes: Vec::new(),
            },
            extra_user_object_classes: Vec::new(),
            extra_group_object_classes: Vec::new(),
        })
    });
}

pub fn create_user_request(user_id: &str) -> CreateUserRequest {
    CreateUserRequest {
        user_id: UserId::from(user_id),
        email: Email::from(format!("{user_id}@example.com")),
        display_name: None,
        attributes: Vec::new(),
    }
}

pub fn ldap_info() -> &'static LdapInfo {
    Box::leak(Box::new(
        LdapInfo::new("dc=example,dc=com", Vec::new(), Vec::new()).unwrap(),
    ))
}

pub fn new_handler(
    backend: MockTestBackendHandler,
    kvstore: PluginKeyValueStore,
    plugins: Vec<PluginConfig>,
) -> MockedHandler {
    let plugin_handler = PluginHandler::new(plugins, kvstore).unwrap();
    PluginBackendHandler::new(backend, plugin_handler, ldap_info())
}
pub async fn new_real_handler(
    plugins: Vec<PluginConfig>,
) -> (PluginBackendHandler, PluginKeyValueStore, SqlBackendHandler) {
    // The lldap tables and the plugin key/value table use one pool, as in the
    // server. Each connection to `sqlite::memory:` has its own database, thus
    // the pool must have only one connection.
    let mut sql_opt = sea_orm::ConnectOptions::new("sqlite::memory:".to_string());
    sql_opt.max_connections(1);
    let sql_pool: DatabaseConnection = Database::connect(sql_opt).await.unwrap();
    // The migrations also create the plugin key/value table.
    sql_tables::init_table(&sql_pool).await.unwrap();

    let kvstore = PluginKeyValueStore::new(sql_pool.clone());
    let backend = SqlBackendHandler::new(generate_random_private_key(), sql_pool);
    let plugin_handler = PluginHandler::new(plugins, kvstore.clone()).unwrap();
    (
        PluginBackendHandler::new(backend.clone(), plugin_handler, ldap_info()),
        kvstore,
        backend,
    )
}
