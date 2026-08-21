-- Each call records its result, so that a failed test shows the step that
-- failed. A failed call gives (null, message), thus the plugin examines the
-- second value.

local record = function(context, key, value)
    context.kvstore:store_str(key, tostring(value))
end

local on_created_user = function(context, args)
    local user_id = args.user_id

    local schema, err = context.api:get_schema()
    record(context, "get_schema", err == nil)
    record(context, "schema_has_user_attributes", schema.user_attributes ~= nil)

    local _, err = context.api:add_user_attribute({
        name = "favourite_colour",
        attribute_type = "String",
        is_list = false,
        is_visible = true,
        is_editable = true,
    })
    record(context, "add_user_attribute", err == nil)

    local group_id, err = context.api:create_group({ display_name = "colour_club" })
    record(context, "create_group", err == nil)

    local _, err = context.api:add_user_to_group(user_id, group_id)
    record(context, "add_user_to_group", err == nil)

    local _, err = context.api:update_user({
        user_id = user_id,
        insert_attributes = { favourite_colour = { string = "blue" } },
    })
    record(context, "update_user", err == nil)

    -- Read the values back, to also test the conversions from Rust to Lua.
    local user, err = context.api:get_user_details(user_id)
    record(context, "get_user_details", err == nil)
    record(context, "user_colour", user.attributes.favourite_colour.string)
    record(context, "user_email", user.email)

    local groups, err = context.api:get_user_groups(user_id)
    record(context, "get_user_groups", err == nil)
    record(context, "user_group_count", #groups)

    local details, err = context.api:get_group_details(group_id)
    record(context, "get_group_details", err == nil)
    record(context, "group_display_name", details.display_name)

    local users, err = context.api:list_users({
        filter = { ldapQuery = "(uid=" .. user_id .. ")" },
    })
    record(context, "list_users", err == nil)
    record(context, "listed_user_count", #users)
    record(context, "listed_user_id", users[1].user.user_id)

    local all_groups, err = context.api:list_groups({})
    record(context, "list_groups", err == nil)
    record(context, "listed_group_count", #all_groups)
end

return {
    api_version = 1,
    name = "test_server_api",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 100, impl = on_created_user },
    },
}
