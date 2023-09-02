use redis::{Commands, RedisResult};

use crate::models::channel::ChannelId;

use super::RedisClient;

impl RedisClient {
    pub fn get_unread_message_count(
        &self,
        user_id: &String,
        channel_id: &ChannelId,
    ) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_unread_message_count_key(user_id, channel_id);

        conn.get(key)
    }

    pub fn get_unread_message_counts(
        &self,
        user_id: &String,
        channel_id_list: &[ChannelId],
    ) -> RedisResult<Vec<Option<i64>>> {
        let mut conn = self.client.get_connection().unwrap();

        let mut keys: Vec<String> = channel_id_list
            .iter()
            .map(|channel_id| {
                self.generate_member_room_unread_message_count_key(user_id, channel_id)
            })
            .collect();

        keys.iter().for_each(|key| println!("{}", key));

        conn.get(keys)
    }

    pub fn set(
        &self,
        user_id: &String,
        channel_id: &ChannelId,
        unread_message_count: u64,
    ) -> RedisResult<()> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_unread_message_count_key(user_id, channel_id);

        conn.set(key, unread_message_count)
    }

    pub fn incr(&self, user_id: &String, channel_id: &ChannelId) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_unread_message_count_key(user_id, channel_id);

        conn.incr(key, 1)
    }

    pub fn decr(&self, user_id: &String, channel_id: &ChannelId) -> RedisResult<i64> {
        let mut conn = self.client.get_connection().unwrap();

        let key = self.generate_member_room_unread_message_count_key(user_id, channel_id);

        conn.decr(key, 1)
    }

    fn generate_member_room_unread_message_count_key(
        &self,
        user_id: &String,
        channel_id: &ChannelId,
    ) -> String {
        format!(
            "ycchat:members:{}:rooms:{}:unreadCount",
            user_id, channel_id
        )
    }
}
