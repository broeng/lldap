use lldap_key_value_store::api::store::Scope;
use mlua::{Lua, Result as LuaResult, Table};

use crate::{
    api::types::{PluginConfig, PluginSource},
    internal::lualib::init,
    internal::types::plugins::{Callback, Plugin, PluginRegistry},
};

use std::sync::Arc;
use tracing::{debug, warn};

fn load_plugin(plugin_config: &PluginConfig, plugins: &mut PluginRegistry) -> LuaResult<()> {
    // Execute the plugin code and obtain a registration table
    let test_module: Table = match &plugin_config.plugin_source {
        PluginSource::ScriptFile(p) => plugins.lua.load(p.clone()).eval()?,
        PluginSource::ScriptSource(s) => plugins.lua.load(s).eval()?,
    };
    // Load metadata from plugin
    let plugin_name: String = test_module.get("name")?;
    let plugin_underlying = Plugin {
        name: plugin_name.clone(),
        version: test_module.get("version")?,
        author: test_module.get("author")?,
        permissions: plugin_config.permissions.clone(),
        configuration: plugin_config.configuration.clone(),
        kvstore_scope: Scope(plugin_config.kvscope.clone().unwrap_or(plugin_name.clone())),
    };
    debug!(
        "[{}] Loading plugin '{}' version '{}'",
        &plugin_name, &plugin_name, &plugin_underlying.version
    );

    // Register the plugin meta data
    let plugin = Arc::new(plugin_underlying);
    plugins.plugins.push(Arc::clone(&plugin));

    //
    // Load initialization function from plugin
    //
    if test_module.contains_key("init")? {
        debug!("[{}] Found an initialization routine", &plugin_name);
        plugins.init.push(Callback {
            plugin: Arc::clone(&plugin),
            priority: 1,
            callback: test_module.get("init")?,
        });
    }

    //
    // Load any event handlers defined by the plugin
    //
    if test_module.contains_key("listeners")? {
        let listeners: Table = test_module.get("listeners")?;
        for listener in listeners.pairs() {
            let (_idx, table): (i32, Table) = listener?;
            if table.contains_key("event")? && table.contains_key("impl")? {
                let event: String = table.get("event")?;
                debug!("[{}] Loading handler for event '{}'", &plugin.name, event);
                plugins.register_handler(&plugin, event.as_str(), table)?;
            }
        }
    }
    Ok(())
}

pub fn load_plugins(plugins: Vec<PluginConfig>) -> LuaResult<PluginRegistry> {
    let lua = init::new_lua_environment()?;
    let lua_ref: &'static Lua = Box::leak(Box::new(lua));
    let mut registry = PluginRegistry::new(lua_ref);
    plugins.iter().for_each(|p| {
        debug!("Loading plugin: {:#?} ...", p.plugin_source.to_string());
        match load_plugin(&p, &mut registry) {
            Ok(_) => {
                debug!("Loaded plugin: {:#?}", p.plugin_source.to_string());
            }
            Err(e) => {
                warn!(
                    "Failed to load plugin: {:#?}. Ignoring.",
                    p.plugin_source.to_string()
                );
                warn!("Error: {:#?}", e);
            }
        }
    });
    registry.sort_handlers();
    Ok(registry)
}
