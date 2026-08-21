use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::Attribute;
use lldap_domain_handlers::requests::CreateGroupRequest;

#[derive(Clone, Serialize, Deserialize, Default, Debug, tealr::ToTypename)]
pub struct CreateGroupArguments {
    #[serde(rename = "display_name")]
    pub display_name: String,
    #[serde(rename = "attributes", with = "crate::internal::types::attribute_map")]
    pub attributes: Vec<Attribute>,
}

impl CreateGroupArguments {
    pub fn from(request: CreateGroupRequest) -> Self {
        CreateGroupArguments {
            display_name: request.display_name.into_string(),
            attributes: request.attributes,
        }
    }

    pub fn into_request(self) -> CreateGroupRequest {
        CreateGroupRequest {
            display_name: self.display_name.into(),
            attributes: self.attributes,
        }
    }
}

impl IntoLua for CreateGroupArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for CreateGroupArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
