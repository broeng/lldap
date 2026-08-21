use mlua::{FromLua, Result as LuaResult, Table, Value};

use lldap_domain::types::Attribute;

use crate::internal::types::attribute_map;

pub fn get_opt_arg<T: FromLua>(key: &str, table: &Table) -> LuaResult<Option<T>> {
    if table.contains_key(key)? {
        Ok(Some(table.get(key)?))
    } else {
        Ok(None)
    }
}

pub fn get_opt_vec<T: FromLua>(key: &str, table: &Table) -> LuaResult<Vec<T>> {
    if table.contains_key(key)? {
        Ok(table.get(key)?)
    } else {
        Ok(Vec::new())
    }
}

pub fn get_opt_attrmap(key: &str, table: &Table) -> LuaResult<Vec<Attribute>> {
    if table.contains_key(key)? {
        let value: Value = table.get(key)?;
        attribute_map::deserialize(mlua::serde::de::Deserializer::new(value))
    } else {
        Ok(Vec::new())
    }
}
