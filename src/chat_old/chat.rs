use std::{collections::HashMap, pin::Pin, sync::Arc, time::SystemTime};

use chrono::{DateTime, Datelike, Timelike, Utc};
use prost_types::Timestamp;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{Result, Status};
use ulid::Ulid;

use crate::redis::{self as yc_redis, RedisClient};

use super::{
    chat_message_pager::ChatMessagePager,
    chat_room_pager::ChatRoomPager,
    chat_room_user_pager::ChatRoomUserPager,
    paging::PagingManager,
    ycchat::{
        chat_message::MessageType, connect_response::Payload, ChatMessage, ChatRoom, ChatUser,
        ConnectRequest, ConnectResponse, EntryChatRoomRequest, EntryChatRoomResponse,
        LeaveChatRoomRequest, LeaveChatRoomResponse, ListChatMessagesRequest,
        ListChatMessagesResponse, ListChatRoomUsersRequest, ListChatRoomUsersResponse,
        ListChatRoomsRequest, ListChatRoomsResponse, ReadChatMessageRequest,
        ReadChatMessageResponse, SpeechRequest, SpeechResponse,
    },
};

pub type UserId = String;

#[derive(Debug)]
pub struct Shared {
    pub redis_client: RedisClient,
    senders: RwLock<HashMap<UserId, mpsc::Sender<ConnectResponse>>>,
}

pub type ConnStream =
    Pin<Box<dyn Stream<Item = Result<ConnectResponse, Status>> + Send + Sync + 'static>>;

impl Shared {
    fn new() -> Self {
        let redis = yc_redis::RedisClient::new();

        let senders = RwLock::new(HashMap::new());
        Shared {
            redis_client: redis,
            senders,
        }
    }

    fn send_message(&self, msg: &ConnectResponse) {
        let message: Option<&ChatMessage> = if let Some(ref payload) = msg.payload {
            match payload {
                Payload::ConnectSuccess(_) => None,
                Payload::ChatMessage(item) => Some(item),
            }
        } else {
            None
        };

        if let Some(chat_message) = message {
            let room_id = chat_message.room_id.clone();
            let message_id = chat_message.name.clone();

            self.redis_client.chat_publish(chat_message).unwrap();
        }
    }

    async fn broadcast(&self, msg: &ChatMessage) {
        let parent = &msg.name;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        let room_members = self.redis_client.get_room_members_all(&room_id).unwrap();

        let read_guard = self.senders.read().await;

        let users = read_guard.clone(); // FIXME

        for (user_id, tx) in &users {
            if !room_members.contains(user_id) {
                continue;
            }

            let conn_response = ConnectResponse {
                id: ulid::Ulid::new().to_string(),
                payload: Some(Payload::ChatMessage(msg.clone())),
            };

            match tx.send(conn_response).await {
                Ok(_) => {}
                Err(_) => {
                    println!("[Broadcast] SendError: to {}, {:?}", user_id, msg)
                }
            }
        }
    }

    pub async fn incr_unread_message_count(&self, room_id: &String) {
        let members = self.redis_client.get_room_members_all(room_id).unwrap();

        members.iter().for_each(|user_id| {
            self.redis_client.incr(user_id, room_id).unwrap();
        });
    }
}

pub struct ChatServerService {
    shared: Arc<Shared>,
    chat_room_list_pager: PagingManager<ChatRoom, ChatRoomPager>,
    chat_room_member_list_pager: PagingManager<ChatUser, ChatRoomUserPager>,
    chat_message_list_pager: PagingManager<ChatMessage, ChatMessagePager>,
}

