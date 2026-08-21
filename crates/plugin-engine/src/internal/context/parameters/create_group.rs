use lldap_domain::types::{Attribute, GroupName};
use lldap_domain_handlers::requests::CreateGroupRequest;
use mlua::{Result as LuaResult, Table};

use super::utils;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateGroupParams {
    pub display_name: String,
    pub attributes: Vec<Attribute>,
}
impl CreateGroupParams {
    pub fn from(args: &Table) -> LuaResult<Self> {
        Ok(CreateGroupParams {
            display_name: args.get("display_name")?,
            attributes: utils::get_opt_attrmap("attributes", args)?,
        })
    }
}

impl From<CreateGroupParams> for CreateGroupRequest {
    fn from(value: CreateGroupParams) -> Self {
        CreateGroupRequest {
            display_name: GroupName::from(value.display_name),
            attributes: value.attributes,
        }
    }
}
