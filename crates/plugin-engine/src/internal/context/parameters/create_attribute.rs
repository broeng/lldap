use mlua::{FromLua, IntoLua, LuaSerdeExt, Result as LuaResult};
use serde::{Deserialize, Serialize};

use lldap_domain::types::AttributeType;
use lldap_domain_handlers::requests::CreateAttributeRequest;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttributeParams {
    pub name: String,
    pub attribute_type: AttributeType,
    pub is_list: bool,
    pub is_visible: bool,
    pub is_editable: bool,
}

impl FromLua for CreateAttributeParams {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for CreateAttributeParams {
    fn into_lua(self, lua: &mlua::Lua) -> LuaResult<mlua::Value> {
        lua.to_value(&self)
    }
}

impl From<CreateAttributeRequest> for CreateAttributeParams {
    fn from(req: CreateAttributeRequest) -> Self {
        CreateAttributeParams {
            name: req.name.into_string(),
            attribute_type: req.attribute_type,
            is_list: req.is_list,
            is_visible: req.is_visible,
            is_editable: req.is_editable,
        }
    }
}

impl Into<CreateAttributeRequest> for CreateAttributeParams {
    fn into(self) -> CreateAttributeRequest {
        CreateAttributeRequest {
            name: self.name.into(),
            attribute_type: self.attribute_type,
            is_list: self.is_list,
            is_visible: self.is_visible,
            is_editable: self.is_editable,
        }
    }
}
