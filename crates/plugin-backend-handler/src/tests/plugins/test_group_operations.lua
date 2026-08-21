-- Creating a group in on_created_group does not cause endless recursion,
-- because the plugin that makes a call does not get the event.

local record = function(context, key, value)
    context.kvstore:store_str(key, tostring(value))
end

local on_created_group = function(context, args)
    local group_id, err = context.api:create_group({ display_name = "workshop" })
    record(context, "create_group", err == nil)

    local _, err = context.api:add_group_attribute({
        name = "team_colour",
        attribute_type = "String",
        is_list = false,
        is_visible = true,
        is_editable = true,
    })
    record(context, "add_group_attribute", err == nil)

    -- A second attribute, to make sure that a value deletion has no effect on
    -- other attributes.
    local _, err = context.api:add_group_attribute({
        name = "team_motto",
        attribute_type = "String",
        is_list = false,
        is_visible = true,
        is_editable = true,
    })
    record(context, "add_group_attribute_second", err == nil)

    local _, err = context.api:update_group({
        group_id = group_id,
        display_name = "workshop_renamed",
        insert_attributes = {
            team_colour = { string = "green" },
            team_motto = { string = "make things" },
        },
    })
    record(context, "update_group", err == nil)

    local details, err = context.api:get_group_details(group_id)
    record(context, "get_group_details", err == nil)
    record(context, "name_after_update", details.display_name)
    record(context, "colour_after_update", details.attributes.team_colour.string)
    record(context, "motto_after_update", details.attributes.team_motto.string)

    -- Delete only the value. The attribute stays in the schema.
    local _, err = context.api:update_group({
        group_id = group_id,
        delete_attributes = { "team_motto" },
    })
    record(context, "update_group_delete_attributes", err == nil)

    local details, err = context.api:get_group_details(group_id)
    record(context, "motto_value_gone", details.attributes.team_motto == nil)
    record(context, "colour_still_set", details.attributes.team_colour ~= nil)

    local _, err = context.api:delete_group_attribute("team_colour")
    record(context, "delete_group_attribute", err == nil)

    local details, err = context.api:get_group_details(group_id)
    record(context, "colour_gone_from_group", details.attributes.team_colour == nil)

    local _, err = context.api:delete_group(group_id)
    record(context, "delete_group", err == nil)

    local _, err = context.api:get_group_details(group_id)
    record(context, "details_after_delete_errored", err ~= nil)
end

return {
    api_version = 1,
    name = "test_group_operations",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_group", priority = 100, impl = on_created_group },
    },
}
