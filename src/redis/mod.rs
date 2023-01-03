use std::env;

use prost::Message;
use redis::{ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs};

use crate::chat::ycchat::ChatMessage;

mod chat_latest_message;
mod chat_member_rooms;
mod chat_message_readed;
mod chat_pubsub;
mod chat_room_latest_message;
mod chat_room_members;
mod chat_room_message;
mod chat_unread_message_count;

#[derive(Debug)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new() -> Self {
        let redis_host_name = "127.0.0.1";
        let redis_password = "";

        let uri_scheme = match env::var("IS_TLS") {
            Ok(_) => "reidss",
            Err(_) => "redis",
        };

        let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_password, redis_host_name);
        println!("{}", redis_conn_url);

        let client = match redis::Client::open(redis_conn_url) {
            Ok(client) => client,
            Err(err) => panic!("{}", err),
        };

        RedisClient { client }
    }
}

impl Clone for RedisClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl ToRedisArgs for ChatMessage {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(&self.encode_to_vec());
    }
}

impl FromRedisValue for ChatMessage {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        match v {
            redis::Value::Data(binary_data) => {
                let buf = &binary_data[..];
                let receive_message = ChatMessage::decode(buf).unwrap();

                RedisResult::Ok(receive_message)
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
