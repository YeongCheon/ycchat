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

#[derive(Clone)]
pub struct MessageRepositoryImpl {}

impl MessageRepositoryImpl {
    pub async fn new() -> Self {
        MessageRepositoryImpl {}
    }
}

#[async_trait]
impl MessageRepository<Surreal<Client>> for MessageRepositoryImpl {
    async fn get(&self, db: &Surreal<Client>, id: &MessageId) -> Result<Option<DbMessage>, String> {
        let res = db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete(&self, db: &Surreal<Client>, id: &MessageId) -> Result<u8, String> {
        db.delete::<Option<DbMessage>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn add(&self, db: &Surreal<Client>, message: &DbMessage) -> Result<DbMessage, String> {
        let created: DbMessage = db
            .create((COLLECTION_NAME, message.id.to_string()))
            .content(message)
            .await
            .unwrap()
            .unwrap();

        Ok(created)
    }

    async fn get_list_by_chnanel_id(
        &self,
        db: &Surreal<Client>,
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
