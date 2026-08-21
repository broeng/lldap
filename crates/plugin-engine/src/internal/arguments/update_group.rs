use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::{Attribute, AttributeName, GroupId, GroupName};
use lldap_domain_handlers::requests::UpdateGroupRequest;

#[derive(Clone, Serialize, Deserialize, Default, Debug, tealr::ToTypename)]
pub struct UpdateGroupArguments {
    #[serde(rename = "group_id")]
    pub group_id: i32,
    #[serde(
        rename = "display_name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<String>,
    #[serde(rename = "delete_attributes")]
    pub delete_attributes: Vec<String>,
    #[serde(
        rename = "insert_attributes",
        with = "crate::internal::types::attribute_map"
    )]
    pub insert_attributes: Vec<Attribute>,
}

impl UpdateGroupArguments {
    pub fn from(request: UpdateGroupRequest) -> Self {
        UpdateGroupArguments {
            group_id: request.group_id.0,
            display_name: request.display_name.map(GroupName::into_string),
            delete_attributes: request
                .delete_attributes
                .into_iter()
                .map(AttributeName::into_string)
                .collect(),
            insert_attributes: request.insert_attributes,
        }
    }

    pub fn into_request(self) -> UpdateGroupRequest {
        UpdateGroupRequest {
            group_id: GroupId(self.group_id),
            display_name: self.display_name.map(GroupName::from),
            delete_attributes: self
                .delete_attributes
                .into_iter()
                .map(AttributeName::from)
                .collect(),
            insert_attributes: self.insert_attributes,
        }
    }
}

impl IntoLua for UpdateGroupArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for UpdateGroupArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
