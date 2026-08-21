use lldap_domain::types::{AttributeValue, Cardinality, JpegPhoto};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Result as LuaResult, Value};
use serde::{Deserialize, Serialize};

use crate::internal::types::datetime::LuaDateTime;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LuaAttributeValue {
    pub value: AttributeValue,
}

impl From<AttributeValue> for LuaAttributeValue {
    fn from(value: AttributeValue) -> Self {
        LuaAttributeValue { value }
    }
}

// A flat copy of `AttributeValue`, which has two levels of enums. Serde serializes this enum
// as `{string = "x"}`, which is the shape that plugin scripts use. The serde derive on
// `AttributeValue` gives `{String = {Singleton = "x"}}`.
#[derive(Serialize, Deserialize)]
enum LuaAttributeValueRepr {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "strings")]
    Strings(Vec<String>),
    #[serde(rename = "int")]
    Int(i64),
    #[serde(rename = "ints")]
    Ints(Vec<i64>),
    #[serde(rename = "datetime")]
    DateTime(LuaDateTime),
    #[serde(rename = "datetimes")]
    DateTimes(Vec<LuaDateTime>),
    #[serde(rename = "jpeg_photo")]
    JpegPhoto(Vec<u8>),
    #[serde(rename = "jpeg_photos")]
    JpegPhotos(Vec<Vec<u8>>),
}

impl From<AttributeValue> for LuaAttributeValueRepr {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(Cardinality::Singleton(s)) => LuaAttributeValueRepr::String(s),
            AttributeValue::String(Cardinality::Unbounded(l)) => LuaAttributeValueRepr::Strings(l),
            AttributeValue::Integer(Cardinality::Singleton(i)) => LuaAttributeValueRepr::Int(i),
            AttributeValue::Integer(Cardinality::Unbounded(l)) => LuaAttributeValueRepr::Ints(l),
            AttributeValue::DateTime(Cardinality::Singleton(dt)) => {
                LuaAttributeValueRepr::DateTime(dt.into())
            }
            AttributeValue::DateTime(Cardinality::Unbounded(l)) => {
                LuaAttributeValueRepr::DateTimes(l.into_iter().map(LuaDateTime::from).collect())
            }
            AttributeValue::JpegPhoto(Cardinality::Singleton(p)) => {
                LuaAttributeValueRepr::JpegPhoto(p.into_bytes())
            }
            AttributeValue::JpegPhoto(Cardinality::Unbounded(l)) => {
                LuaAttributeValueRepr::JpegPhotos(
                    l.into_iter().map(JpegPhoto::into_bytes).collect(),
                )
            }
        }
    }
}

impl TryFrom<LuaAttributeValueRepr> for AttributeValue {
    type Error = anyhow::Error;

    fn try_from(repr: LuaAttributeValueRepr) -> Result<Self, Self::Error> {
        Ok(match repr {
            LuaAttributeValueRepr::String(s) => AttributeValue::String(Cardinality::Singleton(s)),
            LuaAttributeValueRepr::Strings(l) => AttributeValue::String(Cardinality::Unbounded(l)),
            LuaAttributeValueRepr::Int(i) => AttributeValue::Integer(Cardinality::Singleton(i)),
            LuaAttributeValueRepr::Ints(l) => AttributeValue::Integer(Cardinality::Unbounded(l)),
            LuaAttributeValueRepr::DateTime(dt) => {
                AttributeValue::DateTime(Cardinality::Singleton(dt.datetime))
            }
            LuaAttributeValueRepr::DateTimes(l) => AttributeValue::DateTime(
                Cardinality::Unbounded(l.into_iter().map(|dt| dt.datetime).collect()),
            ),
            LuaAttributeValueRepr::JpegPhoto(bytes) => AttributeValue::JpegPhoto(
                Cardinality::Singleton(JpegPhoto::try_from(bytes.as_slice())?),
            ),
            LuaAttributeValueRepr::JpegPhotos(byte_lists) => {
                let mut photos = Vec::with_capacity(byte_lists.len());
                for bytes in byte_lists {
                    photos.push(JpegPhoto::try_from(bytes.as_slice())?);
                }
                AttributeValue::JpegPhoto(Cardinality::Unbounded(photos))
            }
        })
    }
}

impl Serialize for LuaAttributeValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LuaAttributeValueRepr::from(self.value.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LuaAttributeValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = LuaAttributeValueRepr::deserialize(deserializer)?;
        Ok(LuaAttributeValue {
            value: AttributeValue::try_from(repr).map_err(serde::de::Error::custom)?,
        })
    }
}

impl IntoLua for LuaAttributeValue {
    fn into_lua(self, lua: &Lua) -> LuaResult<Value> {
        lua.to_value(&self)
    }
}

impl FromLua for LuaAttributeValue {
    fn from_lua(value: Value, lua: &Lua) -> LuaResult<Self> {
        lua.from_value(value)
    }
}
