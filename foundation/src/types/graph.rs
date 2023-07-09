use super::role::UserRole;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersResponse {
    #[serde(rename = "@odata.context")]
    pub odata_context: String,
    pub value: Vec<User>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub display_name: String,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "extension_0bc691a1eb4c42f49cdf50357f8505b3_Role", default)]
    pub extension_role: UserRole,
    pub id: String,
}
