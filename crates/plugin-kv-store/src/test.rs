#[cfg(test)]
mod tests {

    use crate::{migration::create_plugin_kv_table, store::PluginKeyValueStore};
    use lldap_key_value_store::api::store::{KeyValueStore, Scope};
    use sea_orm::{Database, DatabaseConnection, DbErr, TransactionTrait};

    async fn prepare_dbconn() -> Result<DatabaseConnection, DbErr> {
        let database_url: String = "sqlite::memory:".to_string();
        let sql_pool = {
            let mut sql_opt = sea_orm::ConnectOptions::new(database_url);
            sql_opt
                .max_connections(1)
                .sqlx_logging(true)
                .sqlx_logging_level(log::LevelFilter::Debug);
            Database::connect(sql_opt).await?
        };
        Ok(sql_pool)
    }

    async fn load_fixture() -> DatabaseConnection {
        // Prepare the database connection
        let sql_conn = prepare_dbconn().await.unwrap();
        // Create the expected schema
        sql_conn
            .transaction(|transaction| {
                Box::pin(async move { create_plugin_kv_table(transaction).await })
            })
            .await
            .unwrap();
        sql_conn
    }

    #[tokio::test]
    async fn test_insert_and_fetch_string() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<String>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Exercise: Insert a value
        let store_res = store
            .store(scope.clone(), test_key.clone(), "value-test".to_string())
            .await;
        assert!(store_res.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res = store.fetch(scope, test_key).await;
        // Verify: ensure the expected value was stored
        assert_eq!(fetch_res, Ok(Some("value-test".to_string())));
    }

    #[tokio::test]
    async fn test_insert_and_override_string() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<String>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Exercise: Insert a value
        let store_res = store
            .store(scope.clone(), test_key.clone(), "value-test".to_string())
            .await;
        assert!(store_res.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res = store.fetch(scope.clone(), test_key.clone()).await;
        // Verify: ensure the expected value was stored
        assert_eq!(fetch_res, Ok(Some("value-test".to_string())));
        // Exercise: Insert a new value, overriding the former
        let store_res_2 = store
            .store(scope.clone(), test_key.clone(), "value-test-2".to_string())
            .await;
        assert!(store_res_2.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res_2 = store.fetch::<String>(scope, test_key).await;
        // Verify: ensure the expected value was stored
        assert_eq!(fetch_res_2, Ok(Some("value-test-2".to_string())));
    }

    #[tokio::test]
    async fn test_insert_and_fetch_i64() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<i64>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Exercise: Insert a value
        let store_res = store.store(scope.clone(), test_key.clone(), 42i64).await;
        assert!(store_res.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res = store.fetch(scope, test_key).await;
        // Verify: ensure the expected value was stored
        assert_eq!(fetch_res, Ok(Some(42i64)));
    }

    #[tokio::test]
    async fn test_fetch_and_increment() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<i64>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Setup: Insert a value
        let store_res = store.store(scope.clone(), test_key.clone(), 42i64).await;
        assert!(store_res.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res = store
            .fetch_and_increment(scope.clone(), test_key.clone(), 99)
            .await;
        // Verify: ensure the original value was returned
        assert_eq!(fetch_res, Ok(42i64));
        // Exercise: Fetch the current stored value
        let inc_res = store.fetch::<i64>(scope, test_key).await;
        // Verify: ensure the value was incremented on previous call
        assert_eq!(inc_res, Ok(Some(43i64)));
    }

    #[tokio::test]
    async fn test_fetch_and_increment_default_value() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<i64>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Exercise: get default value from fetch_and_increment
        let fetch_res = store.fetch_and_increment(scope, test_key, 99).await;
        // Verify: ensure the default value was returned
        assert_eq!(fetch_res, Ok(99i64));
    }

    #[tokio::test]
    async fn test_scope_separation() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope_a = Scope("scope-a".to_string());
        let scope_b = Scope("scope-b".to_string());
        // Setup: Prepare two stores with separate scopes
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res_a = store
            .fetch::<String>(scope_a.clone(), test_key.clone())
            .await;
        assert_eq!(pre_res_a, Ok(None));
        let pre_res_b = store
            .fetch::<String>(scope_b.clone(), test_key.clone())
            .await;
        assert_eq!(pre_res_b, Ok(None));
        // Exercise: Insert a value into store_a only
        let store_res = store
            .store(scope_a.clone(), test_key.clone(), "value-test".to_string())
            .await;
        assert!(store_res.is_ok());
        // Exercise: Fetch value stored with key
        let fetch_res_a = store.fetch::<String>(scope_a, test_key.clone()).await;
        let fetch_res_b = store.fetch::<String>(scope_b, test_key).await;
        // Verify: ensure the expected value was stored in storeA
        assert_eq!(fetch_res_a, Ok(Some("value-test".to_string())));
        // Verify: still no stored value in storeB
        assert_eq!(fetch_res_b, Ok(None));
    }

    #[tokio::test]
    async fn test_remove_entry() {
        // Setup: Load a database and get a connection
        let db_conn = load_fixture().await;
        let scope = Scope("test".to_string());
        let store = PluginKeyValueStore::new(db_conn.clone());
        let test_key: String = "test-key".to_string();
        // Verify: ensure nothing is stored under the key beforehand
        let pre_res = store.fetch::<String>(scope.clone(), test_key.clone()).await;
        assert_eq!(pre_res, Ok(None));
        // Exercise: Insert a value
        let store_res = store
            .store(scope.clone(), test_key.clone(), "value-test".to_string())
            .await;
        assert!(store_res.is_ok());
        // Exercise: Fetch the stored value
        let fetch_res = store.fetch::<String>(scope.clone(), test_key.clone()).await;
        // Verify: ensure something is stored
        assert!(fetch_res.unwrap().is_some());
        // Exercise: delete the entry
        let del_res = store.remove(scope.clone(), test_key.clone()).await;
        assert!(del_res.is_ok());
        // Verify: nothing stored anymore
        let fetch_res_post = store.fetch::<String>(scope, test_key).await;
        assert_eq!(fetch_res_post, Ok(None));
    }
}
