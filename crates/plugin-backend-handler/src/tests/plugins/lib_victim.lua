local on_created_user = function(context, args)
    local stdlib_result = string.upper("ok")
    local log_usable = pcall(function()
        lldap.log:debug("victim still has a working logger")
    end)
    context.kvstore:store_str("stdlib_result", stdlib_result)
    context.kvstore:store_str("lldap_log_usable", tostring(log_usable))
end

return {
    api_version = 1,
    name = "lib_victim",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 90, impl = on_created_user },
    },
}
