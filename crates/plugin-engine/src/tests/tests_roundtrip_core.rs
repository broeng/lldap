// Roundtrip tests for the core Lua types in `internal::types`. Each test examines the value in
// Lua, and compares the returned value with the original. This makes sure that the shape that
// plugin scripts see does not change.

use lldap_domain::types::{
    Attribute, AttributeName, AttributeType, AttributeValue, Cardinality, JpegPhoto,
};

use crate::internal::types::attributes::LuaAttributeValue;
use crate::internal::types::datetime::{LuaDateTime, datetime_to_rfc3339};
use crate::internal::types::group::{LuaGroup, LuaGroupDetails, LuaGroupsVec};
use crate::internal::types::schema::{LuaAttributeList, LuaAttributeSchema, LuaSchema};
use crate::internal::types::user::{LuaUser, LuaUserAndGroups, LuaUserAndGroupsVec};

use crate::tests::roundtrip_utils::assert_roundtrip;

fn sample_datetime(day: u32) -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2024, 3, day)
        .unwrap()
        .and_hms_opt(12, 30, 45)
        .unwrap()
}

fn single_attribute() -> Attribute {
    Attribute {
        name: AttributeName::new("nickname"),
        value: AttributeValue::String(Cardinality::Singleton("Bob".to_string())),
    }
}

#[tokio::test]
async fn test_roundtrip_lua_datetime() {
    let dt = sample_datetime(15);
    let expected = datetime_to_rfc3339(&dt);
    let original = LuaDateTime::from(dt);

    let script = format!(r#"assert_eq(value, "{expected}")"#);
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result.datetime, dt);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_string_singleton() {
    let original: LuaAttributeValue =
        AttributeValue::String(Cardinality::Singleton("Hello, world!".to_string())).into();

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.string, "Hello, world!")
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_string_unbounded() {
    let original: LuaAttributeValue = AttributeValue::String(Cardinality::Unbounded(vec![
        "a".to_string(),
        "b".to_string(),
    ]))
    .into();

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(lldap.tables:eq(value.strings, {"a", "b"}), true)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_integer_singleton() {
    let original: LuaAttributeValue = AttributeValue::Integer(Cardinality::Singleton(42)).into();

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.int, 42)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_integer_unbounded() {
    let original: LuaAttributeValue =
        AttributeValue::Integer(Cardinality::Unbounded(vec![1, 2, 3])).into();

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(lldap.tables:eq(value.ints, {1, 2, 3}), true)
        "#,
    );

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_datetime_singleton() {
    let dt = sample_datetime(20);
    let expected = datetime_to_rfc3339(&dt);
    let original: LuaAttributeValue = AttributeValue::DateTime(Cardinality::Singleton(dt)).into();

    let script = format!(r#"assert_eq(value.datetime, "{expected}")"#);
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_datetime_unbounded() {
    let dt1 = sample_datetime(1);
    let dt2 = sample_datetime(2);
    let expected1 = datetime_to_rfc3339(&dt1);
    let expected2 = datetime_to_rfc3339(&dt2);
    let original: LuaAttributeValue =
        AttributeValue::DateTime(Cardinality::Unbounded(vec![dt1, dt2])).into();

    let script = format!(
        r#"assert_eq(lldap.tables:eq(value.datetimes, {{"{expected1}", "{expected2}"}}), true)"#
    );
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result, original);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_value_jpeg_photo_singleton() {
    let photo = JpegPhoto::try_from(Vec::<u8>::new()).unwrap();
    let original: LuaAttributeValue =
        AttributeValue::JpegPhoto(Cardinality::Singleton(photo)).into();

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(lldap.tables:eq(value.jpeg_photo, {}), true)
        "#,
    );

    assert_eq!(result, original);
}

// `attribute_map` is not a type, thus this wrapper gives `assert_roundtrip` a type to send.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct AttributesForTest(#[serde(with = "crate::internal::types::attribute_map")] Vec<Attribute>);

impl mlua::IntoLua for AttributesForTest {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        use mlua::LuaSerdeExt;
        lua.to_value(&self)
    }
}

impl mlua::FromLua for AttributesForTest {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        use mlua::LuaSerdeExt;
        lua.from_value(value)
    }
}

#[tokio::test]
async fn test_roundtrip_attribute_map_argument() {
    let dt = sample_datetime(9);
    let expected_dt = datetime_to_rfc3339(&dt);
    let photo = JpegPhoto::try_from(Vec::<u8>::new()).unwrap();
    let original = AttributesForTest(vec![
        Attribute {
            name: AttributeName::new("nickname"),
            value: AttributeValue::String(Cardinality::Singleton("Bob".to_string())),
        },
        Attribute {
            name: AttributeName::new("aliases"),
            value: AttributeValue::String(Cardinality::Unbounded(vec![
                "Bobby".to_string(),
                "Rob".to_string(),
            ])),
        },
        Attribute {
            name: AttributeName::new("age"),
            value: AttributeValue::Integer(Cardinality::Singleton(42)),
        },
        Attribute {
            name: AttributeName::new("lucky_numbers"),
            value: AttributeValue::Integer(Cardinality::Unbounded(vec![7, 13])),
        },
        Attribute {
            name: AttributeName::new("birthday"),
            value: AttributeValue::DateTime(Cardinality::Singleton(dt)),
        },
        Attribute {
            name: AttributeName::new("avatar"),
            value: AttributeValue::JpegPhoto(Cardinality::Singleton(photo)),
        },
    ]);

    let script = format!(
        r#"
            assert_eq(value.nickname.string, "Bob")
            assert_eq(lldap.tables:eq(value.aliases.strings, {{"Bobby", "Rob"}}), true)
            assert_eq(value.age.int, 42)
            assert_eq(lldap.tables:eq(value.lucky_numbers.ints, {{7, 13}}), true)
            assert_eq(value.birthday.datetime, "{expected_dt}")
            assert_eq(lldap.tables:eq(value.avatar.jpeg_photo, {{}}), true)
        "#
    );
    let result = assert_roundtrip(original.clone(), &script);

    let mut original_sorted = original.0.clone();
    let mut result_sorted = result.0.clone();
    original_sorted.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
    result_sorted.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
    assert_eq!(result_sorted, original_sorted);
}

#[tokio::test]
async fn test_roundtrip_lua_group_details() {
    let creation_date = sample_datetime(1);
    let modified_date = sample_datetime(2);
    let expected_creation = datetime_to_rfc3339(&creation_date);
    let expected_modified = datetime_to_rfc3339(&modified_date);

    let original = LuaGroupDetails {
        group_id: 42,
        display_name: "Test Group".to_string(),
        creation_date: LuaDateTime::from(creation_date),
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        attributes: vec![single_attribute()],
        modified_date: LuaDateTime::from(modified_date),
    };

    let script = format!(
        r#"
            assert_eq(value.group_id, 42)
            assert_eq(value.display_name, "Test Group")
            assert_eq(value.creation_date, "{expected_creation}")
            assert_eq(value.uuid, "550e8400-e29b-41d4-a716-446655440000")
            assert_eq(value.attributes.nickname.string, "Bob")
            assert_eq(value.modified_date, "{expected_modified}")
        "#
    );
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result.group_id, original.group_id);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(
        result.creation_date.datetime,
        original.creation_date.datetime
    );
    assert_eq!(result.uuid, original.uuid);
    assert_eq!(result.attributes, original.attributes);
    assert_eq!(
        result.modified_date.datetime,
        original.modified_date.datetime
    );
}

