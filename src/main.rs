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
    ycchat::v1::services::{
        account::account_service_server,
        auth::auth_service_server,
        channel::channel_service_server,
        connect::connect_service_server,
        me::user::me_user_service_server,
        message::message_service_server,
        server::member::server_member_service_server,
        server::{category::category_service_server, server_service_server},
        user::user_service_server,
    },
};
// use services::server::member::server_member_server::ServerMember as ServerMemberServer;
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

    let auth_service_server =
        auth_service_server::AuthServiceServer::new(AuthService::new(auth_repository.clone()));

    let message_service_server = message_service_server::MessageServiceServer::with_interceptor(
        MessageService::new(
            message_repository.clone(),
            message_acknowledge_repository,
            server_member_repository.clone(),
            channel_repository.clone(),
        ),
        interceptor::auth::check_auth,
    );

    let connect_service_server = connect_service_server::ConnectServiceServer::with_interceptor(
        ConnectService::new(broadcaster_arc.clone()),
        interceptor::auth::check_auth,
    );

    let account_service_server = account_service_server::AccountServiceServer::with_interceptor(
        AccountService::new(auth_repository),
        interceptor::auth::check_auth,
    );

    // // let chat_service_service_server = chat::get_chat_service_service_server();
    let user_service_server = user_service_server::UserServiceServer::with_interceptor(
        services::user::UserService::new(user_repository.clone()).await,
        interceptor::auth::check_auth,
    );

    let me_user_service_server = me_user_service_server::MeUserServiceServer::with_interceptor(
        services::me_user::MeUserService::new(user_repository).await,
        interceptor::auth::check_auth,
    );

    let server_service_server = server_service_server::ServerServiceServer::with_interceptor(
        services::server::ServerService::new(
            server_repository.clone(),
            server_member_repository.clone(),
        ),
        interceptor::auth::check_auth,
    );

    let server_category_service_server =
        category_service_server::CategoryServiceServer::with_interceptor(
            services::server_category::ServerCategoryService::new(
                server_repository.clone(),
                server_category_repository.clone(),
            ),
            interceptor::auth::check_auth,
        );

    let server_member_service_server =
        server_member_service_server::ServerMemberServiceServer::with_interceptor(
            services::server_member::ServerMemberService::new(server_member_repository.clone()),
            interceptor::auth::check_auth,
        );

    let channel_service_server = channel_service_server::ChannelServiceServer::with_interceptor(
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
        .add_service(connect_service_server)
        .add_service(auth_service_server)
        .add_service(account_service_server)
        .add_service(user_service_server)
        .add_service(server_service_server)
        .add_service(server_category_service_server)
        .add_service(server_member_service_server)
        .add_service(channel_service_server)
        .add_service(message_service_server)
        .add_service(me_user_service_server)
        .serve(addr)
        .await?;

    // println!("Start Server...");
    // Server::builder()
    //     // .add_service(chat_service_service_server)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
