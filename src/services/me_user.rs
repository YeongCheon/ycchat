use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::user::UserRepository;

use super::ycchat::v1::models::User;
use super::ycchat::v1::services::me::user::{
    me_user_service_server::MeUserService as MeUserServer, GetMeRequest,
};

use crate::models::user::UserId;

pub struct MeUserService<U>
where
    U: UserRepository<Surreal<Client>>,
{
    user_repository: U,
}

impl<U> MeUserService<U>
where
    U: UserRepository<Surreal<Client>>,
{
    pub async fn new(user_repository: U) -> Self {
        MeUserService { user_repository }
    }
}

#[tonic::async_trait]
impl<U> MeUserServer for MeUserService<U>
where
    U: UserRepository<Surreal<Client>> + 'static,
{
    async fn get_me(&self, request: Request<GetMeRequest>) -> Result<Response<User>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let user = self.user_repository.get_user(&db, &user_id).await.unwrap();
        let user = match user {
            Some(user) => user,
            None => {
                return Err(Status::not_found("not exist"));
            }
        };

        Ok(Response::new(user.to_message()))
    }
}
