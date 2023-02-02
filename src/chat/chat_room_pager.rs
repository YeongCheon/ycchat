use std::sync::Arc;

use ulid::Ulid;

use crate::redis::page_token::{PageToken, PageTokenKey};

use super::{
    chat::Shared,
    paging::Pager,
    ycchat::{chat_room::ChatRoomType, ChatRoom},
};

pub struct ChatRoomPager {
    shared: Arc<Shared>,
}

impl ChatRoomPager {
    pub fn new(shared: Arc<Shared>) -> Self {
        ChatRoomPager { shared }
    }
}

impl Pager<ChatRoom> for ChatRoomPager {
    fn get_total_size(&self, user_id: &String) -> u64 {
        self.shared.redis_client.get_rooms_count(user_id).unwrap()
    }

    fn get_list(&self, user_id: &String, start: isize, end: isize) -> Vec<ChatRoom> {
        let room_ids = self
            .shared
            .redis_client
            .get_rooms(user_id, start, end)
            .unwrap();
        let unread_count_list = if !room_ids.is_empty() {
            self.shared
                .redis_client
                .get_unread_message_counts(user_id, &room_ids)
                .unwrap_or_default()
        } else {
            vec![]
        };

        let rooms: Vec<ChatRoom> = room_ids
            .iter()
            .enumerate()
            .map(|(idx, room_id)| ChatRoom {
                name: format!("rooms/{}", room_id),
                chat_room_type: ChatRoomType::Public as i32,
                unread_message_count: if unread_count_list.is_empty() {
                    0
                } else {
                    unread_count_list[idx].unwrap_or(0) as u64
                },
            })
            .collect();

        rooms
    }

    fn get_offset_id(&self, item: &Option<&ChatRoom>) -> Option<String> {
        item.map(|chat_room| {
            let name = chat_room.name.clone();
            let splited_list: Vec<String> = name.split('/').map(|item| item.to_string()).collect();

            splited_list[1].clone()
        })
    }

    fn get_start_index(&self, user_id: &String, offset_id: &Option<String>) -> isize {
        let start = if let Some(offset_id) = offset_id {
            self.shared
                .redis_client
                .get_rooms_rank(user_id, offset_id)
                .unwrap()
                .map_or(0, |start| start + 1)
        } else {
            0
        };

        isize::try_from(start).unwrap_or(0)
    }

    fn set_page_token(&self, page_token_key: PageTokenKey, page_token: &PageToken) {
        self.shared
            .redis_client
            .set_page_token(page_token_key, page_token.clone())
            .unwrap();
    }

    fn get_page_token(&self, page_token_key: PageTokenKey) -> Option<PageToken> {
        let page_token = self
            .shared
            .redis_client
            .get_page_token(page_token_key)
            .unwrap();

        Some(page_token)
    }

    fn generate_new_page_token_key(&self, user_id: &str) -> PageTokenKey {
        let ulid = Ulid::new();

        PageTokenKey::ChatRoomList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }

    fn generate_page_token_key(&self, user_id: &str, id: &str) -> PageTokenKey {
        let ulid = Ulid::from_string(id).unwrap();

        PageTokenKey::ChatRoomList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }
}
