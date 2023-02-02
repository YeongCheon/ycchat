use std::sync::Arc;

use ulid::Ulid;

use crate::redis::page_token::{PageToken, PageTokenKey};

use super::{chat::Shared, paging::Pager, ycchat::ChatUser};

pub struct ChatRoomUserPager {
    shared: Arc<Shared>,
}

impl ChatRoomUserPager {
    pub fn new(shared: Arc<Shared>) -> Self {
        ChatRoomUserPager { shared }
    }
}

impl Pager<ChatUser> for ChatRoomUserPager {
    fn get_total_size(&self, id: &String) -> u64 {
        self.shared.redis_client.get_room_members_count(id).unwrap()
    }

    fn get_list(&self, room_id: &String, start: isize, end: isize) -> Vec<ChatUser> {
        let room_members = self
            .shared
            .redis_client
            .get_room_members(&room_id, start, end)
            .unwrap();

        let users = room_members
            .iter()
            .map(|room_member| ChatUser {
                name: format!("users/{}", room_member),
            })
            .collect();

        users
    }

    fn get_offset_id(&self, item: &Option<&ChatUser>) -> Option<String> {
        item.map(|chat_user| {
            let name = chat_user.name.clone();
            let splited_list: Vec<String> = name.split('/').map(|item| item.to_string()).collect();

            splited_list[1].clone()
        })
    }

    fn get_start_index(&self, user_id: &String, offset_id: &Option<String>) -> isize {
        let start = if let Some(offset_id) = offset_id {
            self.shared
                .redis_client
                .get_room_members_rank(offset_id)
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

        PageTokenKey::ChatRoomUserList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }

    fn generate_page_token_key(&self, user_id: &str, id: &str) -> PageTokenKey {
        let ulid = Ulid::from_string(id).unwrap();

        PageTokenKey::ChatRoomUserList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }
}
