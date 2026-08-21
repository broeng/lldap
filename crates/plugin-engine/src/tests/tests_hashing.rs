use digest::Digest;
use md4::Md4;
use md5::Md5;
use sha2::{Sha256, Sha512};

use crate::tests::exec_utils::{new_memory_store, run_plugin_init};

fn compute_hash<D: Digest>(input: &[u8]) -> Vec<u8> {
    let mut hasher = D::new();
    hasher.update(input);
    hasher.finalize().as_slice().to_vec()
}

fn lua_byte_table(bytes: &[u8]) -> String {
    let values: Vec<String> = bytes.iter().map(u8::to_string).collect();
    format!("{{ {} }}", values.join(", "))
}

async fn assert_lua_hash_matches(lua_fn: &str, input: &str, expected: &[u8]) {
    let expected_table = lua_byte_table(expected);
    let expected_hex = base16ct::lower::encode_string(expected);
    let script = format!(
        r#"
            local bytes = lldap.strings:to_utf8("{input}")
            local hash = lldap.hashing:{lua_fn}(bytes)
            local expected = {expected_table}
            assert_eq(hash, expected)

            local hash_str = lldap.encoding:base16_encode(hash)
            local expected_str = "{expected_hex}"
            assert_eq(hash_str, expected_str)
        "#
    );
    let res = run_plugin_init(new_memory_store().await, &script).await;
    assert!(res.is_ok(), "{res:?}");
}

#[tokio::test]
async fn test_hashing_md4_hash_bytes() {
    let input = "hello world";
    let expected = compute_hash::<Md4>(input.as_bytes());
    assert_lua_hash_matches("md4_hash_bytes", input, &expected).await;
}

#[tokio::test]
async fn test_hashing_md5_hash_bytes() {
    let input = "hello world";
    let expected = compute_hash::<Md5>(input.as_bytes());
    assert_lua_hash_matches("md5_hash_bytes", input, &expected).await;
}

#[tokio::test]
async fn test_hashing_sha256_hash_bytes() {
    let input = "hello world";
    let expected = compute_hash::<Sha256>(input.as_bytes());
    assert_lua_hash_matches("sha256_hash_bytes", input, &expected).await;
}

#[tokio::test]
async fn test_hashing_sha512_hash_bytes() {
    let input = "hello world";
    let expected = compute_hash::<Sha512>(input.as_bytes());
    assert_lua_hash_matches("sha512_hash_bytes", input, &expected).await;
}
