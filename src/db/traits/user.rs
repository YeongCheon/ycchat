use tonic::async_trait;

use crate::models::user::{DbUser, UserId};

#[async_trait]
pub trait UserRepository<C>: Sync + Send {
    async fn get_user(&self, db: &C, id: &UserId) -> Result<DbUser, String>;
    async fn add_user(&self, db: &C, user: &DbUser) -> Result<DbUser, String>;
    async fn update_user(&self, db: &C, user: &DbUser) -> Result<DbUser, String>;
    async fn delete_user(&self, db: &C, id: &UserId) -> Result<u8, String>;
    async fn get_users(&self, db: &C) -> Result<Vec<DbUser>, String>;
}