#[tokio::test]
async fn test_roundtrip_lua_group() {
    let creation_date = sample_datetime(3);
    let modified_date = sample_datetime(4);
    let expected_creation = datetime_to_rfc3339(&creation_date);
    let expected_modified = datetime_to_rfc3339(&modified_date);

    let original = LuaGroup {
        group_id: 7,
        display_name: "Another Group".to_string(),
        creation_date: LuaDateTime::from(creation_date),
        uuid: "550e8400-e29b-41d4-a716-446655440001".to_string(),
        users: vec!["alice".to_string(), "bob".to_string()],
        attributes: vec![single_attribute()],
        modified_date: LuaDateTime::from(modified_date),
    };

    let script = format!(
        r#"
            assert_eq(value.group_id, 7)
            assert_eq(value.display_name, "Another Group")
            assert_eq(value.creation_date, "{expected_creation}")
            assert_eq(value.uuid, "550e8400-e29b-41d4-a716-446655440001")
            assert_eq(lldap.tables:eq(value.users, {{"alice", "bob"}}), true)
            assert_eq(value.attributes.nickname.string, "Bob")
            assert_eq(value.modified_date, "{expected_modified}")
        "#
    );
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result.group_id, original.group_id);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(
        result.creation_date.datetime,
        original.creation_date.datetime
    );
    assert_eq!(result.uuid, original.uuid);
    assert_eq!(result.users, original.users);
    assert_eq!(result.attributes, original.attributes);
    assert_eq!(
        result.modified_date.datetime,
        original.modified_date.datetime
    );
}

