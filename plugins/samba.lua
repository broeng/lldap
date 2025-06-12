-- Plugin for basic Samba support
--
-- Configuration:
--   base_dn: the base dn used
--   domain_name: the domain name, should correlate with base dn
--   samba_sid: the sambaSID of your local samba node and domain
--       Use 'net getlocalsid' to obtain the SID, and ensure the
--       domain SID is set to the same after joining, with
--       getdomainsid/setdomainsid
--   user_attr_uid: User attribute with unix uid number
--   user_attr_gid: User attribute with unix gid number
--   group_attr_gid: Group attribute with unix gid number
--   init_users: Update existing user accounts at start up
--   init_groups: Update existing groups at start up
--

local calculate_samba_user_sid = function(context, uid)
    return context.configuration.samba_sid .. "-" .. tostring(uid * 2 + 1000)
end

local calculate_samba_group_sid = function(context, gid)
    return context.configuration.samba_sid .. "-" .. tostring(gid * 2 + 1001)
end

local ensure_user_attribute_exists = function(context, schema, attribute_name, attribute_type)
    if schema.user_attributes.attributes[attribute_name] == nil then
        local res = context.api:add_user_attribute({
            name = attribute_name,
            attribute_type = attribute_type,
            is_list = false,
            is_visible = true,
            is_editable = false,
        })
        if res ~= nil then
            -- Error.
            error("Got error from creating '" .. attribute_name .. "' attribute", 1)
        end
    end
end

local ensure_group_attribute_exists = function(context, schema, attribute_name, attribute_type)
    if schema.group_attributes.attributes[attribute_name] == nil then
        local res = context.api:add_group_attribute({
            name = attribute_name,
            attribute_type = attribute_type,
            is_list = false,
            is_visible = true,
            is_editable = false,
        })
        if res ~= nil then
            -- Error.
            error("Got error from creating '" .. attribute_name .. "' group attribute", 1)
        end
    end
end

local assign_samba_user_sids = function(context, user_id, existing_attributes)
    local uidattr = existing_attributes[context.configuration.user_attr_uid]
    local gidattr = existing_attributes[context.configuration.user_attr_gid]
    -- start with an empty set of attributes to update
    local result_attrs = {}
    -- determine if we have a uidnumber set for the user,
    -- which is a requirement for calculating the sambaSID
    if uidattr ~= nil then
        local expected_sid = {
            string = calculate_samba_user_sid(context, uidattr.int)
        }
        local sambasidattr = existing_attributes.sambasid or {};
        if not lldap.tables:eq(sambasidattr, expected_sid) then
            lldap.log:debug("Assigning sambaSID to user " .. user_id)
            -- add sambasid to result attrs for updating
            result_attrs["sambasid"] = expected_sid
        end
    end
    -- determine if we have gidnumber set for the user,
    -- which is a requirement for calculating the sambaPrimaryGroupId
    if gidattr ~= nil then
        local expected_sid = {
            string = calculate_samba_group_sid(context, gidattr.int)
        }
        local sambagrpsidattr = existing_attributes.sambaprimarygroupid or {};
        if not lldap.tables:eq(sambagrpsidattr, expected_sid) then
            lldap.log:debug("Assigning sambaPrimaryGroupId to user " .. user_id)
            -- add sambaprimarygroupid to result attrs for updating
            result_attrs["sambaprimarygroupid"] = expected_sid
        end
    end
    -- update the user's attributes, if we have any changes
    if not lldap.tables:empty(result_attrs) then
        context.api:update_user({
            user_id = user_id,
            insert_attributes = result_attrs
        })
    end
end

local assign_samba_group_sid = function(context, group_id, existing_attributes)
    local gidattr = existing_attributes[context.configuration.group_attr_gid]
    local sambasidattr = existing_attributes.sambasid or {};
    if gidattr ~= nil then
        local expected_sid = {
            string = calculate_samba_group_sid(context, gidattr.int)
        }
        if not lldap.tables:eq(sambasidattr, expected_sid) then
            lldap.log:debug("Assigning sambaSID to group " .. tostring(group_id))
            context.api:update_group({
                group_id = group_id,
                insert_attributes = {
                    sambasid = expected_sid
                }
            })
        end
    end
end

local initialize_attributes = function(context)
    lldap.log:debug("Initializing samba.lua")
    local schema = context.api:get_schema()
    -- ensure we have the basic samba related attributes for users
    ensure_user_attribute_exists(context, schema, "sambantpassword", "String")
    ensure_user_attribute_exists(context, schema, "sambasid", "String")
    ensure_user_attribute_exists(context, schema, "sambaprimarygroupid", "String")
    --ensure_user_attribute_exists(context, schema, "sambaacctflags", "String")
    -- ensure we have the basic samba related attributes for groups
    ensure_group_attribute_exists(context, schema, "sambasid", "String")
    -- ensure we have the 'sambaSamAccount' user object class
    if schema.extra_user_object_classes.sambaSamAccount == nil then
        local res = context.api:add_user_object_class("sambaSamAccount")
        if res ~= nil then
            -- Error.
            lldap.log:warn("Got error from creating 'sambaSamAccount' user object class")
            return res
        end
    end
    -- initialize users
    if context.configuration.init_users == "true" then
        local users, err = context.api:list_users({})
        if err ~= nil then
            lldap.log:warn("Unable to search users for user initialization")
        else
            for idx, u in pairs(users) do
                lldap.log:info("Initializing samba attributes for user " .. u.user.user_id)
                assign_samba_user_sids(context, u.user.user_id, u.user.attributes)
            end
        end
    end
    -- initialize groups
    if context.configuration.init_groups == "true" then
        local groups, err = context.api:list_groups({})
        if err ~= nil then
            lldap.log:warn("Unable to search groups for group initialization")
        else
            for idx, g in pairs(groups) do
                assign_samba_group_sid(context, g.group_id, g.attributes)
            end
        end
    end
