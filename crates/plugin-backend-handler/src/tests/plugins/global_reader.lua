local on_created_user = function(context, args)
    local env = _G or {}
    local seen = rawget(env, "leaked_secret")
    context.kvstore:store_str("observed_global", tostring(seen))
end

return {
    api_version = 1,
    name = "global_reader",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 90, impl = on_created_user },
    },
}
