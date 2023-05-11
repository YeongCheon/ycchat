use crate::models::{auth::DbAuth, user::UserId};

#[tonic::async_trait]
pub trait AuthRepository: Sync + Send {
    async fn get(&self, id: &UserId) -> Result<Option<DbAuth>, String>;
    async fn get_by_username(&self, username: &str) -> Result<Option<DbAuth>, String>;
    async fn add(&self, auth: &DbAuth) -> Result<DbAuth, String>;
    async fn update(&self, auth: &DbAuth) -> Result<DbAuth, String>;
    async fn delete(&self, id: &UserId) -> Result<u8, String>;
}
