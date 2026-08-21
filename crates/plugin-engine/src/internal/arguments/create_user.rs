use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::{Attribute, Email, UserId};
use lldap_domain_handlers::requests::CreateUserRequest;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CreateUserArguments {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "email")]
    pub email: String,
    #[serde(
        rename = "display_name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<String>,
    #[serde(rename = "attributes", with = "crate::internal::types::attribute_map")]
    pub attributes: Vec<Attribute>,
}

impl CreateUserArguments {
    pub fn from(request: CreateUserRequest) -> Self {
        CreateUserArguments {
            user_id: request.user_id.clone().into_string(),
            email: request.email.clone().into_string(),
            display_name: request.display_name,
            attributes: request.attributes,
        }
    }

    pub fn into_request(self) -> CreateUserRequest {
        CreateUserRequest {
            user_id: UserId::from(self.user_id),
            email: Email::from(self.email),
            display_name: self.display_name,
            attributes: self.attributes,
        }
    }
}

impl IntoLua for CreateUserArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for CreateUserArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
