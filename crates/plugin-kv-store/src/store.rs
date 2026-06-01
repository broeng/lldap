use async_trait::async_trait;
use lldap_domain::types::Serialized;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, TransactionTrait, sea_query};
use serde::{Serialize, de::DeserializeOwned};

use lldap_domain_model::model;
use lldap_key_value_store::api::{
    error::KeyValueError,
    store::{KeyValueStore, Scope},
};

use crate::model::PluginKeyValues;

#[derive(Clone, Debug)]
pub struct PluginKeyValueStore {
    pub(crate) sql_pool: DatabaseConnection,
}

impl PluginKeyValueStore {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { sql_pool: conn }
    }
}

#[async_trait]
impl KeyValueStore for PluginKeyValueStore {
    async fn store<V: Serialize + Send>(
        &self,
        scope: Scope,
        key: String,
        value: V,
    ) -> Result<(), KeyValueError> {
        let serialized_value = Serialized::from(&value);
        // prepare new entry
        let entry = model::plugin_key_values::ActiveModel {
            scope: ActiveValue::Set(scope.0.clone()),
            key: ActiveValue::Set(key.clone()),
            value: ActiveValue::Set(serialized_value),
            ..Default::default()
        };
        // Effectively perform an UPSERT
        model::PluginKeyValues::insert(entry)
            .on_conflict(
                sea_query::OnConflict::columns([
                    PluginKeyValues::PluginKeyScope,
                    PluginKeyValues::PluginKey,
                ])
                .update_column(PluginKeyValues::PluginKeyValue)
                .to_owned(),
            )
            .exec(&self.sql_pool)
            .await?;
        Ok(())
    }

    async fn fetch<T: DeserializeOwned>(
        &self,
        scope: Scope,
        key: String,
    ) -> Result<Option<T>, KeyValueError> {
        let existing = model::PluginKeyValues::find_by_id((scope.0.clone(), key.clone()))
            .one(&self.sql_pool)
            .await?;
        match existing {
            Some(model) => match model.value.convert_to() {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(KeyValueError::DecodingError(e.to_string())),
            },
            None => Ok(None),
        }
    }

    async fn fetch_and_increment(
        &self,
        scope: Scope,
        key: String,
        default_value: i64,
    ) -> Result<i64, KeyValueError> {
        self.sql_pool
            .transaction::<_, i64, KeyValueError>(|transaction| {
                Box::pin(async move {
                    // determine if we already have a value stored for this key
                    let existing =
                        model::PluginKeyValues::find_by_id((scope.0.clone(), key.clone()))
                            .one(transaction)
                            .await?;
                    // prepare result value
                    let result_val: i64 = match existing.clone() {
                        Some(model) => match model.value.convert_to::<i64>() {
                            Ok(v) => v,
                            Err(e) => return Err(KeyValueError::DecodingError(e.to_string())),
                        },
                        None => default_value,
                    };
                    // prepare next entry
                    let next_val: i64 = result_val + 1;
                    let entry = model::plugin_key_values::ActiveModel {
                        scope: ActiveValue::Set(scope.0.clone()),
                        key: ActiveValue::Set(key),
                        value: ActiveValue::Set(Serialized::from(&next_val)),
                        ..Default::default()
                    };
                    // Effectively perform an UPSERT
                    model::PluginKeyValues::insert(entry)
                        .on_conflict(
                            sea_query::OnConflict::columns([
                                PluginKeyValues::PluginKeyScope,
                                PluginKeyValues::PluginKey,
                            ])
                            .update_column(PluginKeyValues::PluginKeyValue)
                            .to_owned(),
                        )
                        .exec(transaction)
                        .await?;
                    Ok(result_val)
                })
            })
            .await
            .map_err(|e| e.into())
    }

    async fn remove(&self, scope: Scope, key: String) -> Result<(), KeyValueError> {
        model::PluginKeyValues::delete_by_id((scope.0.clone(), key.clone()))
            .exec(&self.sql_pool)
            .await?;
        Ok(())
    }
}
