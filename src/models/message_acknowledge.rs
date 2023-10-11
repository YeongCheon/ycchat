use crate::{
    db::surreal::{deserialize_ulid_id, message_acknowledge::serialize_id},
    util::pager::PageItem,
};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;
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

    pub create_time: Datetime,
}

impl DbMessageAcknowledge {
    pub fn new(message_id: MessageId, user_id: UserId) -> Self {
        DbMessageAcknowledge {
            id: MessageAcknowledgeId::new(),
            message_id,
            user_id,
            create_time: Datetime::default(),
        }
    }
}

impl PageItem for DbMessageAcknowledge {
    fn get_item_id(&self) -> String {
        self.id.to_string()
    }
}
