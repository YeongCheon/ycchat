use super::{channel::ChannelId, user::UserId};
use crate::{
    db::surreal::{
        channel::serialize_id as channel_serialize_id, deserialize_ulid_id, message::serialize_id,
        user::serialize_id as user_serialize_id,
    },
    services::model::Message,
};
use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

pub type MessageId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbMessage {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: MessageId,

    #[serde(
        serialize_with = "channel_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub channel: ChannelId,

    #[serde(
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub author: UserId,
    pub content: String,
    // pub reactions: Vec<Reaction>,
    // pub attachments: Vec<AttachmentId>,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

pub enum Reaction {}

impl DbMessage {
    pub fn new(author: UserId, channel: ChannelId, content: String) -> Self {
        DbMessage {
            id: MessageId::new(),
            author,
            channel,
            content,
            create_time: chrono::offset::Utc::now(),
            update_time: None,
        }
    }

    pub fn to_message(self) -> Message {
        let mut reactions = HashMap::new();

        Message {
            name: format!(
                "channels/{}/messages/{}",
                self.channel.to_string(),
                self.id.to_string()
            ),
            author: self.author.to_string(),
            content: self.content,
            reactions,
            attachments: vec![],
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
