-- Records each group creation, so that a test can examine which events this
-- plugin received.

local on_created_group = function(context, args)
    local count, err = context.kvstore:fetch_and_increment("created_group_count", 0)
    if err ~= nil then
        error("failed to increment counter: " .. err, 1)
    end
    local _, err = context.kvstore:store_str("last_created_group", args.display_name)
    if err ~= nil then
        error("failed to record group: " .. err, 1)
    end
end

return {
    api_version = 1,
    name = "recorder",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_group", priority = 100, impl = on_created_group },
    },
}
