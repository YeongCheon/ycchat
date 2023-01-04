use redis::{Commands, RedisResult};

use super::RedisClient;

impl RedisClient {
    pub fn set_latest_message(&self, room_id: &String, message_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_latest_message_key(room_id);

        conn.set(key, message_id)
    }

    pub fn delete_latest_message(&self, room_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_latest_message_key(room_id);

        conn.del(key)
    }

    fn generate_latest_message_key(&self, room_id: &String) -> String {
        format!("ycchat:room:{}:latestMessage", room_id)
    }
}
