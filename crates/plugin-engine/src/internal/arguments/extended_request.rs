use ldap3_proto::proto::{LdapExtendedRequest, LdapOp};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtendedRequestArguments {
    #[serde(rename = "extended_result")]
    pub extended_result: Vec<LdapOp>,
    #[serde(rename = "extended_request")]
    pub extended_request: LdapExtendedRequest,
}

impl IntoLua for ExtendedRequestArguments {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for ExtendedRequestArguments {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
