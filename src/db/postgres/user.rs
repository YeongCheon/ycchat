use crate::db::traits::user::UserRepository;
use crate::models::user::{User, UserId};
use postgres::Client;

pub struct UserRepositoryImpl {
    client: Client,
}

impl UserRepositoryImpl {
    pub fn new() -> Self {
        UserRepositoryImpl {
            client: crate::db::postgres::generate_client(),
        }
    }
}

impl UserRepository for UserRepositoryImpl {
    fn get_user(id: &UserId) -> Result<User, String> {
        todo!()
    }

    fn add_user(user: &User) -> Result<String, String> {
        todo!()
    }

    fn update_user(user: &User) -> Result<String, String> {
        todo!()
    }

    fn delete_user(id: &UserId) -> Result<u8, String> {
        todo!()
    }

    fn get_users() -> Result<Vec<User>, String> {
        todo!()
    }
}
