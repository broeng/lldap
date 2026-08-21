use mlua::{FromLua, IntoLua, LuaSerdeExt};

use crate::internal::lualib::init::new_lua_environment;
use crate::tests::lua_scripts::SCRIPT_TEST_LIB;

// Sends `value` through a Lua function and back, and returns the result. `lua_body` can use
// `assert_eq` to examine `value` in Lua.
//
// The global `LUA_NULL` holds mlua's null value. `lua.to_value` gives this value for `None`, not
// `nil`. Thus, scripts must compare an absent field with `LUA_NULL`, unless serde skips the
// field for `None`.
pub fn assert_roundtrip<T>(value: T, lua_body: &str) -> T
where
    T: IntoLua + FromLua + 'static,
{
    let lua = new_lua_environment().expect("failed to create lua environment");
    lua.globals()
        .set("LUA_NULL", lua.null())
        .expect("failed to set LUA_NULL global");
    // `assert_eq` is local to SCRIPT_TEST_LIB, thus `lua_body` must be in the same chunk.
    let script = format!(
        r#"
        {SCRIPT_TEST_LIB}
        return function(value)
            {lua_body}
            return value
        end
        "#
    );
    let check_and_return: mlua::Function = lua
        .load(script)
        .eval()
        .expect("failed to load roundtrip script");
    check_and_return.call(value).expect("roundtrip call failed")
}
