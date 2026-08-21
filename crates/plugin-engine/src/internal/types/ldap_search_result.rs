use crate::{
    api::arguments::ldap_search_result::SearchResult,
    internal::types::{group::LuaGroup, user::LuaUserAndGroups},
};
use ldap3_proto::proto::LdapOp;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

// `UsersAndGroups` and `Empty` are struct variants, so that serde gives
// `{users_and_groups = {users = ..., groups = ...}}` and `{empty = {}}`. A tuple variant gives
// `{users_and_groups = {[...], [...]}}`, and a unit variant gives the string `"empty"`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LuaSearchResult {
    UsersAndGroups {
        users: Vec<LuaUserAndGroups>,
        groups: Vec<LuaGroup>,
    },
    Ldap(Vec<LdapOp>),
    Empty {},
}

impl From<SearchResult> for LuaSearchResult {
    fn from(value: SearchResult) -> Self {
        match value {
            SearchResult::UsersAndGroups(u, g) => LuaSearchResult::UsersAndGroups {
                users: u.into_iter().map(LuaUserAndGroups::from).collect(),
                groups: g.into_iter().map(LuaGroup::from).collect(),
            },
            SearchResult::Ldap(ldap_op) => LuaSearchResult::Ldap(ldap_op),
            SearchResult::Empty => LuaSearchResult::Empty {},
        }
    }
}

impl From<LuaSearchResult> for SearchResult {
    fn from(value: LuaSearchResult) -> Self {
        match value {
            LuaSearchResult::UsersAndGroups { users, groups } => SearchResult::UsersAndGroups(
                users.into_iter().map(LuaUserAndGroups::into).collect(),
                groups.into_iter().map(LuaGroup::into).collect(),
            ),
            LuaSearchResult::Ldap(ldap_op) => SearchResult::Ldap(ldap_op),
            LuaSearchResult::Empty {} => SearchResult::Empty,
        }
    }
}

impl IntoLua for LuaSearchResult {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaSearchResult {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
