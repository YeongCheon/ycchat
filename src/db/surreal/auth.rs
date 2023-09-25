use serde::{Serialize, Serializer};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

use crate::{
    db::traits::auth::AuthRepository,
    models::{auth::DbAuth, user::UserId},
};

#[derive(Clone)]
pub struct AuthRepositoryImpl {}

impl AuthRepositoryImpl {
    pub async fn new() -> Self {
        AuthRepositoryImpl {}
    }
}

const COLLECTION_NAME: &str = "auth";

#[tonic::async_trait]
impl AuthRepository<Surreal<Client>> for AuthRepositoryImpl {
    async fn get(&self, db: &Surreal<Client>, id: &UserId) -> Result<Option<DbAuth>, String> {
        let res = db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_by_username(
        &self,
        db: &Surreal<Client>,
        username: &str,
    ) -> Result<Option<DbAuth>, String> {
        let mut res = db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE username = $username"
            ))
            .bind(("username", username))
            .await
            .unwrap();

        Ok(res.take::<Option<DbAuth>>(0).unwrap())
    }

    async fn add(&self, db: &Surreal<Client>, auth: &DbAuth) -> Result<Option<DbAuth>, String> {
        let created = db
            .create((COLLECTION_NAME, auth.id.to_string()))
            .content(auth)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(&self, db: &Surreal<Client>, auth: &DbAuth) -> Result<Option<DbAuth>, String> {
        let res: Option<DbAuth> = db
            .update((COLLECTION_NAME, auth.id.to_string()))
            .content(auth.clone())
            .await
            .unwrap();

        return Ok(res);
    }

    async fn delete(&self, db: &Surreal<Client>, id: &UserId) -> Result<u8, String> {
        db.delete::<Option<DbAuth>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }
}

pub fn serialize_id<S>(id: &UserId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
