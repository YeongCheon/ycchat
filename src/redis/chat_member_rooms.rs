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

    pub fn get_rooms_all(&self, user_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zrange(key, 0, -1)
    }

    pub fn get_rooms(
        &self,
        user_id: &String,
        start: isize,
        end: isize,
    ) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zrange(key, start, end)
    }

    pub fn get_rooms_rank(&self, user_id: &String, room_id: &String) -> RedisResult<Option<u64>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zrank(key, room_id)
    }

    pub fn get_rooms_count(&self, user_id: &String) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_key(user_id);

        conn.zcount(key, "-inf", "+inf")
    }

    fn generate_member_room_key(&self, user_id: &String) -> String {
        format!("ycchat:members:{}:rooms", user_id)
    }
}
