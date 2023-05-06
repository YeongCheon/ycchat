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

use super::conn;

const COLLECTION_NAME: &str = "server_category";

pub struct ServerCategoryRepositoryImpl {
    db: Surreal<Client>,
}

impl ServerCategoryRepositoryImpl {
    pub async fn new() -> Self {
        ServerCategoryRepositoryImpl { db: conn().await }
    }
}

#[tonic::async_trait]
impl ServerCategoryRepository for ServerCategoryRepositoryImpl {
    async fn get(&self, id: &ServerCategoryId) -> Result<Option<DbServerCategory>, String> {
        let res = self.db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(&self, server_member: &DbServerCategory) -> Result<DbServerCategory, String> {
        let created: DbServerCategory = self
            .db
            .create((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(&self, server_member: &DbServerCategory) -> Result<DbServerCategory, String> {
        let res: Option<DbServerCategory> = self
            .db
            .update((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete(&self, id: &ServerCategoryId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbServerCategory>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_server_categories(
        &self,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerCategory>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let res = self
            .db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE server == $server"
            ))
            .bind(("server", server))
            .await
            .unwrap()
            .take::<Vec<DbServerCategory>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}
