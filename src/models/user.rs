use std::str::FromStr;

use chrono::Timelike;
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
use ulid::Ulid;

use super::attachment::Attachment;

pub type UserId = Ulid;

use crate::services::model::User as UserMessage;

use crate::db::surreal::{deserialize_ulid_id, user::serialize_id};
use crate::util::pager::PageItem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbUser {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: UserId,
    pub display_name: String,
    pub description: String,
    pub avatar: Option<Attachment>,
    pub region_code: Option<String>,
    pub language_code: Option<String>,
    pub time_zone: Option<String>,
    pub create_time: Datetime,
    pub update_time: Option<Datetime>,
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
            create_time: Datetime::default(),
            update_time: None,
        }
    }

    pub fn from(message: UserMessage) -> Self {
        DbUser {
            id: Ulid::from_str(message.name.split('/').collect::<Vec<&str>>()[1]).unwrap(),
            display_name: message.display_name,
            description: message.description,
            avatar: None,
            region_code: message.region_code,
            language_code: message.language_code,
            time_zone: message.time_zone,
            create_time: Datetime::default(),
            update_time: None,
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
            update_time: self.update_time.map(|update_time| Timestamp {
                seconds: update_time.timestamp(),
                nanos: update_time.nanosecond() as i32,
            }),
        }
    }
}

impl PageItem for DbUser {
    fn get_item_id(&self) -> String {
        self.id.to_string()
    }
}
