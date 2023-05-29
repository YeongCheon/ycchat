use serde::{Serialize, Serializer};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tonic::async_trait;

use crate::{
    db::traits::message::MessageRepository,
    models::{
        channel::ChannelId,
        message::{DbMessage, MessageId},
    },
};

pub const COLLECTION_NAME: &str = "message";

use super::conn;

#[derive(Clone)]
pub struct MessageRepositoryImpl {
    db: Surreal<Client>,
}

impl MessageRepositoryImpl {
    pub async fn new() -> Self {
        MessageRepositoryImpl { db: conn().await }
    }
}

#[async_trait]
impl MessageRepository for MessageRepositoryImpl {
    async fn get(&self, id: &MessageId) -> Result<Option<DbMessage>, String> {
        let res = self.db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete(&self, id: &MessageId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbMessage>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn add(&self, message: &DbMessage) -> Result<DbMessage, String> {
        let created: DbMessage = self
            .db
            .create((COLLECTION_NAME, message.id.to_string()))
            .content(message)
            .await
            .unwrap();

        Ok(created)
    }

    async fn get_list_by_chnanel_id(
        &self,
        channel_id: &ChannelId,
    ) -> Result<Vec<DbMessage>, String> {
        todo!()
    }
}

pub fn serialize_id<S>(id: &MessageId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
