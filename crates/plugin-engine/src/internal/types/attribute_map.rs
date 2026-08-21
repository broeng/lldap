use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use lldap_domain::types::Attribute;

use crate::internal::types::attributes::LuaAttributeValue;

// Serializes attributes as a table keyed by attribute name, because plugin scripts read
// attributes by name (`t.some_attr.string`).
//
// If two attributes have the same name, only the last one stays. Attribute names are unique
// for each user or group, thus this is not a problem.
pub fn serialize<S: Serializer>(
    attributes: &[Attribute],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let map: BTreeMap<&str, LuaAttributeValue> = attributes
        .iter()
        .map(|attr| (attr.name.as_str(), attr.value.clone().into()))
        .collect();
    map.serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Attribute>, D::Error> {
    let map = BTreeMap::<String, LuaAttributeValue>::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(name, value)| Attribute {
            name: name.into(),
            value: value.value,
        })
        .collect())
}
