use std::{collections::BTreeMap, path::PathBuf};

use crate::api::{backend::BackendAPI, permissions::Permissions};
use lldap_domain_handlers::{
    handler::RequestContext,
    requests::{ListGroupsRequest, ListUsersRequest},
};
use serde::{Deserialize, Serialize};

pub enum PluginSource {
    ScriptFile(PathBuf),
    ScriptSource(String),
}

impl PluginSource {
    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        match path.try_exists() {
            Ok(true) => {
                if PluginSource::is_lua(&path) {
                    Ok(PluginSource::ScriptFile(path))
                } else {
                    Err("Unrecognized file type".to_string())
                }
            }
            Ok(false) => Err("file does not exist".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn is_lua(path: &PathBuf) -> bool {
        path.is_file()
            && path
                .extension()
                .filter(|s| s.eq_ignore_ascii_case("lua"))
                .is_some()
    }
}

impl std::fmt::Display for PluginSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginSource::ScriptFile(p) => {
                write!(
                    f,
                    "{}",
                    p.file_name().map(|f| f.to_str()).flatten().unwrap_or("n/a")
                )
            }
            PluginSource::ScriptSource(s) => {
                write!(f, "<script: '{} [...]'>", &s.as_str()[0..30])
            }
        }
    }
}

pub struct PluginConfig {
    pub plugin_source: PluginSource,
    pub priority: u8,
    pub kvscope: Option<String>,
    pub permissions: Permissions,
    pub configuration: BTreeMap<String, String>,
}

impl PluginConfig {
    pub fn from_file(
        path: PathBuf,
        kv_scope: Option<String>,
        permissions: Permissions,
        configuration: BTreeMap<String, String>,
        priority: u8,
    ) -> Result<Self, String> {
        Ok(PluginConfig {
            plugin_source: PluginSource::from_path(path)?,
            priority,
            kvscope: kv_scope,
            permissions,
            configuration,
        })
    }
}

#[derive(Clone)]
pub struct PluginContext<A: BackendAPI + 'static> {
    pub api: &'static A,
    pub request_context: RequestContext,
}

impl<A: BackendAPI> PluginContext<A> {
    pub fn new(api: &'static A, request_context: RequestContext) -> Self {
        PluginContext {
            api,
            request_context,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum QueryFilter {
    #[serde(rename = "ldapQuery")]
    LdapFilter(String),
    #[serde(rename = "userQuery")]
    UserFilter(ListUsersRequest),
    #[serde(rename = "groupQuery")]
    GroupFilter(ListGroupsRequest),
}
