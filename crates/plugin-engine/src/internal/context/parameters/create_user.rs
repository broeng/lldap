use lldap_domain::types::{Attribute, Email, UserId};
use lldap_domain_handlers::requests::CreateUserRequest;
use mlua::{Result as LuaResult, Table};

use super::utils;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateUserParams {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub attributes: Vec<Attribute>,
}
impl CreateUserParams {
    pub fn from(args: &Table) -> LuaResult<Self> {
        Ok(CreateUserParams {
            user_id: args.get("user_id")?,
            email: args.get("email")?,
            display_name: args.get("display_name")?,
            attributes: utils::get_opt_attrmap("attributes", args)?,
        })
    }
}

impl From<CreateUserParams> for CreateUserRequest {
    fn from(value: CreateUserParams) -> Self {
        CreateUserRequest {
            user_id: UserId::from(value.user_id),
            email: Email::from(value.email),
            display_name: value.display_name.into(),
            attributes: value.attributes,
        }
    }
}
