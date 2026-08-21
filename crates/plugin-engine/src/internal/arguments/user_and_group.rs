use lldap_domain_handlers::requests::{AddUserToGroupRequest, RemoveUserFromGroupRequest};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use lldap_domain::types::{GroupId, UserId};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct UserAndGroupArguments {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "group_id")]
    pub group_id: i32,
}

impl UserAndGroupArguments {
    pub fn new(user_id: UserId, group_id: GroupId) -> Self {
        UserAndGroupArguments {
            user_id: user_id.into_string(),
            group_id: group_id.0,
        }
    }
}

impl From<AddUserToGroupRequest> for UserAndGroupArguments {
    fn from(args: AddUserToGroupRequest) -> Self {
        UserAndGroupArguments::new(args.user_id, args.group_id)
    }
}

impl From<UserAndGroupArguments> for AddUserToGroupRequest {
    fn from(value: UserAndGroupArguments) -> Self {
        AddUserToGroupRequest {
            user_id: UserId::from(&value.user_id),
            group_id: GroupId(value.group_id),
        }
    }
}
impl From<UserAndGroupArguments> for RemoveUserFromGroupRequest {
    fn from(value: UserAndGroupArguments) -> Self {
        RemoveUserFromGroupRequest {
            user_id: UserId::from(&value.user_id),
            group_id: GroupId(value.group_id),
        }
    }
}

impl IntoLua for UserAndGroupArguments {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}
impl FromLua for UserAndGroupArguments {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
