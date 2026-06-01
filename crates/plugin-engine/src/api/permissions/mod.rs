use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretPermissions {
    #[serde(rename = "DENY")]
    Deny,
    #[serde(rename = "ALLOW-ANY-HASH")]
    AllowAnyHash,
    #[serde(rename = "ALLOW-NTLM-HASH")]
    AllowNtlmHash,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub secrets: SecretPermissions,
}

impl std::default::Default for Permissions {
    fn default() -> Self {
        Permissions {
            secrets: SecretPermissions::Deny,
        }
    }
}
