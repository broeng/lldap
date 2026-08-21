use crate::internal::types::{datetime::LuaDateTime, group::LuaGroupDetails};
use lldap_domain::types::{Attribute, Email, User, UserAndGroups, UserId, Uuid};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaUser {
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
    #[serde(rename = "creation_date")]
    pub creation_date: LuaDateTime,
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "attributes", with = "crate::internal::types::attribute_map")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "modified_date")]
    pub modified_date: LuaDateTime,
    #[serde(rename = "password_modified_date")]
    pub password_modified_date: LuaDateTime,
}

impl From<User> for LuaUser {
    fn from(user: User) -> Self {
        LuaUser {
            user_id: user.user_id.into_string(),
            email: user.email.into_string(),
            display_name: user.display_name,
            creation_date: user.creation_date.into(),
            uuid: user.uuid.into_string(),
            attributes: user.attributes,
            modified_date: user.modified_date.into(),
            password_modified_date: user.password_modified_date.into(),
        }
    }
}

impl From<LuaUser> for User {
    fn from(value: LuaUser) -> Self {
        User {
            user_id: UserId::from(value.user_id),
            email: Email::from(value.email),
            display_name: value.display_name,
            creation_date: value.creation_date.datetime,
            uuid: Uuid::try_from(value.uuid.as_str()).unwrap(),
            attributes: value.attributes,
            modified_date: value.modified_date.datetime,
            password_modified_date: value.password_modified_date.datetime,
        }
    }
}

impl IntoLua for LuaUser {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaUser {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaUserAndGroups {
    #[serde(rename = "user")]
    pub user: LuaUser,
    // The key is not set for `None`, so that plugins get `nil` for "not fetched" and an empty
    // table for "no groups". Without this, `lua.to_value` gives mlua's null value.
    #[serde(rename = "groups", default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<LuaGroupDetails>>,
}

impl IntoLua for LuaUserAndGroups {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaUserAndGroups {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaUserAndGroupsVec {
    #[serde(rename = "user_and_groups")]
    pub user_and_groups: Vec<LuaUserAndGroups>,
}

impl From<UserAndGroups> for LuaUserAndGroups {
    fn from(ug: UserAndGroups) -> Self {
        LuaUserAndGroups {
            user: ug.user.into(),
            groups: ug
                .groups
                .map(|groups| groups.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<LuaUserAndGroups> for UserAndGroups {
    fn from(value: LuaUserAndGroups) -> Self {
        UserAndGroups {
            user: value.user.into(),
            groups: value
                .groups
                .map(|groups| groups.into_iter().map(LuaGroupDetails::into).collect()),
        }
    }
}

impl From<Vec<UserAndGroups>> for LuaUserAndGroupsVec {
    fn from(v: Vec<UserAndGroups>) -> Self {
        LuaUserAndGroupsVec {
            user_and_groups: v.into_iter().map(|ug| ug.into()).collect(),
        }
    }
}
impl From<LuaUserAndGroupsVec> for Vec<UserAndGroups> {
    fn from(value: LuaUserAndGroupsVec) -> Self {
        value
            .user_and_groups
            .into_iter()
            .map(LuaUserAndGroups::into)
            .collect()
    }
}

impl FromLua for LuaUserAndGroupsVec {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
impl IntoLua for LuaUserAndGroupsVec {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}
