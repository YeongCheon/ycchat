use std::str::from_utf8;

use redis::{Commands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::RedisClient;

const EXPIRE: usize = 1800; // seconds

impl RedisClient {
    pub fn set_page_token(
        &self,
        owner_id: &String,
        page_token_id: Ulid,
        page_token: PageToken,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_page_token_key(owner_id, page_token_id);

        conn.set_ex(key, page_token, EXPIRE)
    }

    pub fn get_page_token(&self, owner_id: &String, page_token_id: Ulid) -> RedisResult<PageToken> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_page_token_key(owner_id, page_token_id);

        conn.get(key)
    }

    fn generate_page_token_key(&self, owner_id: &String, ulid: Ulid) -> String {
        format!("ycchat:members:{}:pageToken:{}", owner_id, ulid.to_string())
    }
}

#[derive(Deserialize, Serialize)]
pub struct PageToken {
    page: u64,
    size: u64,
}

impl PageToken {
    pub fn new(page: u64, size: u64) -> Self {
        PageToken { page, size }
    }
}

impl ToRedisArgs for PageToken {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let serialized = serde_json::to_string(&self).unwrap();
        out.write_arg(serialized.as_bytes());
    }
}

impl FromRedisValue for PageToken {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        match v {
            redis::Value::Data(binary_data) => {
                let buf = &binary_data[..];

                let json_str = from_utf8(buf).unwrap();

                let page_token = serde_json::from_str(json_str);

                RedisResult::Ok(page_token.unwrap())
            }
            _ => RedisResult::Err(RedisError::from((
                ErrorKind::ResponseError,
                "decode fail",
                "fail to decode ReceiveMessage.".to_string(),
            ))),
        }
    }

    fn from_redis_values(items: &[redis::Value]) -> RedisResult<Vec<Self>> {
        items.iter().map(FromRedisValue::from_redis_value).collect()
    }

    fn from_byte_vec(_vec: &[u8]) -> Option<Vec<Self>> {
        None
    }
}
