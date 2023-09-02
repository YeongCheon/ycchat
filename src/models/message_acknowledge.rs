use crate::db::surreal::{deserialize_ulid_id, message_acknowledge::serialize_id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{message::MessageId, user::UserId};

pub type MessageAcknowledgeId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbMessageAcknowledge {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: MessageAcknowledgeId,

    pub message_id: MessageId,

    pub user_id: UserId,

    pub create_time: DateTime<Utc>,
}

impl DbMessageAcknowledge {
    pub fn new(message_id: MessageId, user_id: UserId) -> Self {
        DbMessageAcknowledge {
            id: MessageAcknowledgeId::new(),
            message_id,
            user_id,
            create_time: chrono::offset::Utc::now(),
        }
    }
}
