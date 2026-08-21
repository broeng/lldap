use base64ct::{Base64, Encoding};

use crate::tests::exec_utils::{new_memory_store, run_plugin_init};

fn lua_byte_table(bytes: &[u8]) -> String {
    let values: Vec<String> = bytes.iter().map(u8::to_string).collect();
    format!("{{ {} }}", values.join(", "))
}

#[tokio::test]
async fn test_encoding_base64_encode() {
    let input = "hello world";
    let expected = Base64::encode_string(input.as_bytes());
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf8("{input}")
            local encoded = lldap.encoding:base64_encode(bytes)
            assert_eq(encoded, "{expected}")
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_encoding_base64_decode() {
    let input = "hello world";
    let encoded = Base64::encode_string(input.as_bytes());
    let expected_table = lua_byte_table(input.as_bytes());
    let script = format!(
        r#"
            local decoded = lldap.encoding:base64_decode("{encoded}")
            local expected = {expected_table}
            assert_eq(decoded, expected)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_encoding_base64_roundtrip() {
    let input = "The quick brown fox jumps over the lazy dog";
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf8("{input}")
            local encoded = lldap.encoding:base64_encode(bytes)
            local decoded = lldap.encoding:base64_decode(encoded)
            assert_eq(decoded, bytes)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_encoding_base64_decode_invalid_input_errors() {
    let script = r#"
        lldap.encoding:base64_decode("not-valid-base64!!")
    "#;
    let res = run_plugin_init(new_memory_store().await, script).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_encoding_base16_encode() {
    let input = "hello world";
    let expected = base16ct::lower::encode_string(input.as_bytes());
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf8("{input}")
            local encoded = lldap.encoding:base16_encode(bytes)
            assert_eq(encoded, "{expected}")
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_encoding_base16_encode_empty_bytes() {
    let script = r#"
        local encoded = lldap.encoding:base16_encode({})
        assert_eq(encoded, "")
    "#;
    let res = run_plugin_init(new_memory_store().await, script).await;
    assert!(res.is_ok(), "{res:?}");
}
