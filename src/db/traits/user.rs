use tonic::async_trait;

use crate::models::user::{User, UserId};

pub trait UserRepository {
    fn get_user(id: &UserId) -> Result<User, String>;
    fn add_user(user: &User) -> Result<String, String>;
    fn update_user(user: &User) -> Result<String, String>;
    fn delete_user(id: &UserId) -> Result<u8, String>;
    fn get_users() -> Result<Vec<User>, String>;
}
