//! A backend call from a plugin also dispatches plugin events. Thus, a plugin
//! can cause the event that it handles. Two rules stop endless recursion:
//!
//!   * The plugin that makes the call does not get the event.
//!   * A request can nest only to a maximum depth. This stops cycles between
//!     two or more plugins.

use lldap_domain_handlers::handler::RequestContext;
use tracing::{debug, warn};

use crate::internal::types::plugins::Plugin;

// The maximum number of nested plugin callbacks for one request.
pub const MAX_PLUGIN_CALL_DEPTH: usize = 8;

pub fn exceeds_max_depth(context: &RequestContext) -> bool {
    if context.plugin_depth() >= MAX_PLUGIN_CALL_DEPTH {
        let plugins: Vec<&str> = context
            .plugin_stack
            .iter()
            .map(|invocation| invocation.plugin_name.as_str())
            .collect();
        warn!(
            "Plugin call depth {} reached, refusing to dispatch further. Call stack: {:?}",
            MAX_PLUGIN_CALL_DEPTH, plugins
        );
        true
    } else {
        false
    }
}

pub fn is_calling_plugin(context: &RequestContext, plugin: &Plugin) -> bool {
    match context.calling_plugin() {
        Some(invocation) if invocation.plugin_name == plugin.name => {
            debug!(
                "Not dispatching to '{}': it is the plugin making the call.",
                plugin.name
            );
            true
        }
        _ => false,
    }
}
