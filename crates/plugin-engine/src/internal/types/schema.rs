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
    #[serde(rename = "user_attributes")]
    pub user_attributes: LuaAttributeList,
    #[serde(rename = "group_attributes")]
    pub group_attributes: LuaAttributeList,
    #[serde(rename = "extra_user_object_classes")]
    pub extra_user_object_classes: BTreeMap<String, bool>,
    #[serde(rename = "extra_group_object_classes")]
    pub extra_group_object_classes: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToTypename)]
pub struct LuaAttributeSchema {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "attribute_type")]
    pub attribute_type: AttributeType,
    #[serde(rename = "is_list")]
    pub is_list: bool,
    #[serde(rename = "is_visible")]
    pub is_visible: bool,
    #[serde(rename = "is_editable")]
    pub is_editable: bool,
    #[serde(rename = "is_hardcoded")]
    pub is_hardcoded: bool,
    #[serde(rename = "is_readonly")]
    pub is_readonly: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToTypename)]
pub struct LuaAttributeList {
    #[serde(rename = "attributes")]
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

impl From<LuaAttributeSchema> for AttributeSchema {
    fn from(value: LuaAttributeSchema) -> Self {
        AttributeSchema {
            name: AttributeName::from(value.name),
            attribute_type: value.attribute_type,
            is_list: value.is_list,
            is_visible: value.is_visible,
            is_editable: value.is_editable,
            is_hardcoded: value.is_hardcoded,
            is_readonly: value.is_readonly,
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

impl From<LuaAttributeList> for AttributeList {
    fn from(value: LuaAttributeList) -> Self {
        AttributeList {
            attributes: value.attributes.into_iter().map(|e| e.1.into()).collect(),
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

impl From<LuaSchema> for Schema {
    fn from(value: LuaSchema) -> Self {
        Schema {
            user_attributes: value.user_attributes.into(),
            group_attributes: value.group_attributes.into(),
            extra_user_object_classes: value
                .extra_user_object_classes
                .into_iter()
                .map(|e| LdapObjectClass::from(e.0))
                .collect(),
            extra_group_object_classes: value
                .extra_group_object_classes
                .into_iter()
                .map(|e| LdapObjectClass::from(e.0))
                .collect(),
        }
    }
}
