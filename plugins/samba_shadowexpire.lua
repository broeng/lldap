-- Plugin for basic Samba shadowexpire support
--
-- Maintains a 'shadowexpire' user attribute based on an
-- active/disabled flag in the form of another user attribute.
--
-- Configuration:
--   follow_attribute: the attribute used to determine active/inactive state
--   active_value: set the value denoting an active user

local on_create_or_update_user = function(context, attributes, out_attrs)
    local attr = attributes[context.configuration.follow_attribute]
    if attr ~= nil then
        if tostring(attr.int) == context.configuration.active_value then
            -- mark account as active
            out_attrs.shadowexpire = {
                int = -1
            }
        else
            -- disable account
            out_attrs.shadowexpire = {
                int = 0
            }
        end
    else
        lldap.log:info("Couldn't find follow_attribute " .. context.configuration.follow_attribute)
    end
end

local initialize_attributes = function(context)
    lldap.log:debug("Initializing samba_shadowexpire")
    local schema = context.api:get_schema()
    if schema.user_attributes.attributes.shadowexpire == nil then
        local res = context.api:add_user_attribute({
            name = "shadowexpire",
            attribute_type = "Integer",
            is_list = false,
            is_visible = true,
            is_editable = false,
        })
        if res ~= nil then
            -- Error.
            error("Got error from creating 'shadowexpire' attribute", 1)
        end
    end
    -- initialize users
    if context.configuration.init_users == "true" then
        local users, err = context.api:list_users({})
        if err ~= nil then
            lldap.log:warn("Unable to search users for user initialization")
        else
            for idx, u in pairs(users) do
                local updated_attributes = {}
                on_create_or_update_user(context, u.user.attributes, updated_attributes)
                if not lldap.tables:empty(updated_attributes) then
                    lldap.log:info("Initializing shadowexpire attribute for user " .. u.user.user_id)
                    context.api:update_user({
                        user_id = u.user.user_id,
                        insert_attributes = updated_attributes
                    })
                end
            end
        end
    end
end

local on_create_user = function(context, args)
    on_create_or_update_user(context, args.attributes, args.attributes)
    return args
end

local on_update_user = function(context, args)
    on_create_or_update_user(context, args.insert_attributes, args.insert_attributes)
    return args
end

return {
    api_version = 1,
    name = "samba_shadowexpire",
    version = "1.0",
    author = "broeng",
    repo = "https://github.com/broeng/lldap-plugins/samba/samba_shadowexpire.lua",
    init = initialize_attributes,
    listeners = {
        -- Which event you subscribe to, the priority (highest gets called first), and the function to call.
        { event = "on_create_user", priority = 100, impl = on_create_user },
        { event = "on_update_user", priority = 100, impl = on_update_user },
    },
}
