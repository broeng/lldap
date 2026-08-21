-- Tries to set a global for other plugins.

local on_created_user = function(context, args)
    local wrote = pcall(function()
        leaked_secret = "from_writer"
    end)
    context.kvstore:store_str("wrote_global", tostring(wrote))
end

return {
    api_version = 1,
    name = "global_writer",
    version = "1.0",
    author = "broeng",
    listeners = {
        -- A lower priority runs first. Thus, this runs before global_reader.lua.
        { event = "on_created_user", priority = 10, impl = on_created_user },
    },
}
