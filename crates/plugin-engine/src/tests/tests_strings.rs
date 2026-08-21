use utf16string::{BigEndian, LittleEndian, WString};

use crate::tests::exec_utils::{new_memory_store, run_plugin_init};

fn lua_byte_table(bytes: &[u8]) -> String {
    let values: Vec<String> = bytes.iter().map(u8::to_string).collect();
    format!("{{ {} }}", values.join(", "))
}

#[tokio::test]
async fn test_strings_to_utf8() {
    let input = "hello world";
    let expected = lua_byte_table(input.as_bytes());
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf8("{input}")
            local expected = {expected}
            assert_eq(bytes, expected)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_strings_to_utf16le() {
    let input = "hello world";
    let ws: WString<LittleEndian> = WString::from(input);
    let expected = lua_byte_table(ws.into_bytes().as_slice());
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf16le("{input}")
            local expected = {expected}
            assert_eq(bytes, expected)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_strings_to_utf16be() {
    let input = "hello world";
    let ws: WString<BigEndian> = WString::from(input);
    let expected = lua_byte_table(ws.into_bytes().as_slice());
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf16be("{input}")
            local expected = {expected}
            assert_eq(bytes, expected)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_strings_to_utf16le_vs_utf16be_differ() {
    let script = r#"
        local le = lldap.strings:to_utf16le("A")
        local be = lldap.strings:to_utf16be("A")
        assert_eq(lldap.tables:eq(le, be), false)
        assert_eq(le[1], 65)
        assert_eq(le[2], 0)
        assert_eq(be[1], 0)
        assert_eq(be[2], 65)
    "#;
    let res = run_plugin_init(new_memory_store().await, script).await;
    assert!(res.is_ok(), "{res:?}");
}
