use crate::models::{
    channel::{ChannelId, DbChannel},
    server::ServerId,
};

#[tonic::async_trait]
pub trait ChannelRepository: Sync + Send {
    async fn get(&self, id: &ChannelId) -> Result<Option<DbChannel>, String>;

    async fn get_server_channels(&self, server_id: &ServerId) -> Result<Vec<DbChannel>, String>;

    async fn add(&self, channel: &DbChannel) -> Result<DbChannel, String>;

    async fn update(&self, channel: &DbChannel) -> Result<DbChannel, String>;

    async fn delete(&self, id: &ChannelId) -> Result<u8, String>;
}
