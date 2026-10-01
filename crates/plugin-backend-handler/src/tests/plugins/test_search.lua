-- Runs each filter as an LDAP filter string and as a structured query, so that
-- the test can compare the two. A group creation starts this plugin, thus the
-- only user is the user that this plugin creates.

local record = function(context, key, value)
    context.kvstore:store_str(key, tostring(value))
end

local user_query = function(context, name, ldap_query, structured)
    local users, err = context.api:list_users({ filter = { ldapQuery = ldap_query } })
    record(context, name .. "_ldap", err and ("error: " .. err) or #users)

    local users, err = context.api:list_users({ filter = { userQuery = structured } })
    record(context, name .. "_structured", err and ("error: " .. err) or #users)
end

local group_query = function(context, name, ldap_query, structured)
    local groups, err = context.api:list_groups({ filter = { ldapQuery = ldap_query } })
    record(context, name .. "_ldap", err and ("error: " .. err) or #groups)

    local groups, err = context.api:list_groups({ filter = { groupQuery = structured } })
    record(context, name .. "_structured", err and ("error: " .. err) or #groups)
end

local on_created_group = function(context, args)
    local _, err = context.api:create_user({
        user_id = "searchuser",
        email = "searchuser@example.com",
        display_name = "Search User",
    })
    record(context, "create_user", err == nil)

    local group_id, err = context.api:create_group({ display_name = "searchers" })
    record(context, "create_group", err == nil)

    local _, err = context.api:add_user_to_group("searchuser", group_id)
    record(context, "add_user_to_group", err == nil)

    local _, err = context.api:add_user_object_class("Person")
    record(context, "add_user_object_class", err == nil)

    local _, err = context.api:add_user_object_class("lldapTestPerson")
    record(context, "add_user_object_class_extra", err == nil)

    local _, err = context.api:add_group_object_class("Team")
    record(context, "add_group_object_class", err == nil)

    user_query(context, "user_by_uid", "(uid=searchuser)", {
        filter = { userId = "searchuser" },
        need_groups = false,
    })
    user_query(context, "user_by_missing_uid", "(uid=nobody)", {
        filter = { userId = "nobody" },
        need_groups = false,
    })
    user_query(context, "user_by_email", "(mail=searchuser@example.com)", {
        filter = { equality = { "Email", "searchuser@example.com" } },
        need_groups = false,
    })
    user_query(context, "user_by_member_of", "(memberOf=searchers)", {
        filter = { memberOf = "searchers" },
        need_groups = false,
    })
    user_query(context, "user_by_missing_member_of", "(memberOf=nosuchgroup)", {
        filter = { memberOf = "nosuchgroup" },
        need_groups = false,
    })
    user_query(context, "user_by_default_object_class", "(objectClass=Person)", {
        filter = "true",
        need_groups = false,
    })
    user_query(context, "user_by_added_object_class", "(objectClass=lldapTestPerson)", {
        filter = "true",
        need_groups = false,
    })
    user_query(context, "user_by_unknown_object_class", "(objectClass=nosuchclass)", {
        filter = "false",
        need_groups = false,
    })
    user_query(
        context,
        "user_by_and",
        "(&(uid=searchuser)(memberOf=searchers))",
        {
            filter = { ["and"] = { { userId = "searchuser" }, { memberOf = "searchers" } } },
            need_groups = false,
        }
    )
    user_query(context, "user_by_or", "(|(uid=searchuser)(uid=nobody))", {
        filter = { ["or"] = { { userId = "searchuser" }, { userId = "nobody" } } },
        need_groups = false,
    })
    user_query(context, "user_by_not", "(!(uid=nobody))", {
        filter = { ["not"] = { userId = "nobody" } },
        need_groups = false,
    })
    user_query(context, "user_by_uid_prefix", "(uid=search*)", {
        filter = { userIdSubstring = { initial = "search", any = {} } },
        need_groups = false,
    })

    -- A listing that asks for groups gives the memberships.
    local users, err = context.api:list_users({
        filter = { userQuery = { filter = { userId = "searchuser" }, need_groups = true } },
    })
    record(context, "need_groups_ok", err == nil)
    record(context, "need_groups_count", #users[1].groups)

    group_query(context, "group_by_name", "(cn=searchers)", { filter = { displayName = "searchers" } })
    group_query(context, "group_by_missing_name", "(cn=nosuchgroup)", {
        filter = { displayName = "nosuchgroup" },
    })
    group_query(context, "group_by_member", "(member=searchuser)", {
        filter = { member = "searchuser" },
    })
    group_query(context, "group_by_added_object_class", "(objectClass=Team)", { filter = "true" })
    group_query(context, "group_by_unknown_object_class", "(objectClass=nosuchclass)", {
        filter = "false",
    })
    group_query(context, "group_by_name_prefix", "(cn=search*)", {
        filter = { displayNameSubString = { initial = "search", any = {} } },
    })

    -- Delete the object classes, and run the same queries again. A deleted
    -- class must match nothing.
    local _, err = context.api:delete_user_object_class("lldapTestPerson")
    record(context, "delete_user_object_class", err == nil)

    user_query(context, "user_by_deleted_object_class", "(objectClass=lldapTestPerson)", {
        filter = "false",
        need_groups = false,
    })

    local _, err = context.api:delete_group_object_class("Team")
    record(context, "delete_group_object_class", err == nil)

    group_query(context, "group_by_deleted_object_class", "(objectClass=Team)", {
        filter = "false",
    })
end

return {
    api_version = 1,
    name = "test_search",
    version = "1.0",
    author = "broeng",
    listeners = {
        { event = "on_created_group", priority = 100, impl = on_created_group },
    },
}
