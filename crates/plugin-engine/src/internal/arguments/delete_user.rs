use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::UserId;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeleteUserArguments {
    #[serde(rename = "user_id")]
    pub user_id: String,
}

impl DeleteUserArguments {
    pub fn from(user_id: UserId) -> Self {
        DeleteUserArguments {
            user_id: user_id.into_string(),
        }
    }
    pub fn into_user_id(self) -> UserId {
        UserId::from(self.user_id)
    }
}

impl IntoLua for DeleteUserArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for DeleteUserArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