#[tokio::test]
async fn test_roundtrip_lua_groups_vec() {
    let group = LuaGroup {
        group_id: 1,
        display_name: "G".to_string(),
        creation_date: LuaDateTime::from(sample_datetime(5)),
        uuid: "550e8400-e29b-41d4-a716-446655440002".to_string(),
        users: vec!["alice".to_string()],
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(5)),
    };
    let original = LuaGroupsVec {
        groups: vec![group],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(#value.groups, 1)
            assert_eq(value.groups[1].display_name, "G")
        "#,
    );

    assert_eq!(result.groups.len(), original.groups.len());
    assert_eq!(
        result.groups[0].display_name,
        original.groups[0].display_name
    );
    assert_eq!(result.groups[0].group_id, original.groups[0].group_id);
}

#[tokio::test]
async fn test_roundtrip_lua_user() {
    let creation_date = sample_datetime(6);
    let modified_date = sample_datetime(7);
    let password_modified_date = sample_datetime(8);
    let expected_creation = datetime_to_rfc3339(&creation_date);
    let expected_modified = datetime_to_rfc3339(&modified_date);
    let expected_password_modified = datetime_to_rfc3339(&password_modified_date);

    let original = LuaUser {
        user_id: "alice".to_string(),
        email: "alice@example.com".to_string(),
        display_name: Some("Alice Wonderland".to_string()),
        creation_date: LuaDateTime::from(creation_date),
        uuid: "550e8400-e29b-41d4-a716-446655440003".to_string(),
        attributes: vec![single_attribute()],
        modified_date: LuaDateTime::from(modified_date),
        password_modified_date: LuaDateTime::from(password_modified_date),
    };

    let script = format!(
        r#"
            assert_eq(value.user_id, "alice")
            assert_eq(value.email, "alice@example.com")
            assert_eq(value.display_name, "Alice Wonderland")
            assert_eq(value.creation_date, "{expected_creation}")
            assert_eq(value.uuid, "550e8400-e29b-41d4-a716-446655440003")
            assert_eq(value.attributes.nickname.string, "Bob")
            assert_eq(value.modified_date, "{expected_modified}")
            assert_eq(value.password_modified_date, "{expected_password_modified}")
        "#
    );
    let result = assert_roundtrip(original.clone(), &script);

    assert_eq!(result.user_id, original.user_id);
    assert_eq!(result.email, original.email);
    assert_eq!(result.display_name, original.display_name);
    assert_eq!(
        result.creation_date.datetime,
        original.creation_date.datetime
    );
    assert_eq!(result.uuid, original.uuid);
    assert_eq!(result.attributes, original.attributes);
    assert_eq!(
        result.modified_date.datetime,
        original.modified_date.datetime
    );
    assert_eq!(
        result.password_modified_date.datetime,
        original.password_modified_date.datetime
    );
}

#[tokio::test]
async fn test_roundtrip_lua_user_no_display_name() {
    let original = LuaUser {
        user_id: "bob".to_string(),
        email: "bob@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime(10)),
        uuid: "550e8400-e29b-41d4-a716-446655440004".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(10)),
        password_modified_date: LuaDateTime::from(sample_datetime(10)),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.display_name, nil)
        "#,
    );

    assert_eq!(result.display_name, None);
}

#[tokio::test]
async fn test_roundtrip_lua_user_and_groups() {
    let group = LuaGroupDetails {
        group_id: 1,
        display_name: "G".to_string(),
        creation_date: LuaDateTime::from(sample_datetime(11)),
        uuid: "550e8400-e29b-41d4-a716-446655440005".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(11)),
    };
    let user = LuaUser {
        user_id: "carol".to_string(),
        email: "carol@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime(12)),
        uuid: "550e8400-e29b-41d4-a716-446655440006".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(12)),
        password_modified_date: LuaDateTime::from(sample_datetime(12)),
    };
    let original = LuaUserAndGroups {
        user,
        groups: Some(vec![group]),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user.user_id, "carol")
            assert_eq(#value.groups, 1)
            assert_eq(value.groups[1].display_name, "G")
        "#,
    );

    assert_eq!(result.user.user_id, original.user.user_id);
    assert_eq!(
        result.groups.map(|gs| gs.len()),
        original.groups.map(|gs| gs.len())
    );
}

#[tokio::test]
async fn test_roundtrip_lua_user_and_groups_not_fetched() {
    let user = LuaUser {
        user_id: "dave".to_string(),
        email: "dave@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime(13)),
        uuid: "550e8400-e29b-41d4-a716-446655440007".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(13)),
        password_modified_date: LuaDateTime::from(sample_datetime(13)),
    };
    let original = LuaUserAndGroups { user, groups: None };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.groups, nil)
        "#,
    );

    assert!(result.groups.is_none());
}

