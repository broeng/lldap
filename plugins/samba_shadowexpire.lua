-- Plugin for basic Samba shadowexpire support
--
-- Maintains a 'shadowexpire' user attribute based on an
-- active/disabled flag in the form of another user attribute.
--
-- Configuration:
--   follow_attribute: the attribute used to determine active/inactive state
--   active_value: set the value denoting an active user

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
end

local on_create_or_update_user = function(context, attributes)
    local attr = attributes[context.configuration.follow_attribute]
    if attr ~= nil then
        if tostring(attr.int) == context.configuration.active_value then
            -- mark account as active
            attributes.shadowexpire = {
                int = -1
            }
        else
            -- disable account
            attributes.shadowexpire = {
                int = 0
            }
        end
    end
    return attributes
end

local on_create_user = function(context, args)
    args.attributes = on_create_or_update_user(context, args.attributes)
    return args
end

local on_update_user = function(context, args)
    args.insert_attributes = on_create_or_update_user(context, args.insert_attributes)
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
