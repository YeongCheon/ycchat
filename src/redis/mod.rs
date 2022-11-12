use std::env;

use redis::{Commands, ConnectionLike, Msg, RedisResult};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

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

        let mut client = match redis::Client::open(redis_conn_url) {
            Ok(client) => client,
            Err(err) => panic!("{}", err),
        };

        RedisClient { client }
    }

    pub fn add_room_member(&self, room_id: &String, user_id: String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.sadd(key, user_id)
    }

    pub fn delete_room_member(&self, room_id: &String, user_id: String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_room_members_key(room_id);

        conn.srem(key, user_id)
    }

    pub fn get_room_members(&self, room_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.smembers(key)
    }

    pub fn chat_subscribe(&self, tx: Sender<String>) {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        tokio::spawn(async move {
            let mut pubsub = con.as_pubsub();
            pubsub.subscribe(channel).unwrap();

            println!("start subscribe");

            while let Ok(msg) = pubsub.get_message() {
                let payload: String = msg.get_payload().unwrap();
                tx.send(payload).await.unwrap();
            }
        });
    }

    pub fn chat_publish(
        &self,
        room_id: &String,
        message: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        con.publish(channel, message)?;

        Ok(())
    }

    fn generate_room_members_key(&self, room_id: &String) -> String {
        format!("room::{}::members", room_id)
    }

    fn generate_chat_pubsub_key(&self) -> String {
        "hello world".to_string()
    }
}

impl Clone for RedisClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}
