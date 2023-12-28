use serde::{Serialize, Serializer};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tonic::async_trait;

use super::super::traits::user::UserRepository;
use crate::models::user::{DbUser, UserId};

#[derive(Clone)]
pub struct UserRepositoryImpl {}

impl UserRepositoryImpl {
    pub async fn new() -> Self {
        UserRepositoryImpl {}
    }
}

pub const COLLECTION_NAME: &str = "user";

#[async_trait]
impl UserRepository<Surreal<Client>> for UserRepositoryImpl {
    async fn get_user(&self, db: &Surreal<Client>, id: &UserId) -> Result<Option<DbUser>, String> {
        let res = db
            .select::<Option<DbUser>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_user(
        &self,
        db: &Surreal<Client>,
        user: &DbUser,
    ) -> Result<Option<DbUser>, String> {
        let created: Option<DbUser> = db
            .create((COLLECTION_NAME, user.id.to_string()))
            .content(user)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update_user(
        &self,
        db: &Surreal<Client>,
        user: &DbUser,
    ) -> Result<Option<DbUser>, String> {
        let res: Option<DbUser> = db
            .update((COLLECTION_NAME, user.id.to_string()))
            .content(user.clone())
            .await
            .unwrap();

        return Ok(res);
    }

    async fn delete_user(&self, db: &Surreal<Client>, id: &UserId) -> Result<u8, String> {
        db.delete::<Option<DbUser>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_users(
        &self,
        db: &Surreal<Client>,
        page_size: i32,
        offset_id: Option<UserId>,
    ) -> Result<Vec<DbUser>, String> {
        let query = match offset_id {
            Some(offset_id) => db.query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE id < $offset_id ORDER BY id DESC LIMIT $page_size"
            ))
                .bind(("offset_id", offset_id))
                .bind(("page_size", page_size)),
            None => db.query(format!(
                "SELECT * FROM {COLLECTION_NAME} ORDER BY id DESC LIMIT $page_size"
            ))
                .bind(("page_size", page_size)),
        };

        let res = query.await.unwrap().take::<Vec<DbUser>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(id: &UserId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
