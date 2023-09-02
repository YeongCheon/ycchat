use crate::models::{
    server::ServerId,
    server_category::{DbServerCategory, ServerCategoryId},
};

#[tonic::async_trait]
pub trait ServerCategoryRepository<C>: Sync + Send {
    async fn get(&self, db: &C, id: &ServerCategoryId) -> Result<Option<DbServerCategory>, String>;

    async fn add(
        &self,
        db: &C,
        server_category: &DbServerCategory,
    ) -> Result<DbServerCategory, String>;

    async fn update(
        &self,
        db: &C,
        server_category: &DbServerCategory,
    ) -> Result<DbServerCategory, String>;

    async fn delete(&self, db: &C, id: &ServerCategoryId) -> Result<u8, String>;

    async fn get_server_categories(
        &self,
        db: &C,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerCategory>, String>; // FIXME
}
