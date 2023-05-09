use crate::services::model::{channel::ChannelType as ChannelTypeMessage, Channel};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};
use ulid::Ulid;

use crate::db::surreal::{
    channel::serialize_id, deserialize_id, server::COLLECTION_NAME as SERVER_COLLECTION_NAME,
    server_category::COLLECTION_NAME as SERVER_CATEGORY_COLLECTION_NAME,
};

use super::{attachment::Attachment, server::DbServer, server_category::DbServerCategory};

pub type ChannelId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbChannel {
    #[serde(serialize_with = "serialize_id", deserialize_with = "deserialize_id")]
    pub id: ChannelId,
    pub channel_type: ChannelType,
    pub display_name: String,
    pub description: String,
    pub order: u64,
    pub icon: Option<Attachment>,
    pub server: Option<Thing>,   // FIXME
    pub category: Option<Thing>, // FIXME
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChannelType {
    Saved,  // self
    Direct, // 1:1 direct message
    Server,
}

impl DbChannel {
    pub fn new(
        message: Channel,
        server: Option<DbServer>,
        category: Option<DbServerCategory>,
    ) -> Self {
        let server: Option<Thing> = server.map(|server| Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::from(server.id.to_string()),
        });

        let category = category.map(|category| Thing {
            tb: SERVER_CATEGORY_COLLECTION_NAME.to_string(),
            id: Id::from(category.id.to_string()),
        });

        DbChannel {
            id: Ulid::new().to_string(),
            channel_type: ChannelType::from(
                ChannelTypeMessage::from_i32(message.channel_type).unwrap(),
            ),
            display_name: message.display_name,
            description: message.description,
            order: 0,
            icon: None,
            server,
            category,
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

impl From<Channel> for DbChannel {
    fn from(message: Channel) -> Self {
        let create_time = match message.create_time {
            Some(create_time) => {
                let create_time = NaiveDateTime::from_timestamp_millis(create_time.seconds * 1000)
                    .unwrap()
                    .with_nanosecond(create_time.nanos as u32)
                    .unwrap();

                DateTime::<Utc>::from_utc(create_time, Utc)
            }
            None => chrono::offset::Utc::now(),
        };

        let update_time = match message.update_time {
            Some(update_time) => {
                let create_time = NaiveDateTime::from_timestamp_millis(update_time.seconds * 1000)
                    .unwrap()
                    .with_nanosecond(update_time.nanos as u32)
                    .unwrap();

                Some(DateTime::<Utc>::from_utc(create_time, Utc))
            }
            None => None,
        };

        DbChannel {
            id: message.name.split('/').collect::<Vec<&str>>()[1].to_string(),
            channel_type: ChannelType::from(
                ChannelTypeMessage::from_i32(message.channel_type).unwrap(),
            ),
            display_name: message.display_name,
            description: message.description,
            server: None,
            category: None,
            order: 0,   // FIXME
            icon: None, // FIXME
            create_time,
            update_time,
        }
    }
}

impl From<ChannelTypeMessage> for ChannelType {
    fn from(value: ChannelTypeMessage) -> Self {
        match value {
            ChannelTypeMessage::Saved => ChannelType::Saved,
            ChannelTypeMessage::Direct => ChannelType::Direct,
            ChannelTypeMessage::Server => ChannelType::Server,
        }
    }
}

impl ChannelType {
    pub fn to_message(&self) -> ChannelTypeMessage {
        match self {
            Self::Saved => ChannelTypeMessage::Saved,
            Self::Direct => ChannelTypeMessage::Direct,
            Self::Server => ChannelTypeMessage::Server,
        }
    }
}
