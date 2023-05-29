use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::services::model::ServerMember;

use super::{attachment::Attachment, server::ServerId, user::UserId};

pub type ServerMemberId = ulid::Ulid;

use crate::db::surreal::{
    deserialize_ulid_id, server::serialize_id as server_serialize_id, server_member::serialize_id,
    user::serialize_id as user_serialize_id,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServerMember {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ServerMemberId,
    #[serde(
        serialize_with = "server_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub server: ServerId,
    #[serde(
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub user: UserId, // FIXME
    pub display_name: String,
    pub description: String,
    pub avatar: Option<Attachment>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl DbServerMember {
    pub fn new(display_name: String, description: String, server: ServerId, user: UserId) -> Self {
        DbServerMember {
            id: ServerMemberId::new(),
            server,
            user,
            display_name,
            description,
            avatar: None,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
        }
    }

    pub fn to_message(self) -> ServerMember {
        ServerMember {
            name: format!("servers/{}/membmers/{}", self.server, self.id),
            user: self.user.to_string(),
            display_name: self.display_name,
            description: self.description,
            avartar: None, // FIXME
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
