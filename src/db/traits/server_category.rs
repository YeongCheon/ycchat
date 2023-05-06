use crate::models::{
    server::ServerId,
    server_category::{DbServerCategory, ServerCategoryId},
};

#[tonic::async_trait]
pub trait ServerCategoryRepository: Sync + Send {
    async fn get(&self, id: &ServerCategoryId) -> Result<Option<DbServerCategory>, String>;

    async fn add(&self, server_category: &DbServerCategory) -> Result<DbServerCategory, String>;

    async fn update(&self, server_category: &DbServerCategory) -> Result<DbServerCategory, String>;

    async fn delete(&self, id: &ServerCategoryId) -> Result<u8, String>;

    async fn get_server_categories(
        &self,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerCategory>, String>; // FIXME
}
