use crate::internal::types::datetime::LuaDateTime;
use lldap_domain::types::{Attribute, Group, GroupDetails, GroupId, GroupName, UserId, Uuid};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaGroupDetails {
    #[serde(rename = "group_id")]
    pub group_id: i32,
    #[serde(rename = "display_name")]
    pub display_name: String,
    #[serde(rename = "creation_date")]
    pub creation_date: LuaDateTime,
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "attributes", with = "crate::internal::types::attribute_map")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "modified_date")]
    pub modified_date: LuaDateTime,
}

impl From<GroupDetails> for LuaGroupDetails {
    fn from(g: GroupDetails) -> Self {
        LuaGroupDetails {
            group_id: g.group_id.0,
            display_name: g.display_name.into_string(),
            creation_date: g.creation_date.into(),
            uuid: g.uuid.into_string(),
            attributes: g.attributes,
            modified_date: g.modified_date.into(),
        }
    }
}

impl From<LuaGroupDetails> for GroupDetails {
    fn from(value: LuaGroupDetails) -> Self {
        GroupDetails {
            group_id: GroupId(value.group_id),
            display_name: GroupName::from(value.display_name),
            creation_date: value.creation_date.datetime,
            uuid: Uuid::try_from(value.uuid.as_str()).unwrap(),
            attributes: value.attributes,
            modified_date: value.modified_date.datetime,
        }
    }
}

impl IntoLua for LuaGroupDetails {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaGroupDetails {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaGroup {
    #[serde(rename = "group_id")]
    pub group_id: i32,
    #[serde(rename = "display_name")]
    pub display_name: String,
    #[serde(rename = "creation_date")]
    pub creation_date: LuaDateTime,
    #[serde(rename = "uuid")]
    pub uuid: String,
    #[serde(rename = "users")]
    pub users: Vec<String>,
    #[serde(rename = "attributes", with = "crate::internal::types::attribute_map")]
    pub attributes: Vec<Attribute>,
    #[serde(rename = "modified_date")]
    pub modified_date: LuaDateTime,
}

impl From<Group> for LuaGroup {
    fn from(g: Group) -> Self {
        LuaGroup {
            group_id: g.id.0,
            display_name: g.display_name.into_string(),
            creation_date: LuaDateTime {
                datetime: g.creation_date,
            },
            uuid: g.uuid.into_string(),
            users: g.users.into_iter().map(UserId::into_string).collect(),
            attributes: g.attributes,
            modified_date: LuaDateTime {
                datetime: g.modified_date,
            },
        }
    }
}

impl From<LuaGroup> for Group {
    fn from(value: LuaGroup) -> Self {
        Group {
            id: GroupId(value.group_id),
            display_name: GroupName::from(value.display_name),
            creation_date: value.creation_date.datetime,
            uuid: Uuid::try_from(value.uuid.as_str()).unwrap(),
            users: value.users.into_iter().map(UserId::from).collect(),
            attributes: value.attributes,
            modified_date: value.modified_date.datetime,
        }
    }
}

impl IntoLua for LuaGroup {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaGroup {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuaGroupsVec {
    #[serde(rename = "groups")]
    pub groups: Vec<LuaGroup>,
}

impl From<Vec<Group>> for LuaGroupsVec {
    fn from(value: Vec<Group>) -> Self {
        LuaGroupsVec {
            groups: value.into_iter().map(LuaGroup::from).collect(),
        }
    }
}
impl From<LuaGroupsVec> for Vec<Group> {
    fn from(value: LuaGroupsVec) -> Self {
        value.groups.into_iter().map(LuaGroup::into).collect()
    }
}

impl FromLua for LuaGroupsVec {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
impl IntoLua for LuaGroupsVec {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}