impl ChatServerService {
    pub fn new() -> Self {
        let shared = Shared::new();
        let (tx, mut rx) = mpsc::channel(32);

        shared.redis_client.chat_subscribe(tx);

        let shared = Arc::new(shared);

        let shared_clone = shared.clone();
        let shared_clone2 = shared.clone();
        let shared_clone3 = shared.clone();
        let shared_clone4 = shared.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let room_id = &(msg.room_id);
                shared_clone.incr_unread_message_count(room_id).await;
                shared_clone.broadcast(&msg).await;
            }
        });

        ChatServerService {
            shared,
            chat_room_list_pager: PagingManager::new(ChatRoomPager::new(shared_clone2)),
            chat_room_member_list_pager: PagingManager::new(ChatRoomUserPager::new(shared_clone3)),
            chat_message_list_pager: PagingManager::new(ChatMessagePager::new(shared_clone4)),
        }
    }

    pub fn list_chat_rooms(
        &self,
        user_id: String,
        request: ListChatRoomsRequest,
    ) -> Result<ListChatRoomsResponse, tonic::Status> {
        let page_token_id = request.page_token;
        let page_size = request.page_size;

        let paging_result =
            self.chat_room_list_pager
                .paging(user_id, page_token_id, u64::from(page_size));

        Ok(ListChatRoomsResponse {
            rooms: paging_result.list,
            total_size: paging_result.total_size,
            next_page_token: paging_result.next_page_token,
            prev_page_token: paging_result.prev_page_token,
        })
    }

    pub fn list_chat_room_users(
        &self,
        request: ListChatRoomUsersRequest,
    ) -> Result<ListChatRoomUsersResponse, tonic::Status> {
        let parent = request.parent;
        let room_id = self.get_room_id(&parent);

        let page_token_id = request.page_token;
        let page_size = request.page_size;

        let paging_result =
            self.chat_room_member_list_pager
                .paging(room_id, page_token_id, u64::from(page_size));

        Ok(ListChatRoomUsersResponse {
            users: paging_result.list,
            total_size: paging_result.total_size,
            next_page_token: paging_result.next_page_token,
            prev_page_token: paging_result.prev_page_token,
        })
    }

    pub fn list_chat_messages(
        &self,
        request: ListChatMessagesRequest,
    ) -> Result<ListChatMessagesResponse, tonic::Status> {
        let parent = request.parent;
        let room_id = self.get_room_id(&parent);

        let page_token_id = request.page_token;
        let page_size = request.page_size;

        let paging_result =
            self.chat_message_list_pager
                .paging(room_id, page_token_id, u64::from(page_size));

        Ok(ListChatMessagesResponse {
            messages: paging_result.list,
            total_size: paging_result.total_size,
            next_page_token: paging_result.next_page_token,
            prev_page_token: paging_result.prev_page_token,
        })
    }

    pub fn list_chat_messages_old(
        &self,
        request: ListChatMessagesRequest,
    ) -> Result<ListChatMessagesResponse, tonic::Status> {
        let parent = request.parent;
        let room_id = self.get_room_id(&parent);

        let messages = {
            let message_ids = self
                .shared
                .redis_client
                .get_latest_room_message_list_all(&room_id)
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

        let total_size = self
            .shared
            .redis_client
            .get_chat_room_message_count(&room_id)
            .unwrap();

        Ok(ListChatMessagesResponse {
            messages,
            total_size,
            next_page_token: None, // FIXME
            prev_page_token: None, // FIXME
        })
    }

    pub fn read_chat_message(
        &self,
        request: ReadChatMessageRequest,
    ) -> Result<ReadChatMessageResponse, tonic::Status> {
        todo!()
    }

    pub fn entry_chat_room(
        &self,
        user_id: &String,
        request: EntryChatRoomRequest,
    ) -> Result<EntryChatRoomResponse, tonic::Status> {
        let parent: String = request.parent;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        let now = chrono::Utc::now();
        let create_time = now.timestamp_millis();
        let timestamp = self.convert_date_time(&now);

        let message_id = Ulid::new().to_string();

        let message = ChatMessage {
            name: format!("rooms/{}/messages/{}", room_id, message_id),
            owner: user_id.clone(),
            room_id: room_id.clone(),
            message: format!("{} has entered.", user_id),
            message_type: MessageType::ChatRoomEntryUser as i32,
            create_time: Some(timestamp),
        };

        self.shared
            .redis_client
            .add_room_member(&room_id, user_id, &create_time)
            .unwrap();

        self.shared
            .redis_client
            .add_room(user_id, &room_id, &create_time)
            .unwrap();

        self.shared
            .redis_client
            .add_chat_room_message(&room_id, &message_id, &message)
            .unwrap();

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ChatMessage(message.clone())),
        };

        self.shared.send_message(&connect_response);

        Ok(EntryChatRoomResponse {
            result: Some(message),
        })
    }

    pub fn leave_chat_room(
        &self,
        user_id: &String,
        request: LeaveChatRoomRequest,
    ) -> Result<LeaveChatRoomResponse, tonic::Status> {
        let parent: String = request.parent;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        self.shared
            .redis_client
            .delete_room_member(&room_id, user_id)
            .unwrap();

        let now = chrono::Utc::now();
        let create_time = self.convert_date_time(&now);

        let message = ChatMessage {
            name: Ulid::new().to_string(),
            owner: user_id.clone(),
            room_id,
            message: format!("{} has left.", user_id),
            message_type: MessageType::ChatRoomLeaveUser as i32,
            create_time: Some(create_time),
        };

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ChatMessage(message.clone())),
        };

        self.shared.send_message(&connect_response);

        Ok(LeaveChatRoomResponse {
            result: Some(message),
        })
    }

    pub async fn conn(
        &self,
        user_id: &str,
        request: ConnectRequest,
    ) -> Result<ConnStream, tonic::Status> {
        let (stream_tx, stream_rx) = mpsc::channel(1); // Fn usage

        let (tx, mut rx) = mpsc::channel(1);
        {
            self.shared
                .senders
                .write()
                .await
                .insert(user_id.to_owned(), tx);
        }

        let shared_clone = self.shared.clone();
        let user_id = user_id.to_owned();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match stream_tx.send(Ok(msg)).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("[Remote] stream tx sending error. Remote {}", &user_id);
                        shared_clone.senders.write().await.remove(&user_id);
                    }
                }
            }
        });

        println!("connect complete!!!");

        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(
            stream_rx,
        )))
    }

    pub fn speech(
        &self,
        user_id: &str,
        request: SpeechRequest,
    ) -> Result<SpeechResponse, tonic::Status> {
        let parent: String = request.parent;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        let message = request.message;
        let create_time = Timestamp::from(SystemTime::now());

        let message_id = Ulid::new().to_string();

        let message = ChatMessage {
            name: format!("{}/messages/{}", parent, message_id),
            owner: user_id.to_owned(),
            room_id: room_id.clone(),
            message,
            message_type: MessageType::Message as i32,
            create_time: Some(create_time),
        };

        let is_exist = self
            .shared
            .redis_client
            .get_room_member_score(&room_id, &user_id.to_owned())
            .is_ok();

        if !is_exist {
            return Err(Status::permission_denied("no permission"));
        }

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ChatMessage(message.clone())),
        };

        self.shared.send_message(&connect_response);

        self.shared
            .redis_client
            .add_chat_room_message(&room_id, &message_id, &message)
            .unwrap();

        self.shared
            .redis_client
            .set_latest_message(&room_id, &message_id)
            .unwrap();

        Ok(SpeechResponse {
            result: Some(message),
        })
    }

    fn get_room_id(&self, parent: &str) -> String {
        let parent_slice: Vec<&str> = parent.split('/').collect();

        parent_slice[1].to_string()
    }

    fn convert_date_time(&self, date_time: &DateTime<Utc>) -> Timestamp {
        {
            let year = date_time.year() as i64;
            let month = date_time.month() as u8;
            let day = date_time.day() as u8;
            let hour = date_time.hour() as u8;
            let minute = date_time.minute() as u8;
            let second = date_time.second() as u8;
            let nanos = date_time.timestamp_subsec_nanos();

            Timestamp::date_time_nanos(year, month, day, hour, minute, second, nanos)
        }
        .unwrap()
    }
}
