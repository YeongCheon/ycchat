use redis::{Commands, RedisResult};

use super::RedisClient;

impl RedisClient {
    pub fn add_room(
        &self,
        user_id: &String,
        room_id: &String,
        created_at: &i64,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zadd(key, room_id, created_at)
    }

    pub fn delete_room(&self, user_id: &String, room_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zrem(key, room_id)
    }

    pub fn get_rooms(&self, user_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zrange(key, 0, -1)
    }

    pub fn get_rooms_count(&self, user_id: &String) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zcount(key, 0, -1)
    }

    fn generate_member_room_key(&self, user_id: &String) -> String {
        format!("ycchat::member::{}:rooms", user_id)
    }
}
