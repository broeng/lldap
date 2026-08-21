use ldap3_proto::proto::{LdapModifyRequest, LdapOp};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModifyRequestArguments {
    #[serde(rename = "modify_result")]
    pub modify_result: Vec<LdapOp>,
    #[serde(rename = "modify_request")]
    pub modify_request: LdapModifyRequest,
}

impl IntoLua for ModifyRequestArguments {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for ModifyRequestArguments {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
