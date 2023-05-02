use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use tonic::{Request, Response, Status};

use crate::db::traits::auth::AuthRepository;

use super::ycchat_account::{account_server::Account, UpdatePasswordRequest};

pub struct AccountService<U>
where
    U: AuthRepository,
{
    auth_repository: U,
}

impl<U> AccountService<U>
where
    U: AuthRepository,
{
    pub fn new(auth_repository: U) -> Self {
        AccountService { auth_repository }
    }
}

#[tonic::async_trait]
impl<U> Account for AccountService<U>
where
    U: AuthRepository + 'static,
{
    async fn update_password(
        &self,
        request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<()>, Status> {
        let user_id = request
            .metadata()
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let request = request.into_inner();

        let current_password = request.current_password;
        let new_password = request.new_password;

        let exist = self.auth_repository.get(&user_id).await.unwrap();

        let mut exist = match exist {
            Some(exist) => exist,
            None => return Err(Status::not_found("not found.")),
        };

        let parsed_hash = PasswordHash::new(&exist.password).unwrap();

        let argon2 = Argon2::default();

        assert!(argon2
            .verify_password(current_password.as_bytes(), &parsed_hash)
            .is_ok());

        let salt = SaltString::generate(&mut OsRng);

        let hashed_new_password = match argon2.hash_password(new_password.as_bytes(), &salt) {
            Ok(res) => res.to_string(),
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        exist.password = hashed_new_password;

        self.auth_repository.update(&exist).await.unwrap();

        Ok(Response::new(()))
    }
}
