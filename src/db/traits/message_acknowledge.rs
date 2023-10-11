use crate::models::{
    message::MessageId,
    message_acknowledge::{DbMessageAcknowledge, MessageAcknowledgeId},
    user::UserId,
};

#[tonic::async_trait]
pub trait MessageAcknowledgeRepository<C>: Sync + Send {
    async fn get(
        &self,
        db: &C,
        id: MessageAcknowledgeId,
    ) -> Result<Option<DbMessageAcknowledge>, String>;

    async fn get_by_message_and_user(
        &self,
        db: &C,
        message_id: &MessageId,
        user_id: &UserId,
    ) -> Result<Option<DbMessageAcknowledge>, String>;

    async fn get_list_by_message(
        &self,
        db: &C,
        message_id: &MessageId,
        page_size: i32,
        offset_id: Option<MessageAcknowledgeId>,
    ) -> Result<Vec<DbMessageAcknowledge>, String>;

    async fn add(
        &self,
        db: &C,
        message_acknowledge: &DbMessageAcknowledge,
    ) -> Result<Option<DbMessageAcknowledge>, String>;
}
