use lldap_plugin_kv_store::migration::create_plugin_kv_table;
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

pub async fn load_fixture() -> DatabaseConnection {
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
