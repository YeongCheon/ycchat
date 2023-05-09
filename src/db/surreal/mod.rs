// pub mod attachment;
pub mod auth;
pub mod channel;
pub mod server;
pub mod server_category;
pub mod server_member;
pub mod user;

use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};
use ulid::Ulid;

async fn conn() -> Surreal<Client> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();

    db.use_ns("ycchat").use_db("ycchat").await.unwrap();

    db
}

pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let id = Thing::deserialize(deserializer)?;

    Ok(id.id.to_string())
}

pub fn deserialize_ulid_id<'de, D>(deserializer: D) -> Result<Ulid, D::Error>
where
    D: Deserializer<'de>,
{
    let id = Thing::deserialize(deserializer)?;

    let res = Ulid::from_string(&id.id.to_string()).unwrap();

    Ok(res)
}
