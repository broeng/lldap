-- Plugin B of an A -> B -> A chain. See cycle_a.lua.

local on_list_users_result = function(context, users)
    local schema, err = context.api:get_schema()
    context.kvstore:store_str("get_schema_ok", tostring(err == nil))
    context.kvstore:fetch_and_increment("invocations", 0)
    return users
end

return {
    api_version = 1,
    name = "cycle_b",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_list_users_result", priority = 10, impl = on_list_users_result },
    },
}
