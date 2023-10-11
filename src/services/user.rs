use prost::Message as _;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Datetime;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::user::UserRepository;
use crate::util::pager::PageTokenizer;

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
        let request = request.into_inner();
        let db = conn().await;
        let page_token = match request.page_token.clone() {
            Some(page_token) => {
                let page_token = crate::util::pager::get_page_token(page_token);
                Some(page_token.unwrap())
            }
            None => None,
        };

        let (page_size, offset_id, prev_page_token) = match page_token {
            Some(page_token) => (
                page_token.page_size,
                page_token
                    .offset_id
                    .map(|offset_id| UserId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        let mut list = self
            .user_repository
            .get_users(&db, page_size + 1, offset_id)
            .await
            .unwrap();

        let next_page_token = if list.len() > usize::try_from(page_size).unwrap() {
            list.pop();

            let next_page_token = list.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                crate::util::base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        let users: Vec<User> = list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListUsersResponse {
            users,
            next_page_token,
            prev_page_token,
        };

        Ok(Response::new(res))
    }

    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let db = conn().await;

        let req = request.into_inner();
        let name = req.name;

        let id = UserId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let user = self.user_repository.get_user(&db, &id).await.unwrap();
        let user = match user {
            Some(user) => user,
            None => {
                return Err(Status::not_found("not exist"));
            }
        };

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

        let res = self.user_repository.add_user(&db, &user).await.unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
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

        let exist_user = self.user_repository.get_user(&db, &user.id).await.unwrap();

        let mut exist_user = match exist_user {
            Some(exist_user) => exist_user,
            None => {
                return Err(Status::not_found("not exist"));
            }
        };

        exist_user.display_name = user.display_name;
        exist_user.description = user.description;
        exist_user.update_time = Some(Datetime::default());

        let res = self
            .user_repository
            .update_user(&db, &exist_user)
            .await
            .unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
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
