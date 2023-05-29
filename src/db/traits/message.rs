use crate::models::{channel::ChannelId, message::DbMessage};

#[tonic::async_trait]
pub trait MessageRepository: Sync + Send {
    async fn add(&self, message: &DbMessage) -> Result<DbMessage, String>;

    async fn get_list_by_chnanel_id(
        &self,
        channel_id: &ChannelId,
    ) -> Result<Vec<DbMessage>, String>;
}
