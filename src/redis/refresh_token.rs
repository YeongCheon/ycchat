use redis::Commands;

use redis::{Expiry, RedisResult};

use super::RedisClient;

const EXPIRE: usize = 3600 * 24 * 14; // seconds (14 days)

impl RedisClient {
    pub fn set_refresh_token(&self, refresh_token: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_refresh_token_key(refresh_token);

        conn.set_ex(key, true, EXPIRE)
    }

    pub fn get_refresh_token(&self, refresh_token: &str) -> RedisResult<Option<bool>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_refresh_token_key(refresh_token);

        conn.get_ex(key, Expiry::EX(EXPIRE))
    }

    pub fn is_exist_refresh_token(&self, refresh_token: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_refresh_token_key(refresh_token);

        conn.exists(key)
    }

    pub fn delete_refresh_token(&self, refresh_token: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_refresh_token_key(refresh_token);

        conn.del(key)
    }

    fn generate_refresh_token_key(&self, refresh_token: &str) -> String {
        format!("ycchat:auth:refresh_token:{}", refresh_token)
    }
}
