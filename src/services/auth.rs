use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tonic::{Request, Response, Status};
use ulid::Ulid;

use crate::auth::jwt::generate_jwt_token;
use crate::db::traits::auth::AuthRepository;
use crate::models::auth::DbAuth;

use super::ycchat_auth::auth_server::Auth;
use super::ycchat_auth::{
    RefreshTokenRequest, RefreshTokenResponse, RevokeRefreshTokenRequest, SignInRequest,
    SignInResponse, SignUpRequest, SignUpResponse,
};

pub struct AuthService<U>
where
    U: AuthRepository,
{
    auth_repository: U,
}

impl<U> AuthService<U>
where
    U: AuthRepository,
{
    pub fn new(auth_repository: U) -> Self {
        AuthService { auth_repository }
    }
}

#[tonic::async_trait]
impl<U> Auth for AuthService<U>
where
    U: AuthRepository + 'static,
{
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        let req = request.into_inner();

        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let hashed_password = match argon2.hash_password(req.password.as_bytes(), &salt) {
            Ok(res) => res.to_string(),
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        let user_id = Ulid::new().to_string();

        let res = self
            .auth_repository
            .add(&DbAuth {
                id: user_id.clone(),
                username: req.username,
                password: hashed_password,
            })
            .await
            .unwrap();

        let jwt_token = generate_jwt_token(&user_id).unwrap();

        Ok(Response::new(SignUpResponse {
            user_id: res.id,
            access_token: jwt_token,
            refresh_token: "FIXME".to_string(),
            expires_in: 3600,
        }))
    }

    async fn sign_in(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        let req = request.into_inner();
        let username = &req.username;
        let password = &req.password;

        let auth = match self.auth_repository.get_by_username(username).await {
            Ok(res) => res,
            Err(err) => return Err(Status::invalid_argument(err)),
        };

        let auth = match auth {
            Some(auth) => auth,
            None => return Err(Status::invalid_argument("invalid argument")),
        };

        let parsed_hash = PasswordHash::new(&auth.password).unwrap();

        let argon2 = Argon2::default();

        assert!(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok());

        let user_id = auth.id;

        let jwt_token = generate_jwt_token(&user_id).unwrap();

        Ok(Response::new(SignInResponse {
            user_id: user_id,
            access_token: jwt_token,
            refresh_token: "FIXME".to_string(),
            expires_in: 3600,
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        todo!()
    }

    async fn revoke_refresh_token(
        &self,
        request: Request<RevokeRefreshTokenRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
}
