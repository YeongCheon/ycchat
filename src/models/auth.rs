use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::UserId;
use crate::db::surreal::deserialize_id;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbAuth {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: UserId,
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub is_email_verified: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
    pub last_login_time: Option<DateTime<Utc>>,
}
