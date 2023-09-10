use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::user::UserRepository;

use super::model::User;
use super::ycchat_user::user_server::User as UserServer;
use super::ycchat_user::{
    CreateUserRequest, DeleteUserRequest, GetUserRequest, ListUsersRequest, ListUsersResponse,
    UpdateUserRequest,
};
use crate::models::user::{DbUser, UserId};

pub struct UserService<U>
where
    U: UserRepository<Surreal<Client>>,
{
    user_repository: U,
}

impl<U> UserService<U>
where
    U: UserRepository<Surreal<Client>>,
{
    pub async fn new(user_repository: U) -> Self {
        UserService { user_repository }
    }
}

#[tonic::async_trait]
impl<U> UserServer for UserService<U>
where
    U: UserRepository<Surreal<Client>> + 'static,
{
    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let db = conn().await;

        let list = self.user_repository.get_users(&db).await.unwrap();

        let users: Vec<User> = list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListUsersResponse { users, page: None };

        Ok(Response::new(res))
    }

    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let db = conn().await;

        let req = request.into_inner();
        let name = req.name;

        let id = UserId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let user = self.user_repository.get_user(&db, &id).await.unwrap();

        Ok(Response::new(user.to_message()))
    }

    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<User>, tonic::Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let req = request.into_inner();

        let user = match req.user {
            Some(user) => {
                let mut user = DbUser::new(user);

                user.id = user_id;

                user
            }
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let user_res = self.user_repository.add_user(&db, &user).await.unwrap();

        Ok(Response::new(user_res.to_message()))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let db = conn().await;

        let req = request.into_inner();

        let user = match req.user {
            Some(user) => DbUser::from(user),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };
        dbg!(&user);

        let mut exist_user = self.user_repository.get_user(&db, &user.id).await.unwrap();

        exist_user.display_name = user.display_name;
        exist_user.description = user.description;
        exist_user.update_time = Some(chrono::offset::Utc::now());

        let res = self
            .user_repository
            .update_user(&db, &exist_user)
            .await
            .unwrap();
        Ok(Response::new(res.to_message()))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

        let req = request.into_inner();
        let name = req.name;

        let id = UserId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        self.user_repository.delete_user(&db, &id).await.unwrap();

        Ok(Response::new(()))
    }
}
