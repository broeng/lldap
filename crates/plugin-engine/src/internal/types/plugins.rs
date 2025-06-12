use lldap_key_value_store::api::store::Scope;
use mlua::{Function, Lua};
use std::{collections::BTreeMap, sync::Arc};
use tracing::warn;

use crate::api::permissions::Permissions;

macro_rules! declare_plugin_registry {
    ($($is:ident),+) => {
        #[derive(Clone)]
        pub struct PluginRegistry {
            pub lua: &'static Lua,
            pub plugins: Vec<Arc<Plugin>>,
            pub init: Vec<Callback>,
            $(pub $is: Vec<Callback>),*
        }

        impl PluginRegistry {
            pub fn new(lua: &'static mlua::Lua) -> Self {
                Self {
                    lua,
                    plugins: Vec::new(),
                    init: Vec::new(),
                    $($is : Vec::new()),*
                }
            }
            pub fn sort_handlers(&mut self) -> () {
                let sort_callbacks = |callback: &Callback| callback.priority;
                self.init.sort_by_key(sort_callbacks);
                $(self.$is.sort_by_key(sort_callbacks));*
            }
            pub fn register_handler(&mut self, plugin: &Arc<Plugin>, event: &str, table: mlua::Table) -> mlua::Result<()> {
                match event {
                    $(stringify!($is) => {
                        self.$is.push(Callback {
                            plugin: Arc::clone(plugin),
                            priority: table.get("priority").unwrap_or(50),
                            callback: table.get("impl")?
                        });
                    })*
                    _ => {
                        warn!("[{}] Unrecognized event: {}", &plugin.name, event);
                    }
                }
                Ok(())
            }
        }
    };
}

declare_plugin_registry! {
    on_create_user,
    on_created_user,
    on_update_user,
    on_updated_user,
    on_delete_user,
    on_deleted_user,
    on_create_group,
    on_created_group,
    on_update_group,
    on_updated_group,
    on_delete_group,
    on_deleted_group,
    on_add_user_to_group,
    on_added_user_to_group,
    on_remove_user_from_group,
    on_removed_user_from_group,
    on_get_user_details,
    on_get_user_details_result,
    on_get_group_details,
    on_get_group_details_result,
    on_get_user_groups,
    on_get_user_groups_result,
    on_list_users,
    on_list_users_result,
    on_list_groups,
    on_list_groups_result,
    on_get_schema,
    on_add_user_attribute,
    on_added_user_attribute,
    on_add_group_attribute,
    on_added_group_attribute,
    on_delete_user_attribute,
    on_deleted_user_attribute,
    on_delete_group_attribute,
    on_deleted_group_attribute,
    on_add_user_object_class,
    on_added_user_object_class,
    on_add_group_object_class,
    on_added_group_object_class,
    on_delete_user_object_class,
    on_deleted_user_object_class,
    on_delete_group_object_class,
    on_deleted_group_object_class,
    on_ldap_password_update,
    on_ldap_search_result,
    on_ldap_root_dse,
    on_ldap_bind,
    on_ldap_unbind,
    on_ldap_modify,
    on_ldap_extended_request
}

#[derive(Clone)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub author: String,
    pub permissions: Permissions,
    pub configuration: BTreeMap<String, String>,
    pub kvstore_scope: Scope,
}

#[derive(Clone)]
pub struct Callback {
    pub plugin: Arc<Plugin>,
    pub priority: u8,
    pub callback: Function,
}
