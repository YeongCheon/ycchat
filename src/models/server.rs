use chrono::Timelike;
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

use crate::{services::model::Server, util::pager::PageItem};

use super::{
    attachment::{Attachment, AttachmentId},
    user::UserId,
};

pub type ServerId = ulid::Ulid;

use crate::db::surreal::{
    deserialize_ulid_id, server::serialize_id, user::serialize_id as user_serialize_id,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServer {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ServerId,
    pub display_name: String, // TITLE
    pub description: String,
    #[serde(
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub owner: UserId,
    #[serde(
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub author: UserId,
    pub icon: Option<AttachmentId>,
    pub create_time: Datetime,
    pub update_time: Option<Datetime>,
    // pub managers: Vec<UserId>,
}

impl DbServer {
    pub fn new(owner: UserId, message: Server) -> Self {
        DbServer {
            id: ServerId::new(),
            display_name: message.display_name,
            description: message.description,
            owner,
            author: owner,
            icon: None,
            create_time: Datetime::default(),
            update_time: None,
        }
    }

    pub fn from(message: Server) -> Self {
        DbServer {
            id: ServerId::from_string(message.name.split('/').collect::<Vec<&str>>()[1]).unwrap(),
            display_name: message.display_name,
            description: message.description,
            owner: UserId::new(),  // FIXME
            author: UserId::new(), // FIXME
            icon: None,
            create_time: Datetime::default(),
            update_time: Some(Datetime::default()),
        }
    }

    pub fn to_message(self) -> Server {
        Server {
            name: format!("servers/{}", self.id.to_string()),
            display_name: self.display_name,
            description: self.description,
            icon: None,         // FIXME
            categories: vec![], // FIXME
            channels: vec![],
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

impl PageItem for DbServer {
    fn get_item_id(&self) -> String {
        self.id.to_string()
    }
}
