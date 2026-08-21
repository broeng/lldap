use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::{Attribute, AttributeName, Email, UserId};
use lldap_domain_handlers::requests::UpdateUserRequest;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct UpdateUserArguments {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "email", default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
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

impl UpdateUserArguments {
    pub fn from(request: UpdateUserRequest) -> Self {
        UpdateUserArguments {
            user_id: request.user_id.clone().into_string(),
            email: request.email.map(Email::into_string),
            display_name: request.display_name,
            delete_attributes: request
                .delete_attributes
                .into_iter()
                .map(AttributeName::into_string)
                .collect(),
            insert_attributes: request.insert_attributes,
        }
    }

    pub fn into_request(self) -> UpdateUserRequest {
        UpdateUserRequest {
            user_id: UserId::from(self.user_id),
            email: self.email.map(Email::from),
            display_name: self.display_name,
            delete_attributes: self
                .delete_attributes
                .into_iter()
                .map(AttributeName::from)
                .collect(),
            insert_attributes: self.insert_attributes,
        }
    }
}

impl IntoLua for UpdateUserArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for UpdateUserArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
