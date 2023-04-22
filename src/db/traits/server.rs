use crate::models::server::{DbServer, ServerId};

#[tonic::async_trait]
pub trait ServerRepository: Sync + Send {
    async fn get_server(&self, id: &ServerId) -> Result<DbServer, String>;
    async fn add_server(&self, server: &DbServer) -> Result<DbServer, String>;
    async fn update_server(&self, server: &DbServer) -> Result<DbServer, String>;
    async fn delete_server(&self, id: &ServerId) -> Result<u8, String>;
    async fn get_servers(&self) -> Result<Vec<DbServer>, String>;
}
