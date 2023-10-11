use serde::{Serialize, Serializer};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Id, Thing},
    Surreal,
};

use crate::{
    db::traits::server_category::ServerCategoryRepository,
    models::server::ServerId,
    models::server_category::{DbServerCategory, ServerCategoryId},
};

use super::server::COLLECTION_NAME as SERVER_COLLECTION_NAME;

pub const COLLECTION_NAME: &str = "server_category";

#[derive(Clone)]
pub struct ServerCategoryRepositoryImpl {}

impl ServerCategoryRepositoryImpl {
    pub async fn new() -> Self {
        ServerCategoryRepositoryImpl {}
    }
}

#[tonic::async_trait]
impl ServerCategoryRepository<Surreal<Client>> for ServerCategoryRepositoryImpl {
    async fn get(
        &self,
        db: &Surreal<Client>,
        id: &ServerCategoryId,
    ) -> Result<Option<DbServerCategory>, String> {
        let res = db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res.unwrap()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(
        &self,
        db: &Surreal<Client>,
        server_category: &DbServerCategory,
    ) -> Result<Option<DbServerCategory>, String> {
        let created: Option<DbServerCategory> = db
            .create((COLLECTION_NAME, server_category.id.to_string()))
            .content(server_category)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(
        &self,
        db: &Surreal<Client>,
        server_category: &DbServerCategory,
    ) -> Result<Option<DbServerCategory>, String> {
        let res: Option<DbServerCategory> = db
            .update((COLLECTION_NAME, server_category.id.to_string()))
            .content(server_category.clone())
            .await
            .unwrap();

        return Ok(res);
    }

    async fn delete(&self, db: &Surreal<Client>, id: &ServerCategoryId) -> Result<u8, String> {
        db.delete::<Option<DbServerCategory>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    // TODO: paging
    async fn get_server_categories(
        &self,
        db: &Surreal<Client>,
        server_id: &ServerId,
        page_size: i32,
        offset_id: Option<ServerCategoryId>,
    ) -> Result<Vec<DbServerCategory>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let query = match offset_id {
            Some(offset_id) => db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE server_id == $server AND id < $offset_id ORDER BY id DESC LIMIT $page_size"
            ))
                .bind(("server", server))
                .bind(("offset_id", offset_id))
                .bind(("page_size", page_size)),
            None => db
                .query(format!(
                    "SELECT * FROM {COLLECTION_NAME} WHERE server_id == $server ORDER BY id DESC LIMIT $page_size"
                ))
                .bind(("server", server))
                .bind(("page_size", page_size)),
        };

        let res = query.await.unwrap().take::<Vec<DbServerCategory>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(id: &ServerCategoryId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
