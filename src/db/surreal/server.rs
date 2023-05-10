use serde::{Serialize, Serializer};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tonic::async_trait;

use crate::{
    db::traits::server::ServerRepository,
    models::server::{DbServer, ServerId},
};

use super::conn;

#[derive(Clone)]
pub struct ServerRepositoryImpl {
    db: Surreal<Client>,
}

impl ServerRepositoryImpl {
    pub async fn new() -> Self {
        ServerRepositoryImpl { db: conn().await }
    }
}

pub const COLLECTION_NAME: &str = "server";

#[async_trait]
impl ServerRepository for ServerRepositoryImpl {
    async fn get_server(&self, id: &ServerId) -> Result<DbServer, String> {
        let res = self
            .db
            .select::<Option<DbServer>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_server(&self, server: &DbServer) -> Result<DbServer, String> {
        let created: DbServer = self
            .db
            .create((COLLECTION_NAME, server.id.to_string()))
            .content(server)
            .await
            .unwrap();
        dbg!(&created);

        Ok(created)
    }

    async fn update_server(&self, server: &DbServer) -> Result<DbServer, String> {
        let res: Option<DbServer> = self
            .db
            .update((COLLECTION_NAME, server.id.to_string()))
            .content(server.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete_server(&self, id: &ServerId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbServer>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_servers(&self) -> Result<Vec<DbServer>, String> {
        let res = self.db.select::<Vec<DbServer>>(COLLECTION_NAME).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(id: &ServerId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
