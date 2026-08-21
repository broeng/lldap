// Roundtrip tests for the Lua types that contain `ldap3_proto` types. The serde derives in
// `ldap3_proto` set their shape, thus these tests find changes in that crate.

use ldap3_proto::proto::{
    LdapBindCred, LdapBindRequest, LdapDerefAliases, LdapExtendedRequest, LdapFilter, LdapModify,
    LdapModifyRequest, LdapModifyType, LdapOp, LdapSearchRequest, LdapSearchScope,
};
use ldap3_proto::{LdapPartialAttribute, LdapResultCode, LdapSearchResultEntry};

use crate::api::arguments::ldap_bind_result::BindResult;
use crate::api::arguments::ldap_search_result::SearchResult;
use crate::internal::arguments::bind_request::BindRequestArguments;
use crate::internal::arguments::extended_request::ExtendedRequestArguments;
use crate::internal::arguments::ldap_search_result_entry::LdapSearchResultEntryArguments;
use crate::internal::arguments::modify_request::ModifyRequestArguments;
use crate::internal::arguments::search_result::SearchResultArguments;
use crate::internal::types::datetime::LuaDateTime;
use crate::internal::types::group::LuaGroup;
use crate::internal::types::ldap_search_result::LuaSearchResult;
use crate::internal::types::user::{LuaUser, LuaUserAndGroups};
use crate::tests::roundtrip_utils::assert_roundtrip;

fn sample_datetime() -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2024, 5, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

#[tokio::test]
async fn test_roundtrip_bind_result() {
    let original = BindResult {
        result_code: LdapResultCode::Success,
        message: "ok".to_string(),
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.result_code, "success")
            assert_eq(value.message, "ok")
        "#,
    );

    assert_eq!(result.result_code, original.result_code);
    assert_eq!(result.message, original.message);
}

#[tokio::test]
async fn test_roundtrip_bind_request_arguments() {
    let original = BindRequestArguments {
        bind_result: BindResult {
            result_code: LdapResultCode::Success,
            message: "ok".to_string(),
        },
        bind_request: LdapBindRequest {
            dn: "cn=admin,dc=example,dc=com".to_string(),
            cred: LdapBindCred::Simple("pw".to_string()),
        },
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.bind_result.result_code, "success")
            assert_eq(value.bind_result.message, "ok")
            assert_eq(value.bind_request.dn, "cn=admin,dc=example,dc=com")
            assert_eq(value.bind_request.cred.simple, "pw")
        "#,
    );

    assert_eq!(
        result.bind_result.result_code,
        original.bind_result.result_code
    );
    assert_eq!(result.bind_result.message, original.bind_result.message);
    assert_eq!(result.bind_request, original.bind_request);
}

#[tokio::test]
async fn test_roundtrip_extended_request_arguments() {
    let original = ExtendedRequestArguments {
        extended_result: vec![LdapOp::UnbindRequest],
        extended_request: LdapExtendedRequest {
            name: "1.3.6.1.4.1.4203.1.11.3".to_string(),
            value: None,
        },
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.extended_request.name, "1.3.6.1.4.1.4203.1.11.3")
            assert_eq(value.extended_request.value, LUA_NULL)
            assert_eq(#value.extended_result, 1)
            assert_eq(value.extended_result[1], "unbind_request")
        "#,
    );

    assert_eq!(result.extended_request, original.extended_request);
    assert_eq!(result.extended_result, original.extended_result);
}

