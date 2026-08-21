-- Plugin A of an A -> B -> A chain. See
-- test_plugin_cycle_reaches_first_plugin_only_once.

local on_created_user = function(context, args)
    local users, err = context.api:list_users({
        filter = { ldapQuery = "(uid=bob)" },
    })
    context.kvstore:store_str("list_users_ok", tostring(err == nil))
end

local on_get_schema = function(context, schema)
    context.kvstore:fetch_and_increment("get_schema_invocations", 0)
    return schema
end

return {
    api_version = 1,
    name = "cycle_a",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 10, impl = on_created_user },
        { event = "on_get_schema", priority = 10, impl = on_get_schema },
    },
}
