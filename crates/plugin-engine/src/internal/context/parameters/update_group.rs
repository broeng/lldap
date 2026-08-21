use mlua::{Result as LuaResult, Table};

use lldap_domain::types::{Attribute, AttributeName, GroupId, GroupName};
use lldap_domain_handlers::requests::UpdateGroupRequest;

use crate::internal::context::parameters::utils::{get_opt_arg, get_opt_attrmap, get_opt_vec};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UpdateGroupParams {
    pub group_id: i32,
    pub display_name: Option<String>,
    pub delete_attributes: Vec<String>,
    pub insert_attributes: Vec<Attribute>,
}

impl UpdateGroupParams {
    pub fn from(args: &Table) -> LuaResult<Self> {
        Ok(UpdateGroupParams {
            group_id: args.get("group_id")?,
            display_name: get_opt_arg("display_name", args)?,
            delete_attributes: get_opt_vec("delete_attributes", args)?,
            insert_attributes: get_opt_attrmap("insert_attributes", args)?,
        })
    }
}

impl From<UpdateGroupParams> for UpdateGroupRequest {
    fn from(value: UpdateGroupParams) -> Self {
        UpdateGroupRequest {
            group_id: GroupId(value.group_id),
            display_name: value.display_name.map(GroupName::from),
            delete_attributes: value
                .delete_attributes
                .into_iter()
                .map(AttributeName::from)
                .collect(),
            insert_attributes: value.insert_attributes,
        }
    }
}
