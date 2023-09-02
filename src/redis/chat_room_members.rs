use redis::{Commands, RedisResult};

use crate::models::channel::ChannelId;

use super::RedisClient;

impl RedisClient {
    #[deprecated]
    pub fn add_room_member(
        &self,
        channel_id: &ChannelId,
        user_id: &String,
        created_at: &i64,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zadd(key, user_id, created_at)
    }

    #[deprecated]
    pub fn delete_room_member(&self, channel_id: &ChannelId, user_id: &String) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();
        let key = self.generate_room_members_key(channel_id);

        conn.zrem(key, user_id)
    }

    #[deprecated]
    pub fn get_room_members_all(&self, channel_id: &ChannelId) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zrange(key, 0, -1)
    }

    #[deprecated]
    pub fn get_room_members(
        &self,
        channel_id: &ChannelId,
        start: isize,
        end: isize,
    ) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zrange(key, start, end)
    }

    #[deprecated]
    pub fn get_room_member_score(
        &self,
        channel_id: &ChannelId,
        user_id: &String,
    ) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zscore(key, user_id)
    }

    #[deprecated]
    pub fn get_room_members_count(&self, channel_id: &ChannelId) -> RedisResult<u64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zcount(key, 0, -1)
    }

    #[deprecated]
    pub fn get_room_members_rank(&self, channel_id: &ChannelId) -> RedisResult<Option<u64>> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_room_members_key(channel_id);

        conn.zrank(key, channel_id.to_string())
    }

    #[deprecated]
    fn generate_room_members_key(&self, channel_id: &ChannelId) -> String {
        format!("ycchat:room:{}:members", channel_id)
    }
}
