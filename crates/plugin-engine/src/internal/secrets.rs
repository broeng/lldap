use ldap3_proto::{
    proto::{
        LdapBindCred, LdapBindRequest, LdapExtendedRequest, LdapModify, LdapModifyRequest,
        OID_PASSWORD_MODIFY,
    },
    LdapPartialAttribute,
};

pub const USER_PASS_KEY: &str = "userpassword";

pub fn strip_bind_password(bind_request: LdapBindRequest) -> LdapBindRequest {
    LdapBindRequest {
        dn: bind_request.dn,
        cred: match bind_request.cred {
            LdapBindCred::SASL(c) => LdapBindCred::SASL(c),
            LdapBindCred::Simple(_) => LdapBindCred::Simple("".to_string()),
        },
    }
}

pub fn strip_modify_password(modify_request: LdapModifyRequest) -> LdapModifyRequest {
    LdapModifyRequest {
        dn: modify_request.dn,
        changes: modify_request
            .changes
            .into_iter()
            .map(|c| {
                if c.modification.atype.eq_ignore_ascii_case(USER_PASS_KEY) {
                    LdapModify {
                        operation: c.operation,
                        modification: LdapPartialAttribute {
                            atype: c.modification.atype,
                            vals: Vec::new(),
                        },
                    }
                } else {
                    c
                }
            })
            .collect(),
    }
}

pub fn strip_change_password_extended(
    extended_request: LdapExtendedRequest,
) -> LdapExtendedRequest {
    let is_password_modify = extended_request
        .name
        .eq_ignore_ascii_case(OID_PASSWORD_MODIFY);
    LdapExtendedRequest {
        name: extended_request.name,
        value: if is_password_modify {
            None
        } else {
            extended_request.value
        },
    }
}
