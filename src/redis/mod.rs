use std::env;

use prost::Message;
use redis::{Commands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use tokio::sync::mpsc::Sender;

use crate::chat::ycchat::ChatMessage;

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

    pub fn add_room_member(&self, room_id: &String, user_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.sadd(key, user_id)
    }

    pub fn delete_room_member(&self, room_id: &String, user_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_room_members_key(room_id);

        conn.srem(key, user_id)
    }

    pub fn get_room_members(&self, room_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.smembers(key)
    }

    pub fn get_rooms(&self, user_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.smembers(key)
    }

    pub fn chat_subscribe(&self, tx: Sender<ChatMessage>) {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        // tokio::task::spawn_blocking(move || tx.send(ReceiveMessageResponse::default()));

        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                let mut pubsub = con.as_pubsub();
                pubsub.subscribe(channel).unwrap();

                while let Ok(msg) = pubsub.get_message() {
                    let payload: ChatMessage = msg.get_payload().unwrap();
                    tx.blocking_send(payload).unwrap();
                }
            })
            .await
            .unwrap();
        });
    }

    pub fn chat_publish(&self, message: &ChatMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        con.publish(channel, message)?;

        Ok(())
    }

    fn generate_room_members_key(&self, room_id: &String) -> String {
        format!("ycchat::room::{}::members", room_id)
    }

    fn generate_chat_pubsub_key(&self) -> String {
        "ycchat::pubsub".to_string()
    }

    fn generate_member_room_key(&self, user_id: &String) -> String {
        format!("ycchat::member::{}:rooms", user_id)
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
