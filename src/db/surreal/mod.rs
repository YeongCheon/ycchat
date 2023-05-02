// pub mod attachment;
pub mod auth;
pub mod server;
pub mod server_member;
pub mod user;

use serde::{Deserialize, Deserializer};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};

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

// for surrealdb
pub fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let id = Thing::deserialize(deserializer)?;

    Ok(id.id.to_string())
}