#[tokio::test]
async fn test_roundtrip_lua_user_and_groups_fetched_but_empty() {
    // Zero groups must stay different from "not fetched".
    let user = LuaUser {
        user_id: "frank".to_string(),
        email: "frank@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime(13)),
        uuid: "550e8400-e29b-41d4-a716-446655440009".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(13)),
        password_modified_date: LuaDateTime::from(sample_datetime(13)),
    };
    let original = LuaUserAndGroups {
        user,
        groups: Some(vec![]),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.groups ~= nil, true)
            assert_eq(#value.groups, 0)
        "#,
    );

    assert!(result.groups.is_some());
    assert_eq!(result.groups.unwrap().len(), 0);
}

#[tokio::test]
async fn test_roundtrip_lua_user_and_groups_vec() {
    let user = LuaUser {
        user_id: "erin".to_string(),
        email: "erin@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime(14)),
        uuid: "550e8400-e29b-41d4-a716-446655440008".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime(14)),
        password_modified_date: LuaDateTime::from(sample_datetime(14)),
    };
    let original = LuaUserAndGroupsVec {
        user_and_groups: vec![LuaUserAndGroups { user, groups: None }],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(#value.user_and_groups, 1)
            assert_eq(value.user_and_groups[1].user.user_id, "erin")
        "#,
    );

    assert_eq!(result.user_and_groups.len(), original.user_and_groups.len());
    assert_eq!(
        result.user_and_groups[0].user.user_id,
        original.user_and_groups[0].user.user_id
    );
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_schema() {
    let original = LuaAttributeSchema {
        name: "nickname".to_string(),
        attribute_type: AttributeType::String,
        is_list: false,
        is_visible: true,
        is_editable: true,
        is_hardcoded: false,
        is_readonly: false,
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.name, "nickname")
            assert_eq(value.attribute_type, "String")
            assert_eq(value.is_list, false)
            assert_eq(value.is_visible, true)
            assert_eq(value.is_editable, true)
            assert_eq(value.is_hardcoded, false)
            assert_eq(value.is_readonly, false)
        "#,
    );

    assert_eq!(result.name, original.name);
    assert_eq!(result.attribute_type, original.attribute_type);
    assert_eq!(result.is_list, original.is_list);
    assert_eq!(result.is_visible, original.is_visible);
    assert_eq!(result.is_editable, original.is_editable);
    assert_eq!(result.is_hardcoded, original.is_hardcoded);
    assert_eq!(result.is_readonly, original.is_readonly);
}

#[tokio::test]
async fn test_roundtrip_lua_attribute_list() {
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        "nickname".to_string(),
        LuaAttributeSchema {
            name: "nickname".to_string(),
            attribute_type: AttributeType::String,
            is_list: false,
            is_visible: true,
            is_editable: true,
            is_hardcoded: false,
            is_readonly: false,
        },
    );
    let original = LuaAttributeList { attributes };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.attributes.nickname.name, "nickname")
            assert_eq(value.attributes.nickname.attribute_type, "String")
        "#,
    );

    assert_eq!(result.attributes.len(), original.attributes.len());
    assert_eq!(
        result.attributes.get("nickname").map(|a| &a.name),
        original.attributes.get("nickname").map(|a| &a.name)
    );
}

#[tokio::test]
async fn test_roundtrip_lua_schema() {
    let mut user_attributes = std::collections::BTreeMap::new();
    user_attributes.insert(
        "nickname".to_string(),
        LuaAttributeSchema {
            name: "nickname".to_string(),
            attribute_type: AttributeType::String,
            is_list: false,
            is_visible: true,
            is_editable: true,
            is_hardcoded: false,
            is_readonly: false,
        },
    );
    let group_attributes = std::collections::BTreeMap::new();
    let mut extra_user_object_classes = std::collections::BTreeMap::new();
    extra_user_object_classes.insert("customUserClass".to_string(), true);
    let mut extra_group_object_classes = std::collections::BTreeMap::new();
    extra_group_object_classes.insert("customGroupClass".to_string(), true);

    let original = LuaSchema {
        user_attributes: LuaAttributeList {
            attributes: user_attributes,
        },
        group_attributes: LuaAttributeList {
            attributes: group_attributes,
        },
        extra_user_object_classes,
        extra_group_object_classes,
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.user_attributes.attributes.nickname.name, "nickname")
            assert_eq(lldap.tables:eq(value.group_attributes.attributes, {}), true)
            assert_eq(value.extra_user_object_classes.customUserClass, true)
            assert_eq(value.extra_group_object_classes.customGroupClass, true)
        "#,
    );

    assert_eq!(
        result.user_attributes.attributes.len(),
        original.user_attributes.attributes.len()
    );
    assert_eq!(
        result.extra_user_object_classes,
        original.extra_user_object_classes
    );
    assert_eq!(
        result.extra_group_object_classes,
        original.extra_group_object_classes
    );
}
