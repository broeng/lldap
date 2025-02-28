use serde::{Deserialize, Serialize};

use sea_orm::DeriveIden;

#[derive(DeriveIden, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PluginKeyValues {
    Table,
    PluginKeyScope,
    PluginKey,
    PluginKeyValue,
}
