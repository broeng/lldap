use crate::tests::exec_utils::{new_memory_store, run_plugin_init};

#[tokio::test]
async fn test_lib_tables_empty() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            assert_eq(lldap.tables:empty({}), true)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_tables_not_empty() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local t = { a = 1 }
            assert_eq(lldap.tables:empty(t), false)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_tables_has_subtree_top_level_match() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local base = { a = 1, b = 2 }
            local tree = { a = 1, b = 2 }
            assert_eq(lldap.tables:has_subtree(base, tree), true)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_tables_has_subtree_nested_match() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local base = {
                a = 1,
                child = {
                    x = { y = 1, z = 2 },
                    other = "value",
                },
            }
            local tree = { y = 1, z = 2 }
            assert_eq(lldap.tables:has_subtree(base, tree), true)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_tables_has_subtree_no_match() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local base = {
                a = 1,
                child = { x = 1, y = 2 },
            }
            local tree = { x = 1, y = 3 }
            assert_eq(lldap.tables:has_subtree(base, tree), false)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_tables_has_subtree_partial_no_match() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local base = { a = 1, b = 2, c = 3 }
            local tree = { a = 1, b = 2 }
            assert_eq(lldap.tables:has_subtree(base, tree), false)
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_log_methods_do_not_error() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            lldap.log:debug("debug message")
            lldap.log:info("info message")
            lldap.log:warn("warn message")
        "#,
    )
    .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_lib_strings_split() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            local splitted = lldap.strings:split("hello world", " ")
            local expected = { "hello", "world" }
            assert_eq(lldap.tables:eq(splitted, expected), true)
        "#,
    )
    .await;
    assert!(res.is_ok());
}
