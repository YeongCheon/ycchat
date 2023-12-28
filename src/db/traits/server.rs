use crate::models::{
    server::{DbServer, ServerId},
    user::UserId,
};

#[tonic::async_trait]
pub trait ServerRepository<C>: Sync + Send {
    async fn get_server(&self, db: &C, id: &ServerId) -> Result<Option<DbServer>, String>;
    async fn add_server(&self, db: &C, server: &DbServer) -> Result<Option<DbServer>, String>;
    async fn update_server(&self, db: &C, server: &DbServer) -> Result<Option<DbServer>, String>;
    async fn delete_server(&self, db: &C, id: &ServerId) -> Result<u8, String>;
    async fn get_servers(
        &self,
        db: &C,
        page_size: i32,
        offset_id: Option<ServerId>,
    ) -> Result<Vec<DbServer>, String>;

    async fn get_joined_servers(
        &self,
        db: &C,
        user_id: &UserId,
        page_size: i32,
        offset_id: Option<ServerId>,
    ) -> Result<Vec<DbServer>, String>;
}
