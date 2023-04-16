use tonic::async_trait;

use crate::models::user::{DbUser, UserId};

#[async_trait]
pub trait UserRepository: Sync + Send {
    async fn get_user(&self, id: &UserId) -> Result<DbUser, String>;
    async fn add_user(&self, user: &DbUser) -> Result<DbUser, String>;
    async fn update_user(&self, user: &DbUser) -> Result<DbUser, String>;
    async fn delete_user(&self, id: &UserId) -> Result<u8, String>;
    async fn get_users(&self) -> Result<Vec<DbUser>, String>;
}