#[tokio::test]
async fn test_roundtrip_modify_request_arguments() {
    let original = ModifyRequestArguments {
        modify_result: vec![],
        modify_request: LdapModifyRequest {
            dn: "cn=admin,dc=example,dc=com".to_string(),
            changes: vec![LdapModify {
                operation: LdapModifyType::Replace,
                modification: LdapPartialAttribute {
                    atype: "mail".to_string(),
                    vals: vec!["a@example.com".as_bytes().to_vec()],
                },
            }],
        },
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.modify_request.dn, "cn=admin,dc=example,dc=com")
            assert_eq(#value.modify_request.changes, 1)
            assert_eq(value.modify_request.changes[1].operation, "replace")
            assert_eq(value.modify_request.changes[1].modification.atype, "mail")
            assert_eq(#value.modify_result, 0)
        "#,
    );

    assert_eq!(result.modify_request, original.modify_request);
    assert_eq!(result.modify_result, original.modify_result);
}

#[tokio::test]
async fn test_roundtrip_ldap_search_result_entry_arguments() {
    let original = LdapSearchResultEntryArguments::new(LdapSearchResultEntry {
        dn: "cn=admin,dc=example,dc=com".to_string(),
        attributes: vec![LdapPartialAttribute {
            atype: "cn".to_string(),
            vals: vec!["admin".as_bytes().to_vec()],
        }],
    });

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(value.search_result_entry.dn, "cn=admin,dc=example,dc=com")
            assert_eq(#value.search_result_entry.attributes, 1)
            assert_eq(value.search_result_entry.attributes[1].atype, "cn")
        "#,
    );

    assert_eq!(result.search_result_entry, original.search_result_entry);
}

#[tokio::test]
async fn test_roundtrip_lua_search_result_empty() {
    let original = LuaSearchResult::Empty {};

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(type(value.empty), "table")
        "#,
    );

    assert!(matches!(result, LuaSearchResult::Empty {}));
}

#[tokio::test]
async fn test_roundtrip_lua_search_result_ldap() {
    let original = LuaSearchResult::Ldap(vec![LdapOp::UnbindRequest]);

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(#value.ldap, 1)
            assert_eq(value.ldap[1], "unbind_request")
        "#,
    );

    match result {
        LuaSearchResult::Ldap(ops) => assert_eq!(ops, vec![LdapOp::UnbindRequest]),
        _ => panic!("expected LuaSearchResult::Ldap"),
    }
}

#[tokio::test]
async fn test_roundtrip_lua_search_result_users_and_groups() {
    let user = LuaUser {
        user_id: "alice".to_string(),
        email: "alice@example.com".to_string(),
        display_name: None,
        creation_date: LuaDateTime::from(sample_datetime()),
        uuid: "550e8400-e29b-41d4-a716-446655440020".to_string(),
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime()),
        password_modified_date: LuaDateTime::from(sample_datetime()),
    };
    let group = LuaGroup {
        group_id: 1,
        display_name: "Admins".to_string(),
        creation_date: LuaDateTime::from(sample_datetime()),
        uuid: "550e8400-e29b-41d4-a716-446655440021".to_string(),
        users: vec!["alice".to_string()],
        attributes: vec![],
        modified_date: LuaDateTime::from(sample_datetime()),
    };
    let original = LuaSearchResult::UsersAndGroups {
        users: vec![LuaUserAndGroups { user, groups: None }],
        groups: vec![group],
    };

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(#value.users_and_groups.users, 1)
            assert_eq(value.users_and_groups.users[1].user.user_id, "alice")
            assert_eq(#value.users_and_groups.groups, 1)
            assert_eq(value.users_and_groups.groups[1].display_name, "Admins")
        "#,
    );

    match result {
        LuaSearchResult::UsersAndGroups { users, groups } => {
            assert_eq!(users.len(), 1);
            assert_eq!(users[0].user.user_id, "alice");
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].display_name, "Admins");
        }
        _ => panic!("expected LuaSearchResult::UsersAndGroups"),
    }
}

#[tokio::test]
async fn test_roundtrip_search_result_arguments() {
    let original = SearchResultArguments::new(
        SearchResult::Empty,
        LdapSearchRequest {
            base: "dc=example,dc=com".to_string(),
            scope: LdapSearchScope::Subtree,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Equality("cn".to_string(), "admin".to_string()),
            attrs: vec!["cn".to_string()],
        },
    );

    let result = assert_roundtrip(
        original.clone(),
        r#"
            assert_eq(type(value.search_result.empty), "table")
            assert_eq(value.search_request.base, "dc=example,dc=com")
            assert_eq(value.search_request.scope, "subtree")
            assert_eq(value.search_request.aliases, "never")
            assert_eq(value.search_request.typesonly, false)
            assert_eq(lldap.tables:eq(value.search_request.filter.equality, {"cn", "admin"}), true)
            assert_eq(lldap.tables:eq(value.search_request.attrs, {"cn"}), true)
        "#,
    );

    assert!(matches!(result.search_result, LuaSearchResult::Empty {}));
    assert_eq!(result.search_request, original.search_request);
}
