use chrono::{DateTime, Timelike, Utc};
use prost_types::Timestamp;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    db::surreal::{
        deserialize_ulid_id, server::serialize_id as server_serialize_id,
        server_category::serialize_id,
    },
    services::model::Category,
};

use super::{
    attachment::AttachmentId,
    server::{DbServer, ServerId},
};

pub type ServerCategoryId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbServerCategory {
    #[serde(
        serialize_with = "serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub id: ServerCategoryId,
    pub display_name: String,
    pub description: String,

    #[serde(
        serialize_with = "server_serialize_id",
        deserialize_with = "deserialize_ulid_id"
    )]
    pub server: ServerId,
    pub icon: Option<AttachmentId>,
    pub order: u32,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

impl DbServerCategory {
    pub fn new(server: DbServer, message: Category) -> Self {
        let id = Ulid::new();

        DbServerCategory {
            id,
            display_name: message.display_name,
            description: message.description,
            server: server.id,
            icon: None,
            order: message.order,
            create_time: chrono::offset::Utc::now(),
            update_time: None,
        }
    }

    pub fn to_message(self) -> Category {
        Category {
            name: format!("servers/{}/categories/{}", self.server, self.id),
            display_name: self.display_name,
            description: self.description,
            order: self.order,
            icon: None,
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

    pub fn update(&mut self, message: Category) {
        self.display_name = message.display_name;
        self.description = message.description;
        self.order = message.order;
        self.update_time = Some(chrono::offset::Utc::now());
    }
}
