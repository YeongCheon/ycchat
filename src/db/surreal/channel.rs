use serde::{Serialize, Serializer};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Id, Thing},
    Surreal,
};

use crate::{
    db::traits::channel::ChannelRepository,
    models::channel::{ChannelId, DbChannel},
    models::server::ServerId,
};

use super::server::COLLECTION_NAME as SERVER_COLLECTION_NAME;

pub const COLLECTION_NAME: &str = "channel";

#[derive(Clone)]
pub struct ChannelRepositoryImpl {}

impl ChannelRepositoryImpl {
    pub async fn new() -> Self {
        ChannelRepositoryImpl {}
    }
}

#[tonic::async_trait]
impl ChannelRepository<Surreal<Client>> for ChannelRepositoryImpl {
    async fn get(&self, db: &Surreal<Client>, id: &ChannelId) -> Result<Option<DbChannel>, String> {
        let res = db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_server_channels(
        &self,
        db: &Surreal<Client>,
        server_id: &ServerId,
        page_size: i32,
        offset_id: Option<ChannelId>,
    ) -> Result<Vec<DbChannel>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let query = match offset_id {
            Some(offset_id) => db
                .query(format!(
                    "SELECT * FROM {COLLECTION_NAME} WHERE server == $server AND id < $offset_id ORDER BY id DESC LIMIT $page_size"
                ))
                .bind(("server", server))
                .bind(("offset_id", offset_id))
                .bind(("page_size", page_size)),

            None => db
                .query(format!(
                    "SELECT * FROM {COLLECTION_NAME} WHERE server == $server ORDER BY id DESC LIMIT $page_size"
                ))
                .bind(("server", server))
                .bind(("page_size", page_size)),
        };

        let res = query.await.unwrap().take::<Vec<DbChannel>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(
        &self,
        db: &Surreal<Client>,
        channel: &DbChannel,
    ) -> Result<Option<DbChannel>, String> {
        let created: Option<DbChannel> = db
            .create((COLLECTION_NAME, channel.id.to_string()))
            .content(channel)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(
        &self,
        db: &Surreal<Client>,
        channel: &DbChannel,
    ) -> Result<Option<DbChannel>, String> {
        let res: Option<DbChannel> = db
            .update((COLLECTION_NAME, channel.id.to_string()))
            .content(channel.clone())
            .await
            .unwrap();

        return Ok(res);
    }

    async fn delete(&self, db: &Surreal<Client>, id: &ChannelId) -> Result<u8, String> {
        db.delete::<Option<DbChannel>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }
}

pub fn serialize_id<S>(id: &ChannelId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
