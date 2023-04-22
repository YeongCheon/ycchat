// pub mod attachment;
pub mod server;
pub mod user;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
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
