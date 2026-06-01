use sea_orm::{
    ConnectionTrait, DatabaseTransaction, DbErr,
    sea_query::{ColumnDef, Index, Table},
};

use crate::model::PluginKeyValues;

pub async fn create_plugin_kv_table(transaction: &DatabaseTransaction) -> Result<(), DbErr> {
    let builder = transaction.get_database_backend();
    transaction
        .execute(
            builder.build(
                Table::create()
                    .table(PluginKeyValues::Table)
                    .col(
                        ColumnDef::new(PluginKeyValues::PluginKeyScope)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PluginKeyValues::PluginKey)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PluginKeyValues::PluginKeyValue)
                            .blob()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PluginKeyValues::PluginKeyScope)
                            .col(PluginKeyValues::PluginKey),
                    ),
            ),
        )
        .await?;
    Ok(())
}
