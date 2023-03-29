use std::{future::Future, pin::Pin};

use tonic::{Request, Response, Status};

use self::ycchat_user::{
    CreateUserRequest, DeleteUserRequest, GetUserRequest, GetUserResponse, ListUsersRequest,
    ListUsersResponse, UpdateUserRequest,
};
use model::User;
use ycchat_user::user_server::User as UserServer;

pub mod model {
    tonic::include_proto!("ycchat.model");
}

pub mod ycchat_user {
    tonic::include_proto!("ycchat.user");
}

pub struct UserService {}

impl UserService {
    pub fn new() -> Self {
        UserService {}
    }
}

#[tonic::async_trait]
impl UserServer for UserService {
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        todo!("not implemented yet.");
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        todo!("not implemented yet.");
    }

    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        todo!("not implemented yet.");
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        todo!("not implemented yet.");
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        todo!("not implemented yet.");
    }
}
