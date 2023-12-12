use futures::lock::Mutex;
use std::sync::Arc;

use chat::broadcaster::Broadcaster;
use db::surreal::{
    auth::AuthRepositoryImpl, channel::ChannelRepositoryImpl, message::MessageRepositoryImpl,
    message_acknowledge::MessageAcknowledgeRepositoryImpl, server::ServerRepositoryImpl,
    server_category::ServerCategoryRepositoryImpl, server_member::ServerMemberRepositoryImpl,
    user::UserRepositoryImpl,
};
use services::{
    account::AccountService,
    auth::AuthService,
    connect::ConnectService,
    message::MessageService,
    ycchat_account::account_server,
    ycchat_auth::auth_server,
    ycchat_channel::channel_server,
    ycchat_connect::connect_server,
    ycchat_message::message_server,
    ycchat_server::member::server_member_server,
    ycchat_server::{category::category_server, server_server},
    ycchat_user::user_server,
};
// use services::ycchat_server::member::server_member_server::ServerMember as ServerMemberServer;
use tonic::transport::Server;

mod auth;
mod chat;
mod db;
mod interceptor;
mod models;
// mod redis;
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
    let server_category_repository = ServerCategoryRepositoryImpl::new().await;
    let server_member_repository = ServerMemberRepositoryImpl::new().await;
    let channel_repository = ChannelRepositoryImpl::new().await;
    let message_repository = MessageRepositoryImpl::new().await;
    let message_acknowledge_repository = MessageAcknowledgeRepositoryImpl::new().await;

    let broadcaster = Broadcaster::new();
    let broadcaster_arc: Arc<Mutex<Broadcaster>> = Arc::new(Mutex::new(broadcaster));

    let auth_server = auth_server::AuthServer::new(AuthService::new(auth_repository.clone()));

    let message_server = message_server::MessageServer::with_interceptor(
        MessageService::new(
            message_repository.clone(),
            message_acknowledge_repository,
            server_member_repository.clone(),
            channel_repository.clone(),
        ),
        interceptor::auth::check_auth,
    );

    let connect_server = connect_server::ConnectServer::with_interceptor(
        ConnectService::new(broadcaster_arc.clone()),
        interceptor::auth::check_auth,
    );

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
        services::server::ServerService::new(
            server_repository.clone(),
            server_member_repository.clone(),
        ),
        interceptor::auth::check_auth,
    );

    let server_category_server = category_server::CategoryServer::with_interceptor(
        services::server_category::ServerCategoryService::new(
            server_repository.clone(),
            server_category_repository.clone(),
        ),
        interceptor::auth::check_auth,
    );

    let server_member_server = server_member_server::ServerMemberServer::with_interceptor(
        services::server_member::ServerMemberService::new(server_member_repository.clone()),
        interceptor::auth::check_auth,
    );

    let channel_server = channel_server::ChannelServer::with_interceptor(
        services::channel::ChannelService::new(
            server_member_repository,
            message_repository,
            channel_repository,
            server_repository,
            server_category_repository,
            broadcaster_arc.clone(),
        ),
        interceptor::auth::check_auth,
    );

    Server::builder()
        .add_service(connect_server)
        .add_service(auth_server)
        .add_service(account_server)
        .add_service(user_server)
        .add_service(server_server)
        .add_service(server_category_server)
        .add_service(server_member_server)
        .add_service(channel_server)
        .add_service(message_server)
        .serve(addr)
        .await?;

    // println!("Start Server...");
    // Server::builder()
    //     // .add_service(chat_service_server)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
