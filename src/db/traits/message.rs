use crate::models::{
    channel::ChannelId,
    message::{DbMessage, MessageId},
};

#[tonic::async_trait]
pub trait MessageRepository: Sync + Send {
    async fn get(&self, id: &MessageId) -> Result<Option<DbMessage>, String>;
    async fn add(&self, message: &DbMessage) -> Result<DbMessage, String>;
    async fn delete(&self, id: &MessageId) -> Result<u8, String>;

    async fn get_list_by_chnanel_id(
        &self,
        channel_id: &ChannelId,
    ) -> Result<Vec<DbMessage>, String>;
}
