use db::{
    surreal::{
        auth::AuthRepositoryImpl, server::ServerRepositoryImpl,
        server_member::ServerMemberRepositoryImpl, user::UserRepositoryImpl,
    },
    traits::auth::AuthRepository,
};
use services::{
    account::AccountService, auth::AuthService, ycchat_account::account_server,
    ycchat_auth::auth_server, ycchat_server::member::server_member_server,
    ycchat_server::server_server, ycchat_user::user_server,
};
// use services::ycchat_server::member::server_member_server::ServerMember as ServerMemberServer;
use tonic::transport::Server;

// mod chat;
mod auth;
mod db;
mod interceptor;
mod models;
mod redis;
mod services;
mod util;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let addr = "0.0.0.0:50051".parse().unwrap();

    let auth_repository = AuthRepositoryImpl::new().await;
    let user_repository = UserRepositoryImpl::new().await;
    let server_repository = ServerRepositoryImpl::new().await;
    let server_member_repository = ServerMemberRepositoryImpl::new().await;

    let auth_server = auth_server::AuthServer::new(AuthService::new(auth_repository.clone()));

    let account_server = account_server::AccountServer::with_interceptor(
        AccountService::new(auth_repository),
        interceptor::auth::check_auth,
    );

    // // let chat_service_server = chat::get_chat_service_server();
    let user_server = user_server::UserServer::with_interceptor(
        services::user::UserService::new(user_repository).await,
        interceptor::auth::check_auth,
    );

    let server_server = server_server::ServerServer::with_interceptor(
        services::server::ServerService::new(server_repository),
        interceptor::auth::check_auth,
    );

    let server_member_server = server_member_server::ServerMemberServer::with_interceptor(
        services::server_member::ServerMemberService::new(server_member_repository),
        interceptor::auth::check_auth,
    );

    Server::builder()
        .add_service(auth_server)
        .add_service(account_server)
        .add_service(user_server)
        .add_service(server_server)
        .add_service(server_member_server)
        .serve(addr)
        .await?;

    // println!("Start Server...");
    // Server::builder()
    //     // .add_service(chat_service_server)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
