use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::GroupId;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeleteGroupArguments {
    #[serde(rename = "group_id")]
    pub group_id: i32,
}

impl DeleteGroupArguments {
    pub fn from(group_id: GroupId) -> Self {
        DeleteGroupArguments {
            group_id: group_id.0,
        }
    }
    pub fn into_group_id(self) -> GroupId {
        GroupId(self.group_id)
    }
}

impl IntoLua for DeleteGroupArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for DeleteGroupArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
