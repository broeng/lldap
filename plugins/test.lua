local initialize_attributes = function(context)
    print("Initializing test.lua")
    teststr = "test"
    bytes = lldap.strings:to_utf8(teststr)
    sha256hash = lldap.hashing:sha256_hash_bytes(bytes)
    sha256_str = lldap.encoding:base16_encode(sha256hash)
    print(sha256_str)
    utf16bytes = lldap.strings:to_utf16le(teststr)
    md4hash = lldap.hashing:md4_hash_bytes(utf16bytes)
    md4str = lldap.encoding:base16_encode(md4hash)
    print(md4str:upper())
    local schema = context.api:get_schema()
    context.kvstore:store_table("json", {
        a = "b",
        c = "d",
    })
    local res, err = context.kvstore:fetch_str("json")
    if err ~= nil then
        print("Err: " .. tostring(err))
    end
    print("JSON: " .. tostring(res))
    local table, err = context.kvstore:fetch_table("json")
    if table ~= nil then
        print("table.a" .. table.a)
        print("table.c" .. table.c)
    end
    local uid, err = context.kvstore:fetch_and_increment("uid", 100000)
    print("uid: " .. tostring(uid))
    uid, err = context.kvstore:fetch_and_increment("uid", 100000)
    print("uid: " .. tostring(uid))
    uid, err = context.kvstore:fetch_and_increment("uid", 100000)
    print("uid: " .. tostring(uid))
end

local on_password_update = function(context, args)
    lldap.log:debug("New password being set for " .. args.user_id)
    utf16bytes = lldap.strings:to_utf16le(args.password)
    md4hash = lldap.hashing:md4_hash_bytes(utf16bytes)
    md4str = lldap.encoding:base16_encode(md4hash)
    ntlmhash = md4str:upper()
    lldap.log:debug("New samba hash: " .. ntlmhash)
    return args
end

local on_create_user = function(context, args)
    print("test: on_create_user")
    print(tostring(args))
    for k, v in pairs(args) do
        print(k .. "=" .. tostring(v))
    end
    for i, v in pairs(args.attributes) do
        print("Attribute: " .. i .. ":")
        for k, v in pairs(args.attributes[i]) do
            print("  => " .. k .. "=" .. tostring(v))
        end
    end
    args.display_name = "Simon Jensen"
    --args.first_name = "Simon"
    --args.last_name = "Jensen"
    print(args.user_id)
    args.attributes["last_name"] = { string = "MF" }
    for i, v in pairs(args.attributes) do
        print("Attribute: " .. i .. ":")
        for k, v in pairs(args.attributes[i]) do
            print("  => " .. k .. "=" .. tostring(v))
        end
    end
    print(args.display_name)
    return args
end

local on_create_group = function(context, args)
    return args
end

return {
    api_version = 1,
    name = "test",
    version = "1.0",
    author = "broeng",
    repo = "https://github.com/nitnelave/lldap_plugin_poc/lua/pam.lua",
    init = initialize_attributes,
    listeners = {
        -- Which event you subscribe to, the priority (highest gets called first), and the function to call.
        { event = "on_create_user",          priority = 40, impl = on_create_user },
        { event = "on_create_group",         priority = 40, impl = on_create_group },
        { event = "on_ldap_password_update", priority = 40, impl = on_password_update },
    },
}
