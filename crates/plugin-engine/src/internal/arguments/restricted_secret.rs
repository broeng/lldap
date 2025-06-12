use mlua::UserData;

use digest::Digest;
use md4::Md4;
use utf16string::{LittleEndian, WString};

use crate::{
    api::permissions::SecretPermissions,
    internal::types::{result::MyLuaResult, secret::Secret},
};

#[derive(Clone)]
pub struct RestrictedSecret {
    pub secret: Secret,
    pub permissions: SecretPermissions,
}

impl RestrictedSecret {
    pub fn new(secret: Secret) -> Self {
        Self {
            secret,
            permissions: SecretPermissions::Deny,
        }
    }

    pub fn has_any_permission(&self, permissions: Vec<SecretPermissions>) -> bool {
        permissions.contains(&self.permissions)
    }

    pub fn with_permissions(&self, perm: SecretPermissions) -> Self {
        Self {
            secret: self.secret.clone(),
            permissions: perm,
        }
    }
}

impl UserData for RestrictedSecret {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("to_ntlm_hash", |_, secret, ()| {
            if secret.has_any_permission(vec![
                SecretPermissions::AllowAnyHash,
                SecretPermissions::AllowNtlmHash,
            ]) {
                Ok(MyLuaResult(
                    match String::from_utf8(secret.secret.secret.clone()) {
                        Ok(s) => {
                            let ws: WString<LittleEndian> = WString::from(&s);
                            let hash = hash_bytes::<Md4>(ws.into_bytes().as_slice());
                            Ok(base16ct::lower::encode_string(hash.as_slice()).to_uppercase())
                        }
                        Err(_) => Err("invalid utf8 string".to_string()),
                    },
                ))
            } else {
                Ok(MyLuaResult(Err(
                    "plugin has been granted insufficient permissions".to_string(),
                )))
            }
        });
    }
}

fn hash_bytes<D: Digest>(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = D::new();
    hasher.update(bytes);
    hasher.finalize().as_slice().to_vec()
}
