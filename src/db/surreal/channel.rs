use serde::Serializer;
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

pub struct ChannelRepositoryImpl {
    db: Surreal<Client>,
}

impl ChannelRepositoryImpl {
    pub async fn new() -> Self {
        ChannelRepositoryImpl { db: conn().await }
    }
}

#[tonic::async_trait]
impl ChannelRepository for ChannelRepositoryImpl {
    async fn get(&self, id: &ChannelId) -> Result<Option<DbChannel>, String> {
        let res = self.db.select((COLLECTION_NAME, id.to_string())).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_server_channels(&self, server_id: &ServerId) -> Result<Vec<DbChannel>, String> {
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
            .take::<Vec<DbChannel>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add(&self, channel: &DbChannel) -> Result<DbChannel, String> {
        let created: DbChannel = self
            .db
            .create((COLLECTION_NAME, channel.id.to_string()))
            .content(channel)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update(&self, channel: &DbChannel) -> Result<DbChannel, String> {
        let res: Option<DbChannel> = self
            .db
            .update((COLLECTION_NAME, channel.id.to_string()))
            .content(channel.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete(&self, id: &ChannelId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbChannel>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }
}

pub fn serialize_id<S>(channel_id: &ChannelId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = format!("{}:{}", COLLECTION_NAME, channel_id);

    s.serialize_str(&surreal_id)
}
