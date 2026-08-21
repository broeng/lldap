use mlua::{Result as LuaResult, Table};

use lldap_domain::types::{Attribute, AttributeName, Email, UserId};
use lldap_domain_handlers::requests::UpdateUserRequest;

use crate::internal::context::parameters::utils::{get_opt_arg, get_opt_attrmap, get_opt_vec};

#[derive(Debug, Clone)]
pub struct UpdateUserParams {
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub delete_attributes: Vec<String>,
    pub insert_attributes: Vec<Attribute>,
}

impl UpdateUserParams {
    pub fn from(args: &Table) -> LuaResult<Self> {
        Ok(UpdateUserParams {
            user_id: args.get("user_id")?,
            email: get_opt_arg("email", args)?,
            display_name: get_opt_arg("display_name", args)?,
            delete_attributes: get_opt_vec("delete_attributes", args)?,
            insert_attributes: get_opt_attrmap("insert_attributes", args)?,
        })
    }
}

impl From<UpdateUserParams> for UpdateUserRequest {
    fn from(value: UpdateUserParams) -> Self {
        UpdateUserRequest {
            user_id: UserId::from(value.user_id),
            email: value.email.map(Email::from),
            display_name: value.display_name,
            delete_attributes: value
                .delete_attributes
                .into_iter()
                .map(AttributeName::from)
                .collect(),
            insert_attributes: value.insert_attributes,
        }
    }
}
