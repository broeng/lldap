use mlua::{Error, Result as LuaResult, Table, UserData, UserDataMethods, Value};

#[derive(Clone, Debug)]
pub struct LuaTablesLib;

//
// String conversion utilities
//
impl UserData for LuaTablesLib {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("eq", |_, _, (a, b): (Table, Table)| table_equals(&a, &b));
        methods.add_method("has_subtree", |_, _, (base, tree): (Table, Table)| {
            table_has_tree(&base, &tree)
        });
        methods.add_method("empty", |_, _, t: Table| {
            Ok(t.pairs()
                .collect::<Vec<Result<(Value, Value), Error>>>()
                .is_empty())
        });
    }
}

fn table_equals(a: &Table, b: &Table) -> LuaResult<bool> {
    // Collect the elements to vec's, for reliable length comparison
    let apairs = a.pairs().collect::<Vec<Result<(Value, Value), Error>>>();
    let bpairs = b.pairs().collect::<Vec<Result<(Value, Value), Error>>>();
    if apairs.len() != bpairs.len() {
        return Ok(false);
    }
    for pair in apairs {
        let (key, value) = pair?;
        if !b.contains_key(key.clone())? {
            return Ok(false);
        }
        let bval: Value = b.get(key)?;
        if !value.type_name().eq_ignore_ascii_case(bval.type_name()) {
            return Ok(false);
        }
        if value.is_table() {
            if !table_equals(value.as_table().unwrap(), bval.as_table().unwrap())? {
                return Ok(false);
            }
        } else if !value.equals(&bval)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn table_has_tree(base: &Table, tree: &Table) -> LuaResult<bool> {
    if table_equals(base, tree)? {
        return Ok(true);
    }
    for pair in base.pairs::<Value, Value>() {
        let (_, value) = pair?;
        if let Some(subtable) = value.as_table()
            && table_has_tree(subtable, tree)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}
