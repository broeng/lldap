use std::collections::HashSet;

use lldap_domain::types::GroupDetails;
use mlua::{FromLua, IntoLua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

use crate::internal::types::group::LuaGroupDetails;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserGroupsArguments {
    #[serde(rename = "user_groups")]
    user_groups: Vec<LuaGroupDetails>,
}

impl From<HashSet<GroupDetails>> for UserGroupsArguments {
    fn from(value: HashSet<GroupDetails>) -> Self {
        Self {
            user_groups: value.into_iter().map(LuaGroupDetails::from).collect(),
        }
    }
}
impl From<UserGroupsArguments> for HashSet<GroupDetails> {
    fn from(value: UserGroupsArguments) -> Self {
        value
            .user_groups
            .into_iter()
            .map(LuaGroupDetails::into)
            .collect()
    }
}

impl IntoLua for UserGroupsArguments {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}
impl FromLua for UserGroupsArguments {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
