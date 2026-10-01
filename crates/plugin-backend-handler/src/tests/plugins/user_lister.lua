-- Lists users when a user is created. Tests use this plugin to check which
-- need_groups value reaches the backend for each kind of query.

local on_created_user = function(context, args)
    local _, err = context.api:list_users({
        filter = { userQuery = { filter = { userId = args.user_id }, need_groups = false } },
    })
    if err ~= nil then
        error("failed to list users with a user query: " .. err, 1)
    end

    local _, err = context.api:list_users({ filter = { ldapQuery = "(uid=" .. args.user_id .. ")" } })
    if err ~= nil then
        error("failed to list users with an LDAP query: " .. err, 1)
    end
end

return {
    api_version = 1,
    name = "user_lister",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 100, impl = on_created_user },
    },
}
