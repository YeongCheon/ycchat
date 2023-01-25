use core::fmt;
use std::str::from_utf8;

use redis::{Commands, ErrorKind, Expiry, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::RedisClient;

const EXPIRE: usize = 1800; // seconds

impl RedisClient {
    pub fn set_page_token(
        &self,
        token_key: PageTokenKey,
        page_token: PageToken,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_page_token_key_pattern(token_key);

        conn.set_ex(key, page_token, EXPIRE)
    }

    pub fn get_page_token(&self, token_key: PageTokenKey) -> RedisResult<PageToken> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_page_token_key_pattern(token_key);

        conn.get_ex(key, Expiry::EX(EXPIRE))
    }

    fn generate_page_token_key_pattern(&self, key: PageTokenKey) -> String {
        let page_type = key.to_string();

        let (owner_id, ulid) = match key {
            PageTokenKey::ChatMessageList { owner_id, ulid } => (owner_id, ulid),
            PageTokenKey::ChatRoomList { owner_id, ulid } => (owner_id, ulid),
            PageTokenKey::ChatRoomUserList { owner_id, ulid } => (owner_id, ulid),
        };

        format!(
            "ycchat:pages:{}:members:{}:token:{}",
            page_type,
            owner_id,
            ulid.to_string()
        )
    }
}

pub enum PageTokenKey {
    ChatMessageList { owner_id: String, ulid: Ulid },
    ChatRoomList { owner_id: String, ulid: Ulid },
    ChatRoomUserList { owner_id: String, ulid: Ulid },
}

impl fmt::Display for PageTokenKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PageTokenKey::ChatMessageList { owner_id, ulid } => "chatMessageList",
            PageTokenKey::ChatRoomList { owner_id, ulid } => "chatRoomList",
            PageTokenKey::ChatRoomUserList { owner_id, ulid } => "chatRoomUserList",
        };
        write!(f, "{}", name)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PageToken {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub offset_id: Option<String>, // ulid
    pub size: u64,
    pub order_by: Option<String>,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
}

impl PageToken {
    pub fn new(offset_id: Option<String>, size: u64, order_by: Option<String>) -> Self {
        PageToken {
            id: Some(Ulid::new().to_string()),
            offset_id,
            size,
            order_by,
            next_page_token: None,
            prev_page_token: None,
        }
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn set_next_page_token(&mut self, next_page_token: String) {
        self.next_page_token = Some(next_page_token);
    }

    pub fn set_prev_page_token(&mut self, prev_page_token: String) {
        self.prev_page_token = Some(prev_page_token);
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
