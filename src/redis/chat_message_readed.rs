use redis::{Commands, RedisResult};

use super::RedisClient;

impl RedisClient {
    fn set(&self, room_id: &String, user_id: &String, message_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_message_readed_key(room_id);

        conn.hset(key, user_id, message_id)
    }

    fn generate_message_readed_key(&self, room_id: &String) -> String {
        format!("ycchat::room::{}::readed", room_id)
    }
}
