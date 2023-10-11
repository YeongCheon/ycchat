use crate::models::{
    channel::{ChannelId, DbChannel},
    server::ServerId,
};

#[tonic::async_trait]
pub trait ChannelRepository<C>: Sync + Send {
    async fn get(&self, db: &C, id: &ChannelId) -> Result<Option<DbChannel>, String>;

    async fn get_server_channels(
        &self,
        db: &C,
        server_id: &ServerId,
        page_size: i32,
        offset_id: Option<ChannelId>,
    ) -> Result<Vec<DbChannel>, String>;

    async fn add(&self, db: &C, channel: &DbChannel) -> Result<Option<DbChannel>, String>;

    async fn update(&self, db: &C, channel: &DbChannel) -> Result<Option<DbChannel>, String>;

    async fn delete(&self, db: &C, id: &ChannelId) -> Result<u8, String>;
}
