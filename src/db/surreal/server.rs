use serde::{Serialize, Serializer};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};
use tonic::async_trait;

use crate::{
    db::traits::server::ServerRepository,
    models::{
        server::{DbServer, ServerId},
        user::UserId,
    },
};

#[derive(Clone)]
pub struct ServerRepositoryImpl {}

impl ServerRepositoryImpl {
    pub async fn new() -> Self {
        ServerRepositoryImpl {}
    }
}

pub const COLLECTION_NAME: &str = "server";

#[async_trait]
impl ServerRepository<Surreal<Client>> for ServerRepositoryImpl {
    async fn get_server(
        &self,
        db: &Surreal<Client>,
        id: &ServerId,
    ) -> Result<Option<DbServer>, String> {
        let res = db
            .select::<Option<DbServer>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_server(
        &self,
        db: &Surreal<Client>,
        server: &DbServer,
    ) -> Result<Option<DbServer>, String> {
        let created: Option<DbServer> = db
            .create((COLLECTION_NAME, server.id.to_string()))
            .content(server)
            .await
            .unwrap();

        dbg!(&created);

        Ok(created)
    }

    async fn update_server(
        &self,
        db: &Surreal<Client>,
        server: &DbServer,
    ) -> Result<Option<DbServer>, String> {
        let res: Option<DbServer> = db
            .update((COLLECTION_NAME, server.id.to_string()))
            .content(server.clone())
            .await
            .unwrap();

        return Ok(res);
    }

    async fn delete_server(&self, db: &Surreal<Client>, id: &ServerId) -> Result<u8, String> {
        db.delete::<Option<DbServer>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_servers(
        &self,
        db: &Surreal<Client>,
        page_size: i32,
        offset_id: Option<ServerId>,
    ) -> Result<Vec<DbServer>, String> {
        let query = match offset_id {
            Some(offset_id) => db
                .query(format!(
                    "SELECT * FROM {COLLECTION_NAME} WHERE id < $offset_id ORDER BY id DESC LIMIT $page_size"
                ))
                .bind(("offset_id", offset_id))
                .bind(("page_size", page_size)),

            None => db.query(format!("SELECT * FROM {COLLECTION_NAME} ORDER BY id DESC LIMIT $page_size"))
                .bind(("page_size", page_size)),
        };

        let res = query.await.unwrap().take::<Vec<DbServer>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_joined_servers(
        &self,
        db: &Surreal<Client>,
        user_id: &UserId,
        page_size: i32,
        offset_id: Option<ServerId>,
    ) -> Result<Vec<DbServer>, String> {
        todo!()
    }
}

pub fn serialize_id<S>(id: &ServerId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
