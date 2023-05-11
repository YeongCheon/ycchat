use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use tonic::{Request, Response, Status};
use ulid::Ulid;

use crate::auth::jwt::{decode, generate_access_token, generate_refresh_token};
use crate::db::traits::auth::AuthRepository;
use crate::models::auth::DbAuth;
use crate::models::user::UserId;
use crate::redis::RedisClient;

use super::ycchat_auth::auth_server::Auth;
use super::ycchat_auth::{
    RefreshTokenRequest, RefreshTokenResponse, RevokeRefreshTokenRequest, SignInRequest,
    SignInResponse, SignUpRequest, SignUpResponse,
};

pub struct AuthService<U>
where
    U: AuthRepository,
{
    redis_client: RedisClient,
    auth_repository: U,
}

impl<U> AuthService<U>
where
    U: AuthRepository,
{
    pub fn new(auth_repository: U) -> Self {
        let redis_client = RedisClient::new();

        AuthService {
            redis_client,
            auth_repository,
        }
    }

    fn get_user_id(&self, refresh_token: &str) -> Result<UserId, Status> {
        let token_data = match decode(refresh_token) {
            Ok(res) => res,
            Err(err) => {
                return Err(Status::unauthenticated(err.to_string()));
            }
        };

        Ok(token_data.claims.aud) // aud is user_id
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

        let exist = self
            .auth_repository
            .get_by_username(&req.username)
            .await
            .unwrap();

        if exist.is_some() {
            return Err(Status::already_exists("username already exist."));
        }

        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let hashed_password = match argon2.hash_password(req.password.as_bytes(), &salt) {
            Ok(res) => res.to_string(),
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        let user_id = Ulid::new();

        let res = self
            .auth_repository
            .add(&DbAuth {
                id: user_id.clone(),
                username: req.username,
                password: hashed_password,
                email: None,
                is_email_verified: false,
                create_time: chrono::offset::Utc::now(),
                update_time: None,
                last_login_time: None,
            })
            .await
            .unwrap();

        let access_token = generate_access_token(&user_id).unwrap();

        let refresh_token = generate_refresh_token(&user_id).unwrap();
        self.redis_client.set_refresh_token(&refresh_token).unwrap();

        Ok(Response::new(SignUpResponse {
            user_id: res.id.to_string(),
            access_token,
            refresh_token,
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

        let mut auth = match auth {
            Some(auth) => auth,
            None => return Err(Status::invalid_argument("invalid argument")),
        };

        let parsed_hash = PasswordHash::new(&auth.password).unwrap();

        let argon2 = Argon2::default();

        assert!(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok());

        auth.last_login_time = Some(chrono::offset::Utc::now());
        self.auth_repository.update(&auth).await.unwrap();

        let user_id = auth.id;

        let access_token = generate_access_token(&user_id).unwrap();
        let refresh_token = generate_refresh_token(&user_id).unwrap();
        self.redis_client.set_refresh_token(&refresh_token).unwrap();

        Ok(Response::new(SignInResponse {
            user_id: user_id.to_string(),
            access_token,
            refresh_token,
            expires_in: 3600,
        }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let old_refresh_token = request.into_inner().refresh_token;
        let user_id = self.get_user_id(&old_refresh_token)?;

        let res = self
            .redis_client
            .get_refresh_token(&old_refresh_token)
            .unwrap();

        if res.is_none() {
            return Err(Status::unauthenticated("invalid argument"));
        }

        let access_token = generate_access_token(&user_id).unwrap();
        let new_refresh_token = generate_refresh_token(&user_id).unwrap();

        self.redis_client
            .delete_refresh_token(&old_refresh_token)
            .unwrap();

        self.redis_client
            .set_refresh_token(&new_refresh_token)
            .unwrap();

        Ok(Response::new(RefreshTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: 3600,
        }))
    }

    async fn revoke_refresh_token(
        &self,
        request: Request<RevokeRefreshTokenRequest>,
    ) -> Result<Response<()>, Status> {
        let refresh_token = request.into_inner().refresh_token;

        self.redis_client
            .delete_refresh_token(&refresh_token)
            .unwrap();

        Ok(Response::new(()))
    }
}
