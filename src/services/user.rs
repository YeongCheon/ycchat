use tonic::{Request, Response, Status};

use crate::{db::traits::user::UserRepository, models::user::UserId};

use self::ycchat_user::{
    CreateUserRequest, DeleteUserRequest, GetUserRequest, GetUserResponse, ListUsersRequest,
    ListUsersResponse, UpdateUserRequest,
};
use crate::models::user::DbUser;
use model::User;
use ycchat_user::user_server::User as UserServer;

pub mod model {
    tonic::include_proto!("ycchat.model");
}

pub mod ycchat_user {
    tonic::include_proto!("ycchat.user");
}

pub struct UserService<U>
where
    U: UserRepository,
{
    user_repository: U,
}

impl<U> UserService<U>
where
    U: UserRepository,
{
    pub async fn new(user_repository: U) -> Self {
        UserService { user_repository }
    }
}

#[tonic::async_trait]
impl<U> UserServer for UserService<U>
where
    U: UserRepository + 'static,
{
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let list = self.user_repository.get_users().await.unwrap();

        let users: Vec<User> = list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListUsersResponse { users, page: None };

        Ok(Response::new(res))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        let name = req.name;

        let id = name.split('/').collect::<Vec<&str>>()[1];
        let id = UserId::from_string(id).unwrap();

        let user = self.user_repository.get_user(&id).await.unwrap();

        Ok(Response::new(GetUserResponse {
            user: Some(user.to_message()),
        }))
    }

    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        let req = request.into_inner();

        let user = match req.user {
            Some(user) => DbUser::new(user),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let user_res = self.user_repository.add_user(&user).await.unwrap();

        Ok(Response::new(user_res.to_message()))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let req = request.into_inner();

        let user = match req.user {
            Some(user) => DbUser::from(user),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };
        dbg!(&user);

        let mut exist_user = self.user_repository.get_user(&user.id).await.unwrap();

        exist_user.display_name = user.display_name;
        exist_user.description = user.description;
        exist_user.update_time = chrono::offset::Utc::now();

        let res = self.user_repository.update_user(&exist_user).await.unwrap();
        Ok(Response::new(res.to_message()))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let name = req.name;

        let id = name.split('/').collect::<Vec<&str>>()[1];
        let id = UserId::from_string(id).unwrap();

        self.user_repository.delete_user(&id).await.unwrap();

        Ok(Response::new(()))
    }
}
