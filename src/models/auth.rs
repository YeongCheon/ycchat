use serde::{Deserialize, Serialize};

use super::user::UserId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbAuth {
    #[serde(rename(serialize = "user_id", deserialize = "user_id"))] // FIXME
    pub id: UserId,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub is_email_verified: bool,
}
