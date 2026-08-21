use crate::tests::exec_utils::{new_memory_store, run_plugin_init};

#[tokio::test]
async fn test_init01_can_initialize_without_error() {
    let res = run_plugin_init(new_memory_store().await, r#""#).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_init02_init_can_trigger_failure() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            error("Trigger failure", 1)
        "#,
    )
    .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_init03_assert_eq_can_trigger_failure() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            assert_eq("a", "b")
        "#,
    )
    .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_init04_assert_eq_checks_equality() {
    let res = run_plugin_init(
        new_memory_store().await,
        r#"
            assert_eq("a", "a")
            assert_eq("Hello, world!", "Hello, world!")
            assert_eq(true, true)
            assert_eq(false, false)
            assert_eq(42, 42)
            assert_eq(nil, nil)
            assert_eq({}, {})
            assert_eq({ a = "b" }, { a = "b" })
        "#,
    )
    .await;
    assert!(res.is_ok());
}
