use redis::{Commands, RedisResult};

use crate::chat::ycchat::ChatMessage;

use super::RedisClient;

impl RedisClient {
    pub fn add_chat_room_message(
        &self,
        room_id: &String,
        message_id: &String,
        message: &ChatMessage,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_chat_room_messages_key(room_id);

        conn.hset(key, room_id, message)
    }

    pub fn get_chat_room_message(
        &self,
        room_id: &String,
        message_id: &String,
    ) -> RedisResult<ChatMessage> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_chat_room_messages_key(room_id);

        conn.hget(key, message_id)
    }

    pub fn delete_chat_room_message(
        &self,
        room_id: &String,
        message_id: &String,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_chat_room_messages_key(room_id);

        conn.hdel(key, message_id)
    }

    pub fn get_chat_room_messages(
        &self,
        room_id: &String,
        message_id_list: &[String],
    ) -> RedisResult<Vec<Option<ChatMessage>>> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_chat_room_messages_key(room_id);

        conn.hget(key, message_id_list)
    }

    pub fn get_chat_room_message_count(&self, room_id: &String) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_chat_room_messages_key(room_id);

        conn.hlen(key)
    }

    fn generate_chat_room_messages_key(&self, room_id: &String) -> String {
        format!("ycchat:room:{}:messages", room_id)
    }
}
