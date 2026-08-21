-- Tries to overwrite the standard library and the lldap library.

local on_created_user = function(context, args)
    local stdlib_write = pcall(function()
        string.upper = function(_)
            return "TAMPERED"
        end
    end)
    local lldap_write = pcall(function()
        lldap.log = "clobbered"
    end)
    context.kvstore:store_str("ran", "yes")
    context.kvstore:store_str("stdlib_write_allowed", tostring(stdlib_write))
    context.kvstore:store_str("lldap_write_allowed", tostring(lldap_write))
end

return {
    api_version = 1,
    name = "lib_tamperer",
    version = "1.0",
    author = "broeng",
    listeners = {
        -- A lower priority runs first. Thus, this runs before lib_victim.lua.
        { event = "on_created_user", priority = 10, impl = on_created_user },
    },
}
