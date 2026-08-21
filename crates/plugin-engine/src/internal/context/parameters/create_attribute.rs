use mlua::{FromLua, IntoLua, LuaSerdeExt, Result as LuaResult};
use serde::{Deserialize, Serialize};

use lldap_domain::types::AttributeType;
use lldap_domain_handlers::requests::CreateAttributeRequest;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttributeParams {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "attribute_type")]
    pub attribute_type: AttributeType,
    #[serde(rename = "is_list")]
    pub is_list: bool,
    #[serde(rename = "is_visible")]
    pub is_visible: bool,
    #[serde(rename = "is_editable")]
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

impl From<CreateAttributeParams> for CreateAttributeRequest {
    fn from(value: CreateAttributeParams) -> Self {
        CreateAttributeRequest {
            name: value.name.into(),
            attribute_type: value.attribute_type,
            is_list: value.is_list,
            is_visible: value.is_visible,
            is_editable: value.is_editable,
        }
    }
}
