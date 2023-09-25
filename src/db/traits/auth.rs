use crate::models::{auth::DbAuth, user::UserId};

#[tonic::async_trait]
pub trait AuthRepository<C>: Sync + Send {
    async fn get(&self, db: &C, id: &UserId) -> Result<Option<DbAuth>, String>;

    async fn get_by_username(&self, db: &C, username: &str) -> Result<Option<DbAuth>, String>;

    async fn add(&self, db: &C, auth: &DbAuth) -> Result<Option<DbAuth>, String>;

    async fn update(&self, db: &C, auth: &DbAuth) -> Result<Option<DbAuth>, String>;

    async fn delete(&self, db: &C, id: &UserId) -> Result<u8, String>;
}
