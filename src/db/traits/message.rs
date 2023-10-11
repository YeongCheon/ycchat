use crate::models::{
    channel::ChannelId,
    message::{DbMessage, MessageId},
};

#[tonic::async_trait]
pub trait MessageRepository<C>: Sync + Send {
    async fn get(&self, db: &C, id: &MessageId) -> Result<Option<DbMessage>, String>;

    async fn add(&self, db: &C, message: &DbMessage) -> Result<Option<DbMessage>, String>;

    async fn delete(&self, db: &C, id: &MessageId) -> Result<u8, String>;

    async fn get_list_by_chnanel_id(
        &self,
        db: &C,
        channel_id: &ChannelId,
        page_size: i32,
        offset_id: Option<MessageId>,
    ) -> Result<Vec<DbMessage>, String>;
}
