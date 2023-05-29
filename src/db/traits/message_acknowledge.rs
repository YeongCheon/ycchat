use crate::models::{
    message::MessageId,
    message_acknowledge::{self, DbMessageAcknowledge, MessageAcknowledgeId},
    user::UserId,
};

#[tonic::async_trait]
pub trait MessageAcknowledgeRepository: Sync + Send {
    async fn get(&self, id: MessageAcknowledgeId) -> Result<Option<DbMessageAcknowledge>, String>;

    async fn get_by_message_and_user(
        &self,
        message_id: &MessageId,
        user_id: &UserId,
    ) -> Result<Option<DbMessageAcknowledge>, String>;

    async fn get_list_by_message(
        &self,
        message_id: &MessageId,
    ) -> Result<Vec<DbMessageAcknowledge>, String>;

    async fn add(
        &self,
        message_acknowledge: &DbMessageAcknowledge,
    ) -> Result<DbMessageAcknowledge, String>;
}
