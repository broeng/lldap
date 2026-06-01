use crate::api::types::QueryFilter;
use lldap_domain_handlers::requests::{ListGroupsRequest, ListUsersRequest};
use mlua::{FromLua, IntoLua, LuaSerdeExt, Result as LuaResult};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct ListQueryParam {
    #[serde(rename = "filter")]
    pub filter: Option<QueryFilter>,
}

impl IntoLua for QueryFilter {
    fn into_lua(self, lua: &mlua::Lua) -> LuaResult<mlua::Value> {
        lua.to_value(&self)
    }
}
impl FromLua for QueryFilter {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl IntoLua for ListQueryParam {
    fn into_lua(self, lua: &mlua::Lua) -> LuaResult<mlua::Value> {
        lua.to_value(&self)
    }
}
impl FromLua for ListQueryParam {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}

impl From<ListUsersRequest> for ListQueryParam {
    fn from(value: ListUsersRequest) -> Self {
        ListQueryParam {
            filter: Some(QueryFilter::UserFilter(value)),
        }
    }
}
impl From<ListGroupsRequest> for ListQueryParam {
    fn from(value: ListGroupsRequest) -> Self {
        ListQueryParam {
            filter: Some(QueryFilter::GroupFilter(value)),
        }
    }
}

impl ListQueryParam {
    pub fn try_into_user_request(self) -> Result<ListUsersRequest, String> {
        match self.filter {
            Some(QueryFilter::UserFilter(uf)) => Ok(uf),
            _ => Err("Request was not a user filter".to_string()),
        }
    }
    pub fn try_into_group_request(self) -> Result<ListGroupsRequest, String> {
        match self.filter {
            Some(QueryFilter::GroupFilter(gf)) => Ok(gf),
            _ => Err("Request was not a group filter".to_string()),
        }
    }
}
