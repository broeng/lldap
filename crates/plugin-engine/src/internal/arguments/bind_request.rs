use ldap3_proto::proto::LdapBindRequest;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

use crate::api::arguments::ldap_bind_result::BindResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BindRequestArguments {
    #[serde(rename = "bind_result")]
    pub bind_result: BindResult,
    #[serde(rename = "bind_request")]
    pub bind_request: LdapBindRequest,
}

impl IntoLua for BindRequestArguments {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for BindRequestArguments {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
