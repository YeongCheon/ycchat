use super::{channel::ChannelId, user::UserId};
use crate::{
    db::surreal::{
        channel::serialize_id as channel_serialize_id, deserialize_ulid_id, message::serialize_id,
        user::serialize_id as user_serialize_id,
    },
    services::ycchat::v1::models::Message,
    util::pager::PageItem,
};
use chrono::Timelike;
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Datetime;
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
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub author: UserId,

    #[serde(
        serialize_with = "channel_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub channel: ChannelId,

    pub content: String,

    pub message_type: String,
    // pub attachments: Vec<AttachmentId>,
    pub create_time: Datetime,
    pub update_time: Option<Datetime>,
}

impl DbMessage {
    pub fn new(author: UserId, channel: ChannelId, content: String) -> Self {
        DbMessage {
            id: MessageId::new(),
            author,
            channel,
            content,
            message_type: "FIXME".to_string(),
            create_time: Datetime::default(),
            update_time: None,
        }
    }

    pub fn to_message(self) -> Message {
        Message {
            name: format!(
                "channels/{}/messages/{}",
                self.channel.to_string(),
                self.id.to_string()
            ),
            author: self.author.to_string(),
            content: self.content,
            reactions: HashMap::new(), // FIXME
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

impl PageItem for DbMessage {
    fn get_item_id(&self) -> String {
        self.id.to_string()
    }
}
