use db::postgres::user::UserRepositoryImpl;
use hyper::service::service_fn;
use services::{server::ycchat_server::server_server, user::ycchat_user::user_server};
use tonic::transport::Server;

// mod chat;
mod db;
mod interceptor;
mod models;
mod services;
// mod redis;
mod util;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let addr = "0.0.0.0:50051".parse().unwrap();

    // // let chat_service_server = chat::get_chat_service_server();
    let user_server = user_server::UserServer::with_interceptor(
        services::user::UserService::new(),
        interceptor::auth::check_auth,
    );

    let server_server = server_server::ServerServer::with_interceptor(
        services::server::ServerService::new(),
        interceptor::auth::check_auth,
    );

    Server::builder()
        .add_service(user_server)
        .add_service(server_server)
        .serve(addr)
        .await?;

    // println!("Start Server...");
    // Server::builder()
    //     // .add_service(chat_service_server)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
