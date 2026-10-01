use mlua::{Lua, LuaOptions, Result as LuaResult, StdLib, Value};

use crate::internal::lualib::{
    encoding::LuaEncodingLib, hashing::LuaHashingLib, logger::LuaLogger, strings::LuaStringsLib,
    tables::LuaTablesLib,
};

pub fn new_lua_environment() -> LuaResult<Lua> {
    let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())?;
    let lldap_lib = lua.create_table()?;
    lldap_lib.set("encoding", LuaEncodingLib {})?;
    lldap_lib.set("hashing", LuaHashingLib {})?;
    lldap_lib.set("log", LuaLogger {})?;
    lldap_lib.set("strings", LuaStringsLib {})?;
    lldap_lib.set("tables", LuaTablesLib {})?;
    let globals = lua.globals();
    globals.set("lldap", Value::Table(lldap_lib))?;
    Ok(lua)
}
