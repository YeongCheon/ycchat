use serde::Serializer;
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::async_trait;

use super::super::traits::user::UserRepository;
use super::conn;
use crate::models::user::{DbUser, UserId};

pub struct UserRepositoryImpl {
    db: Surreal<Client>,
}

impl UserRepositoryImpl {
    pub async fn new() -> Self {
        UserRepositoryImpl { db: conn().await }
    }
}

const COLLECTION_NAME: &str = "user";

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn get_user(&self, id: &UserId) -> Result<DbUser, String> {
        let res = self
            .db
            .select::<Option<DbUser>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_user(&self, user: &DbUser) -> Result<DbUser, String> {
        let created: DbUser = self
            .db
            .create((COLLECTION_NAME, user.id.to_string()))
            .content(user)
            .await
            .unwrap();
        dbg!(&created);

        Ok(created)
    }

    async fn update_user(&self, user: &DbUser) -> Result<DbUser, String> {
        let res: Option<DbUser> = self
            .db
            .update((COLLECTION_NAME, user.id.to_string()))
            .content(user.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete_user(&self, id: &UserId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbUser>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_users(&self) -> Result<Vec<DbUser>, String> {
        let res = self.db.select::<Vec<DbUser>>(COLLECTION_NAME).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(user_id: &UserId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = format!("{}:{}", COLLECTION_NAME, user_id);

    s.serialize_str(&surreal_id)
}
