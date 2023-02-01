use redis::{Commands, RedisResult};

use super::RedisClient;

impl RedisClient {
    pub fn add_room_member(
        &self,
        room_id: &String,
        user_id: &String,
        created_at: &i64,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zadd(key, user_id, created_at)
    }

    pub fn delete_room_member(&self, room_id: &String, user_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_room_members_key(room_id);

        conn.zrem(key, user_id)
    }

    pub fn get_room_members_all(&self, room_id: &String) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zrange(key, 0, -1)
    }

    pub fn get_room_members(
        &self,
        room_id: &String,
        start: isize,
        end: isize,
    ) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zrange(key, start, end)
    }

    pub fn get_room_member_score(&self, room_id: &String, user_id: &String) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zscore(key, user_id)
    }

    pub fn get_room_members_count(&self, room_id: &String) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zcount(key, 0, -1)
    }

    pub fn get_room_members_rank(&self, room_id: &String) -> RedisResult<Option<u64>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(room_id);

        conn.zrank(key, room_id)
    }

    fn generate_room_members_key(&self, room_id: &String) -> String {
        format!("ycchat:room:{}:members", room_id)
    }
}
