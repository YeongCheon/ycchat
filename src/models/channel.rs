use crate::services::model::{channel::ChannelType as ChannelTypeMessage, Channel};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};
use ulid::Ulid;

use crate::db::surreal::{
    channel::serialize_id, deserialize_ulid_id, server::COLLECTION_NAME as SERVER_COLLECTION_NAME,
    server_category::COLLECTION_NAME as SERVER_CATEGORY_COLLECTION_NAME,
};

use super::{
    attachment::Attachment,
    server::{DbServer, ServerId},
    server_category::DbServerCategory,
    user::UserId,
};

pub type ChannelId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbChannel {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ChannelId,
    pub channel_type: ChannelType,
    pub display_name: String,
    pub description: String,
    pub order: u64,
    pub icon: Option<Attachment>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ChannelType {
    Saved { owner: UserId }, // self
    Direct,                  // 1:1 direct message
    Server { server: ServerId },
}

impl DbChannel {
    pub fn new(owner: UserId, message: Channel, server: Option<ServerId>) -> Self {
        let channel_type = ChannelTypeMessage::from_i32(message.channel_type).unwrap();

        let channel_type = match channel_type {
            ChannelTypeMessage::Saved => ChannelType::Saved { owner },
            ChannelTypeMessage::Direct => ChannelType::Direct,
            ChannelTypeMessage::Server => ChannelType::Server {
                server: server.unwrap(),
            },
        };

        DbChannel {
            id: ChannelId::new(),
            channel_type,
            display_name: message.display_name,
            description: message.description,
            order: 0,
            icon: None,
            create_time: chrono::offset::Utc::now(),
            update_time: None,
        }
    }

    pub fn to_message(self) -> Channel {
        Channel {
            name: format!("channels/{}", self.id),
            display_name: self.display_name,
            description: self.description,
            icon: None,
            channel_type: self.channel_type.to_message() as i32,
            unread_message_count: 0, // FIXME
            order: self.order,
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

    pub fn update(&mut self, message: Channel) {
        self.display_name = message.display_name;
        self.description = message.description;
        self.order = message.order;
        // self.icon = message.icon;
        self.update_time = Some(chrono::offset::Utc::now())
    }
}

impl ChannelType {
    pub fn new_saved(&self, owner: UserId) -> ChannelType {
        ChannelType::Saved { owner }
    }

    pub fn new_direct(&self) -> ChannelType {
        ChannelType::Direct
    }

    pub fn new_server(&self, server: ServerId) -> ChannelType {
        ChannelType::Server { server }
    }

    pub fn to_message(&self) -> ChannelTypeMessage {
        match self {
            ChannelType::Saved { owner } => ChannelTypeMessage::Saved,
            Self::Direct => ChannelTypeMessage::Direct,
            ChannelType::Server { server } => ChannelTypeMessage::Server,
        }
    }
}
