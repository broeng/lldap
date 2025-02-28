use ldap3_proto::proto::{
    LdapBindCred, LdapBindRequest, LdapExtendedRequest, LdapModifyRequest,
    LdapPasswordModifyRequest,
};

use crate::internal::secrets::USER_PASS_KEY;

#[derive(Clone)]
pub struct Secret {
    pub secret: Vec<u8>,
}

impl Secret {
    pub fn try_from_extended_request(extended_request: &LdapExtendedRequest) -> Option<Secret> {
        match LdapPasswordModifyRequest::try_from(extended_request) {
            Ok(password_request) => password_request.new_password.map(|p| Secret {
                secret: p.as_bytes().to_vec(),
            }),
            Err(_) => None,
        }
    }

    pub fn try_from_modify_request(modify_request: &LdapModifyRequest) -> Option<Secret> {
        match (&modify_request.changes)
            .into_iter()
            .find(|c| c.modification.atype.eq_ignore_ascii_case(USER_PASS_KEY))
        {
            Some(change) => {
                if let [value] = change.modification.vals.as_slice() {
                    Some(Secret {
                        secret: value.clone(),
                    })
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn try_from_bind_request(bind_request: &LdapBindRequest) -> Option<Secret> {
        match &bind_request.cred {
            LdapBindCred::Simple(s) => Some(Secret {
                secret: s.as_bytes().to_vec(),
            }),
            LdapBindCred::SASL(_) => None,
        }
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Secret {
            secret: value.as_bytes().to_vec(),
        }
    }
}
