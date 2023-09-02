use crate::models::server::{DbServer, ServerId};

#[tonic::async_trait]
pub trait ServerRepository<C>: Sync + Send {
    async fn get_server(&self, db: &C, id: &ServerId) -> Result<DbServer, String>;
    async fn add_server(&self, db: &C, server: &DbServer) -> Result<DbServer, String>;
    async fn update_server(&self, db: &C, server: &DbServer) -> Result<DbServer, String>;
    async fn delete_server(&self, db: &C, id: &ServerId) -> Result<u8, String>;
    async fn get_servers(&self, db: &C) -> Result<Vec<DbServer>, String>;
}
