use ldap3_proto::proto::LdapSearchRequest;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};

use crate::{
    api::arguments::ldap_search_result::SearchResult,
    internal::types::ldap_search_result::LuaSearchResult,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResultArguments {
    #[serde(rename = "search_result")]
    pub search_result: LuaSearchResult,
    #[serde(rename = "search_request")]
    pub search_request: LdapSearchRequest,
}

impl SearchResultArguments {
    pub fn new(result: SearchResult, request: LdapSearchRequest) -> Self {
        Self {
            search_result: LuaSearchResult::from(result),
            search_request: request,
        }
    }
}

impl IntoLua for SearchResultArguments {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for SearchResultArguments {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
