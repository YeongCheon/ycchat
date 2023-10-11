use super::{attachment::Attachment, server::ServerId, user::UserId};
use crate::db::surreal::{
    deserialize_ulid_id, server::serialize_id as server_serialize_id, server_member::serialize_id,
    user::serialize_id as user_serialize_id,
};
use crate::services::model::ServerMember;
use crate::util::pager::PageItem;
use chrono::Timelike;
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

pub type ServerMemberId = ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServerMember {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ServerMemberId,
    #[serde(
        rename = "in",
        serialize_with = "user_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub user: UserId,
    #[serde(
        rename = "out",
        serialize_with = "server_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub server: ServerId,
    pub display_name: String,
    pub description: String,
    pub avatar: Option<Attachment>,
    pub create_time: Datetime,
    pub update_time: Option<Datetime>,
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
            create_time: Datetime::default(),
            update_time: None,
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
            update_time: self.update_time.map(|update_time| Timestamp {
                seconds: update_time.timestamp(),
                nanos: update_time.nanosecond() as i32,
            }),
        }
    }
}

impl PageItem for DbServerMember {
    fn get_item_id(&self) -> String {
        self.id.to_string()
    }
}
