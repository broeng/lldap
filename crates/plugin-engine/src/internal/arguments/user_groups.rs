use std::collections::HashSet;

use lldap_domain::types::GroupDetails;
use mlua::{Error, FromLua, IntoLua, Value};

use crate::internal::types::group::LuaGroupDetails;

#[derive(Clone, Debug)]
pub struct UserGroupsArguments {
    user_groups: Vec<LuaGroupDetails>,
}

impl From<HashSet<GroupDetails>> for UserGroupsArguments {
    fn from(value: HashSet<GroupDetails>) -> Self {
        Self {
            user_groups: value.into_iter().map(LuaGroupDetails::from).collect(),
        }
    }
}
impl Into<HashSet<GroupDetails>> for UserGroupsArguments {
    fn into(self) -> HashSet<GroupDetails> {
        self.user_groups
            .into_iter()
            .map(LuaGroupDetails::into)
            .collect()
    }
}

impl IntoLua for UserGroupsArguments {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
        let t = lua.create_table()?;
        t.set("user_groups", self.user_groups)?;
        Ok(Value::Table(t))
    }
}
impl FromLua for UserGroupsArguments {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => Ok(UserGroupsArguments {
                user_groups: t.get("user_groups")?,
            }),
            _ => Err(Error::FromLuaConversionError {
                from: "{unknown}",
                to: "UserGroupsArguments".to_string(),
                message: Some("Lua table expected".to_string()),
            }),
        }
    }
}
