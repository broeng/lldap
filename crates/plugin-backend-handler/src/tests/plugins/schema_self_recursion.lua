-- Reads the schema in its own on_get_schema callback, to examine if the
-- callback runs again. The plugin does this only one time, so that the result
-- does not depend on the depth limit.

local on_get_schema = function(context, schema)
    local invocations = context.kvstore:fetch_and_increment("invocations", 0)
    if invocations == 0 then
        context.api:get_schema()
    end
    return schema
end

return {
    api_version = 1,
    name = "schema_self_recursion",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_get_schema", priority = 10, impl = on_get_schema },
    },
}
