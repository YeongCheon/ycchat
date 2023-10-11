use super::channel::COLLECTION_NAME as CHANNEL_COLLECTION_NAME;
use crate::{
    db::traits::message::MessageRepository,
    models::{
        channel::ChannelId,
        message::{DbMessage, MessageId},
    },
};
use serde::{Serialize, Serializer};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Id, Thing},
    Surreal,
};
use tonic::async_trait;

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

    async fn add(
        &self,
        db: &Surreal<Client>,
        message: &DbMessage,
    ) -> Result<Option<DbMessage>, String> {
        let created: Option<DbMessage> = db
            .create((COLLECTION_NAME, message.id.to_string()))
            .content(message)
            .await
            .unwrap();

        Ok(created)
    }

    async fn get_list_by_chnanel_id(
        &self,
        db: &Surreal<Client>,
        channel_id: &ChannelId,
        page_size: i32,
        offset_id: Option<MessageId>,
    ) -> Result<Vec<DbMessage>, String> {
        let channel = Thing {
            tb: CHANNEL_COLLECTION_NAME.to_string(),
            id: Id::String(channel_id.to_string()),
        };

        let query = match offset_id {
            Some(offset_id) => db.query(format!(
            "SELECT * FROM {COLLECTION_NAME} WHERE channel == $channel AND id < $offset_id ORDER BY id DESC LIMIT $page_size"
        ))
                .bind(("channel", channel))
                .bind(("offset_id", offset_id))
                .bind(("page_size", page_size)),
            None => db.query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE channel == $channel ORDER BY id DESC LIMIT $page_size"
            ))
                .bind(("channel", channel))
                .bind(("page_size", page_size)),
        };

        let res = query.await.unwrap().take::<Vec<DbMessage>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(id: &MessageId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
