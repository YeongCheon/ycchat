use super::{
    message::COLLECTION_NAME as MESSAGE_COLLECTION_NAME,
    user::COLLECTION_NAME as USER_COLLECTION_NAME,
};
use crate::{
    db::traits::message_acknowledge::MessageAcknowledgeRepository,
    models::{
        message::MessageId,
        message_acknowledge::{DbMessageAcknowledge, MessageAcknowledgeId},
        user::UserId,
    },
};
use serde::{Serialize, Serializer};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Id, Thing},
    Surreal,
};
use tonic::async_trait;

const COLLECTION_NAME: &str = "message_acknowledge";

#[derive(Clone)]
pub struct MessageAcknowledgeRepositoryImpl {}

impl MessageAcknowledgeRepositoryImpl {
    pub async fn new() -> Self {
        MessageAcknowledgeRepositoryImpl {}
    }
}

#[async_trait]
impl MessageAcknowledgeRepository<Surreal<Client>> for MessageAcknowledgeRepositoryImpl {
    async fn get(
        &self,
        db: &Surreal<Client>,
        id: MessageAcknowledgeId,
    ) -> Result<Option<DbMessageAcknowledge>, String> {
        let res = db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_by_message_and_user(
        &self,
        db: &Surreal<Client>,
        message_id: &MessageId,
        user_id: &UserId,
    ) -> Result<Option<DbMessageAcknowledge>, String> {
        let message = Thing {
            tb: MESSAGE_COLLECTION_NAME.to_string(),
            id: Id::String(message_id.to_string()),
        };

        let user = Thing {
            tb: USER_COLLECTION_NAME.to_string(),
            id: Id::String(user_id.to_string()),
        };

        let res = db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE message_id == $message_id AND user_id == $user_id"
            ))
            .bind(("message_id", message))
            .bind(("user_id", user))
            .await
            .unwrap()
            .take::<Option<DbMessageAcknowledge>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_list_by_message(
        &self,
        db: &Surreal<Client>,
        message_id: &MessageId,
    ) -> Result<Vec<DbMessageAcknowledge>, String> {
        let message = Thing {
            tb: MESSAGE_COLLECTION_NAME.to_string(),
            id: Id::String(message_id.to_string()),
        };

        let res = db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE message_id == $message_id"
            ))
            .bind(("message_id", message))
            .await
            .unwrap()
            .take::<Vec<DbMessageAcknowledge>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(
        &self,
        db: &Surreal<Client>,
        message_acknowledge: &DbMessageAcknowledge,
    ) -> Result<DbMessageAcknowledge, String> {
        let created: DbMessageAcknowledge = db
            .create((COLLECTION_NAME, message_acknowledge.id.to_string()))
            .content(message_acknowledge)
            .await
            .unwrap()
            .unwrap();

        Ok(created)
    }
}

pub fn serialize_id<S>(id: &MessageAcknowledgeId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
