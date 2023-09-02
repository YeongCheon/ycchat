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

use super::conn;

const COLLECTION_NAME: &str = "channel";

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
    ) -> Result<Vec<DbChannel>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let res = db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE server == $server"
            ))
            .bind(("server", server))
            .await
            .unwrap()
            .take::<Vec<DbChannel>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(&self, db: &Surreal<Client>, channel: &DbChannel) -> Result<DbChannel, String> {
        let created: DbChannel = db
            .create((COLLECTION_NAME, channel.id.to_string()))
            .content(channel)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(&self, db: &Surreal<Client>, channel: &DbChannel) -> Result<DbChannel, String> {
        let res: Option<DbChannel> = db
            .update((COLLECTION_NAME, channel.id.to_string()))
            .content(channel.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
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
