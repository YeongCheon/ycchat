use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

use super::user::UserId;
use crate::db::surreal::auth::serialize_id;
use crate::db::surreal::deserialize_ulid_id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbAuth {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: UserId,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub is_email_verified: bool,
    pub create_time: Datetime,
    pub update_time: Option<Datetime>,
    pub last_login_time: Option<Datetime>,
}
