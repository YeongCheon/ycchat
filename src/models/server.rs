use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::services::model::Server;

use super::attachment::Attachment;

pub type ServerId = ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServer {
    #[serde(rename(serialize = "server_id", deserialize = "server_id"))] // FIXME
    pub id: ServerId,
    pub display_name: String, // TITLE
    pub description: String,
    pub icon: Option<Attachment>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl DbServer {
    pub fn new(message: Server) -> Self {
        DbServer {
            id: ServerId::new(),
            display_name: message.display_name,
            description: message.description,
            icon: None,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
        }
    }

    pub fn from(message: Server) -> Self {
        DbServer {
            id: ServerId::from_string(message.name.split('/').collect::<Vec<&str>>()[1]).unwrap(),
            display_name: message.display_name,
            description: message.description,
            icon: None,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
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
            update_time: Some(Timestamp {
                seconds: self.update_time.timestamp(),
                nanos: self.update_time.nanosecond() as i32,
            }),
        }
    }
}
