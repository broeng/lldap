-- Plugin B of a mutually recursive pair. See
-- test_mutual_plugin_recursion_is_cut_short.

local on_get_schema = function(context, schema)
    context.kvstore:fetch_and_increment("invocations", 0)
    context.api:get_schema()
    return schema
end

return {
    api_version = 1,
    name = "mutual_recursion_b",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_get_schema", priority = 20, impl = on_get_schema },
    },
}
