-- Creating a user in on_created_user does not cause endless recursion,
-- because the plugin that makes a call does not get the event.

local record = function(context, key, value)
    context.kvstore:store_str(key, tostring(value))
end

local on_created_user = function(context, args)
    local _, err = context.api:create_user({
        user_id = "bob",
        email = "bob@example.com",
        display_name = "Bob",
    })
    record(context, "create_user", err == nil)

    local _, err = context.api:add_user_attribute({
        name = "temp_marker",
        attribute_type = "String",
        is_list = false,
        is_visible = true,
        is_editable = true,
    })
    record(context, "add_user_attribute", err == nil)

    -- A second attribute, to make sure that a value deletion has no effect on
    -- other attributes.
    local _, err = context.api:add_user_attribute({
        name = "temp_note",
        attribute_type = "String",
        is_list = false,
        is_visible = true,
        is_editable = true,
    })
    record(context, "add_user_attribute_second", err == nil)

    local _, err = context.api:update_user({
        user_id = "bob",
        insert_attributes = {
            temp_marker = { string = "here" },
            temp_note = { string = "noted" },
        },
    })
    record(context, "update_user", err == nil)

    local user, err = context.api:get_user_details("bob")
    record(context, "note_after_update", user.attributes.temp_note.string)

    -- Delete only the value. The attribute stays in the schema.
    local _, err = context.api:update_user({
        user_id = "bob",
        delete_attributes = { "temp_note" },
    })
    record(context, "update_user_delete_attributes", err == nil)

    local user, err = context.api:get_user_details("bob")
    record(context, "note_value_gone", user.attributes.temp_note == nil)
    record(context, "marker_still_set", user.attributes.temp_marker ~= nil)

    local group_id, err = context.api:create_group({ display_name = "removal_club" })
    record(context, "create_group", err == nil)

    local _, err = context.api:add_user_to_group("bob", group_id)
    record(context, "add_user_to_group", err == nil)

    local groups, err = context.api:get_user_groups("bob")
    record(context, "groups_before_removal", #groups)

    local _, err = context.api:remove_user_from_group("bob", group_id)
    record(context, "remove_user_from_group", err == nil)

    local groups, err = context.api:get_user_groups("bob")
    record(context, "groups_after_removal", #groups)

    -- Delete an attribute that still has a value, to make sure that the value
    -- is also deleted.
    local _, err = context.api:delete_user_attribute("temp_marker")
    record(context, "delete_user_attribute", err == nil)

    local user, err = context.api:get_user_details("bob")
    record(context, "marker_gone_from_user", user.attributes.temp_marker == nil)

    local _, err = context.api:delete_user("bob")
    record(context, "delete_user", err == nil)

    local _, err = context.api:get_user_details("bob")
    record(context, "details_after_delete_errored", err ~= nil)
end

return {
    api_version = 1,
    name = "test_user_operations",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_user", priority = 100, impl = on_created_user },
    },
}
