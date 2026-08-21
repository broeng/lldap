-- Creates a group when a user is created. Tests use this plugin to make a
-- backend call from a plugin.

local on_created_user = function(context, args)
    local group_id, err = context.api:create_group({
        display_name = "derived_from_" .. args.user_id,
    })
    if err ~= nil then
        error("failed to create group: " .. err, 1)
    end
    lldap.log:info("created group " .. tostring(group_id))
end

return {
    api_version = 1,
    name = "creator",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 100, impl = on_created_user },
    },
}
