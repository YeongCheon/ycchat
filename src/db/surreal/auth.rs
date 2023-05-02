use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    db::traits::auth::AuthRepository,
    models::{auth::DbAuth, user::UserId},
};

use super::conn;

#[derive(Clone)]
pub struct AuthRepositoryImpl {
    db: Surreal<Client>,
}

impl AuthRepositoryImpl {
    pub async fn new() -> Self {
        AuthRepositoryImpl { db: conn().await }
    }
}

const COLLECTION_NAME: &str = "auth";

#[tonic::async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn get(&self, id: &UserId) -> Result<Option<DbAuth>, String> {
        let res = self.db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<DbAuth>, String> {
        let mut res = self
            .db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE username = $username"
            ))
            .bind(("username", username))
            .await
            .unwrap();

        Ok(res.take::<Option<DbAuth>>(0).unwrap())
    }

    async fn add(&self, auth: &DbAuth) -> Result<crate::models::auth::DbAuth, String> {
        let created: DbAuth = self
            .db
            .create((COLLECTION_NAME, auth.id.to_string()))
            .content(auth)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(&self, auth: &DbAuth) -> Result<DbAuth, String> {
        let res: Option<DbAuth> = self
            .db
            .update((COLLECTION_NAME, auth.id.to_string()))
            .content(auth.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete(&self, id: &UserId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbAuth>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }
}
