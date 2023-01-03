use redis::{Commands, RedisResult};

use super::RedisClient;

impl RedisClient {
    pub fn add_latest_room_message(
        &self,
        room_id: &String,
        message_id: &String,
        created_at: &i64,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_latest_chat_room_messages_key(room_id);

        conn.zadd(key, message_id, created_at)
    }

    pub fn get_latest_room_message_list(&self, room_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_latest_chat_room_messages_key(room_id);

        conn.zrange(key, 0, -1)
    }

    pub fn get_latest_room_message_count(&self, room_id: &String) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_latest_chat_room_messages_key(room_id);

        conn.zcount(key, 0, -1)
    }

    fn generate_latest_chat_room_messages_key(&self, room_id: &String) -> String {
        format!("ycchat:room:{}:messages:latest", room_id)
    }
}
