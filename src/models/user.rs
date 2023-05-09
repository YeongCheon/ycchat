use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};

use super::attachment::Attachment;

pub type UserId = String;

use crate::services::model::User as UserMessage;

use crate::db::surreal::{deserialize_id, user::serialize_id};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbUser {
    #[serde(serialize_with = "serialize_id", deserialize_with = "deserialize_id")]
    pub id: UserId,
    pub display_name: String,
    pub description: String,
    pub avatar: Option<Attachment>,
    pub region_code: Option<String>,
    pub language_code: Option<String>,
    pub time_zone: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl DbUser {
    pub fn new(message: UserMessage) -> Self {
        DbUser {
            id: UserId::new(),
            display_name: message.display_name,
            description: message.description,
            avatar: None,
            region_code: message.region_code,
            language_code: message.language_code,
            time_zone: message.time_zone,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
        }
    }

    pub fn from(message: UserMessage) -> Self {
        DbUser {
            id: message.name.split('/').collect::<Vec<&str>>()[1].to_string(),
            display_name: message.display_name,
            description: message.description,
            avatar: None,
            region_code: message.region_code,
            language_code: message.language_code,
            time_zone: message.time_zone,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
        }
    }

    pub fn to_message(self) -> UserMessage {
        UserMessage {
            name: format!("users/{}", self.id),
            display_name: self.display_name,
            description: self.description,
            avatar: None, // FIXME
            region_code: self.region_code,
            language_code: self.language_code,
            time_zone: self.time_zone,
            create_time: Some(Timestamp {
                seconds: self.create_time.timestamp(),
                nanos: self.create_time.nanosecond() as i32,
            }),
            update_time: Some(Timestamp {
                seconds: self.update_time.timestamp(),
                nanos: self.update_time.nanosecond() as i32,
            }),
        }
    }
}
