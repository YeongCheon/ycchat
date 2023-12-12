use std::sync::Arc;

use ulid::Ulid;

use crate::redis::page_token::{PageToken, PageTokenKey};

use super::{chat::Shared, paging::Pager, ycchat::ChatMessage};

pub struct ChatMessagePager {
    shared: Arc<Shared>,
}

impl ChatMessagePager {
    pub fn new(shared: Arc<Shared>) -> Self {
        ChatMessagePager { shared }
    }

    fn get_room_id(&self, parent: &str) -> String {
        let parent_slice: Vec<&str> = parent.split('/').collect();

        parent_slice[1].to_string()
    }
}

impl Pager<ChatMessage> for ChatMessagePager {
    fn get_total_size(&self, id: &String) -> u64 {
        self.shared
            .redis_client
            .get_latest_room_message_count(id)
            .unwrap()
    }

    fn get_list(&self, name: &String, start: isize, end: isize) -> Vec<ChatMessage> {
        let room_id = self.get_room_id(name);

        let messages = {
            let message_ids = self
                .shared
                .redis_client
                .get_latest_room_message_list(&room_id, start, end)
                .unwrap();

            let messages_vec = if !message_ids.is_empty() {
                self.shared
                    .redis_client
                    .get_chat_room_messages(&room_id, &message_ids)
                    .unwrap()
            } else {
                vec![]
            };

            message_ids
                .iter()
                .enumerate()
                .filter(|(idx, message_id)| messages_vec[*idx].is_some())
                .map(|(idx, message_id)| -> ChatMessage {
                    let res = &messages_vec[idx];
                    res.clone().unwrap() // FIXME: remove clone
                })
                .collect()
        };

        messages
    }

    fn get_offset_id(&self, item: &Option<&ChatMessage>) -> Option<String> {
        item.map(|chat_user| {
            let name = chat_user.name.clone();
            // ex: rooms/room1/messages/message1
            let splited_list: Vec<String> = name.split('/').map(|item| item.to_string()).collect();

            splited_list[3].clone()
        })
    }

    fn get_start_index(&self, room_id: &String, offset_id: &Option<String>) -> isize {
        let start = if let Some(offset_id) = offset_id {
            self.shared
                .redis_client
                .get_latest_room_message_rank(room_id, offset_id)
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

        PageTokenKey::ChatMessageList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }

    fn generate_page_token_key(&self, user_id: &str, id: &str) -> PageTokenKey {
        let ulid = Ulid::from_string(id).unwrap();

        PageTokenKey::ChatMessageList {
            owner_id: user_id.to_string(),
            ulid,
        }
    }
}