end

local on_password_update = function(context, args, secrets)
    local ntlm_hash, err = secrets.to_ntlm_hash()
    if err ~= nil then
        lldap.log:debug("New password being set for " .. args.user_id)
        lldap.log:debug("New samba hash: " .. ntlm_hash)
        context.api:update_user({
            user_id = args.user_id,
            email = nil,
            display_name = nil,
            delete_attributes = {},
            insert_attributes = {
                sambantpassword = { string = ntlm_hash }
            }
        })
    else
        lldap.log:warn("Failed to generate NTLM hash. Unable to set new password for " .. args.user_id)
    end
    return args
end

local inject_metadata_attributes = function(users_and_groups)
    for k, v in pairs(users_and_groups.users) do
        v.user.attributes.shadowmin = { int = 10000 }
        v.user.attributes.shadowmax = { int = 999999 }
        v.user.attributes.gecos = { string = "LLDAP User" }
        v.user.attributes.loginshell = { string = "/bin/sh" }
        v.user.attributes.homeDirectory = {
            string = "/home/" .. v.user.user_id
        }
        v.user.attributes.sambaacctflags = {
            -- user (U), that never expires (X)
            string = "[UX]"
        }
    end
end

local samba_probe_result = function(context)
    return {
        {
            search_result_entry = {
                dn = context.configuration.base_dn,
                attributes = {
                    {
                        atype = "sambaDomainName",
                        vals = {
                            lldap.strings:to_utf8(context.configuration.domain_name)
                        }
                    },
                    {
                        atype = "sambaSID",
                        vals = {
                            lldap.strings:to_utf8(context.configuration.samba_sid)
                        }
                    }
                }
            }
        }
    }
end

local on_search_result = function(context, args)
    local search_request = args.search_request
    print("search filter: " .. tostring(search_request.filter))
    if search_request.filter ~= nil then
        local class_filter = { equality = { "objectClass", "sambaDomain" } }
        local class_and_name_filter = {
            ["and"] = {
                class_filter,
                { equality = { "sambaDomainName", context.configuration.domain_name } },
            }
        }
        if lldap.tables:eq(search_request.filter, class_filter) then
            lldap.log:info("Matched sambaDomain class filter")
            args.search_result.ldap = samba_probe_result(context)
        elseif lldap.tables:eq(search_request.filter, class_and_name_filter) then
            lldap.log:info("Matched sambaDomainName and class filter")
            args.search_result.ldap = samba_probe_result(context)
        end
    end
    -- inject some samba related attributes that we don't really want in our schema.
    if args.search_result.users_and_groups ~= nil then
        inject_metadata_attributes(args.search_result.users_and_groups)
    end
    return args
end


local on_created_or_updated_group = function(context, filter)
    local groups, err = context.api:list_groups({
        filter = {
            groupQuery = {
                filter = filter
            }
        }
    })
    if err == nil then
        for idx, group in pairs(groups) do
            assign_samba_group_sid(context, group.group_id, group.attributes)
        end
    end
end

local on_updated_group = function(context, args)
    on_created_or_updated_group(context, {
        groupId = args.group_id
    })
    return args
end

local on_created_group = function(context, args)
    -- assign sambaSID to new group
    -- we do this after creation, as we need the pam plugin
    -- to assign a gidnumber before us.
    on_created_or_updated_group(context, {
        displayName = args.display_name
    })
    return args
end

local on_create_or_update_user = function(context, attributes)
    local uidattr = attributes[context.configuration.user_attr_uid]
    if uidattr ~= nil then
        attributes.sambasid = {
            string = calculate_samba_user_sid(context, uidattr.int)
        }
    end
    local gidattr = attributes[context.configuration.user_attr_gid]
    if gidattr ~= nil then
        attributes.sambaprimarygroupid = {
            string = calculate_samba_group_sid(context, gidattr.int)
        }
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
    name = "samba",
    version = "1.0",
    author = "broeng",
    repo = "https://github.com/broeng/lldap-plugins/samba/samba.lua",
    init = initialize_attributes,
    listeners = {
        -- Which event you subscribe to, the priority (highest gets called first), and the function to call.
        { event = "on_create_user",          priority = 100, impl = on_create_user },
        { event = "on_update_user",          priority = 100, impl = on_update_user },
        { event = "on_created_group",        priority = 100, impl = on_created_group },
        { event = "on_updated_group",        priority = 100, impl = on_updated_group },
        { event = "on_ldap_password_update", priority = 40,  impl = on_password_update },
        { event = "on_ldap_search_result",   priority = 40,  impl = on_search_result },
    },
}
