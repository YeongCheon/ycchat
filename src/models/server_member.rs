use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::services::model::ServerMember;

use super::attachment::Attachment;
use super::server::DbServer;
use super::user::DbUser;

pub type ServerMemberId = ulid::Ulid;

use crate::db::surreal::{deserialize_ulid_id, server_member::serialize_id};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServerMember {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ServerMemberId,
    pub server: DbServer,
    pub user: DbUser, // FIXME
    pub display_name: String,
    pub description: String,
    pub avatar: Option<Attachment>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl DbServerMember {
    pub fn new(message: ServerMember, server: DbServer, user: DbUser) -> Self {
        DbServerMember {
            id: ServerMemberId::new(),
            server,
            user,
            display_name: message.display_name,
            description: message.description,
            avatar: None,
            create_time: chrono::offset::Utc::now(),
            update_time: chrono::offset::Utc::now(),
        }
    }

    pub fn to_message(self) -> ServerMember {
        ServerMember {
            name: format!("servers/{}/membmers/{}", self.server.id, self.user.id),
            user: self.user.id,
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
