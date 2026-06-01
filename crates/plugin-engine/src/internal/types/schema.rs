use std::collections::BTreeMap;

use lldap_domain::{
    schema::{AttributeList, AttributeSchema, Schema},
    types::{AttributeName, AttributeType, LdapObjectClass},
};
use mlua::{FromLua, IntoLua, LuaSerdeExt};
use serde::{Deserialize, Serialize};
use tealr::ToTypename;

#[derive(Clone, Debug, Serialize, Deserialize, ToTypename)]
pub struct LuaSchema {
    pub user_attributes: LuaAttributeList,
    pub group_attributes: LuaAttributeList,
    pub extra_user_object_classes: BTreeMap<String, bool>,
    pub extra_group_object_classes: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToTypename)]
pub struct LuaAttributeSchema {
    pub name: String,
    pub attribute_type: AttributeType,
    pub is_list: bool,
    pub is_visible: bool,
    pub is_editable: bool,
    pub is_hardcoded: bool,
    pub is_readonly: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToTypename)]
pub struct LuaAttributeList {
    pub attributes: BTreeMap<String, LuaAttributeSchema>,
}

impl FromLua for LuaAttributeSchema {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
impl IntoLua for LuaAttributeSchema {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self)
    }
}

impl From<AttributeSchema> for LuaAttributeSchema {
    fn from(value: AttributeSchema) -> Self {
        LuaAttributeSchema {
            name: value.name.into_string(),
            attribute_type: value.attribute_type,
            is_list: value.is_list,
            is_visible: value.is_visible,
            is_editable: value.is_editable,
            is_hardcoded: value.is_hardcoded,
            is_readonly: value.is_hardcoded,
        }
    }
}

impl Into<AttributeSchema> for LuaAttributeSchema {
    fn into(self) -> AttributeSchema {
        AttributeSchema {
            name: AttributeName::from(self.name),
            attribute_type: self.attribute_type,
            is_list: self.is_list,
            is_visible: self.is_visible,
            is_editable: self.is_editable,
            is_hardcoded: self.is_hardcoded,
            is_readonly: self.is_readonly,
        }
    }
}

impl FromLua for LuaAttributeList {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
impl IntoLua for LuaAttributeList {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self)
    }
}

impl From<AttributeList> for LuaAttributeList {
    fn from(value: AttributeList) -> Self {
        LuaAttributeList {
            attributes: value
                .attributes
                .into_iter()
                .map(|a| (a.name.to_string(), a.into()))
                .collect(),
        }
    }
}

impl Into<AttributeList> for LuaAttributeList {
    fn into(self) -> AttributeList {
        AttributeList {
            attributes: self.attributes.into_iter().map(|e| e.1.into()).collect(),
        }
    }
}

impl FromLua for LuaSchema {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}
impl IntoLua for LuaSchema {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self)
    }
}

impl From<Schema> for LuaSchema {
    fn from(value: Schema) -> Self {
        LuaSchema {
            user_attributes: value.user_attributes.into(),
            group_attributes: value.group_attributes.into(),
            extra_user_object_classes: value
                .extra_user_object_classes
                .into_iter()
                .map(|a| (a.into_string(), true))
                .collect(),
            extra_group_object_classes: value
                .extra_group_object_classes
                .into_iter()
                .map(|a| (a.into_string(), true))
                .collect(),
        }
    }
}

impl Into<Schema> for LuaSchema {
    fn into(self) -> Schema {
        Schema {
            user_attributes: self.user_attributes.into(),
            group_attributes: self.group_attributes.into(),
            extra_user_object_classes: self
                .extra_user_object_classes
                .into_iter()
                .map(|e| LdapObjectClass::from(e.0))
                .collect(),
            extra_group_object_classes: self
                .extra_group_object_classes
                .into_iter()
                .map(|e| LdapObjectClass::from(e.0))
                .collect(),
        }
    }
}
