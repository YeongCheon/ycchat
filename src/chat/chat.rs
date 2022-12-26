use std::{collections::HashMap, pin::Pin, sync::Arc, time::SystemTime};

use chrono::{DateTime, Datelike, Timelike, Utc};
use prost_types::Timestamp;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{codegen::http::StatusCode, Result, Status};
use ulid::Ulid;

use crate::redis::{self as yc_redis, RedisClient};

use super::ycchat::{
    chat_message::MessageType, chat_room::ChatRoomType, connect_response::Payload, ChatMessage,
    ChatRoom, ChatUser, ConnectRequest, ConnectResponse, EntryChatRoomRequest,
    EntryChatRoomResponse, LeaveChatRoomRequest, LeaveChatRoomResponse, ListChatMessagesRequest,
    ListChatMessagesResponse, ListChatRoomUsersRequest, ListChatRoomUsersResponse,
    ListChatRoomsRequest, ListChatRoomsResponse, ReadChatMessageRequest, ReadChatMessageResponse,
    SpeechRequest, SpeechResponse,
};

pub type UserId = String;

#[derive(Debug)]
struct Shared {
    redis_client: RedisClient,
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
            self.redis_client.chat_publish(chat_message).unwrap();
        }
    }

    async fn broadcast(&self, msg: &ChatMessage) {
        let parent = &msg.name;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        let room_members = self.redis_client.get_room_members(&room_id).unwrap();

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
}

pub struct ChatServerService {
    shared: Arc<Shared>,
}

impl ChatServerService {
    pub fn new() -> Self {
        let shared = Shared::new();
        let (tx, mut rx) = mpsc::channel(32);

        shared.redis_client.chat_subscribe(tx);

        let shared = Arc::new(shared);

        let shared_clone = shared.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                shared_clone.broadcast(&msg).await;
            }
        });

        ChatServerService { shared }
    }

    pub fn list_chat_rooms(
        &self,
        user_id: String,
        request: ListChatRoomsRequest,
    ) -> Result<ListChatRoomsResponse, tonic::Status> {
        let room_ids = self.shared.redis_client.get_rooms(&user_id).unwrap();
        let total_size = self.shared.redis_client.get_rooms_count(&user_id).unwrap();

        let rooms = room_ids
            .iter()
            .map(|room_id| ChatRoom {
                name: format!("rooms/{}", room_id),
                chat_room_type: ChatRoomType::Public as i32,
            })
            .collect();

        Ok(ListChatRoomsResponse {
            rooms,
            total_size,
            next_page_token: None, // FIXME
            prev_page_token: None, // FIXME
        })
    }

    pub fn list_chat_room_users(
        &self,
        request: ListChatRoomUsersRequest,
    ) -> Result<ListChatRoomUsersResponse, tonic::Status> {
        let parent = request.parent;
        let parent_slice: Vec<&str> = parent.split('/').collect();
        let room_id = parent_slice[1].to_string();

        let room_members = self.shared.redis_client.get_room_members(&room_id).unwrap();
        let total_size = self
            .shared
            .redis_client
            .get_room_members_count(&room_id)
            .unwrap();

        let users = room_members
            .iter()
            .map(|room_member| ChatUser {
                name: format!("users/{}", room_member),
            })
            .collect();

        Ok(ListChatRoomUsersResponse {
            users,
            total_size,
            next_page_token: None,
            prev_page_token: None,
        })
    }

    pub fn list_chat_messages(
        &self,
        request: ListChatMessagesRequest,
    ) -> Result<ListChatMessagesResponse, tonic::Status> {
        todo!()
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

        self.shared
            .redis_client
            .add_room_member(&room_id, user_id, &create_time)
            .unwrap();

        let message = ChatMessage {
            name: format!("rooms/{}/messages/{}", room_id, Ulid::new().to_string()),
            owner: user_id.clone(),
            room_id,
            message: format!("{} has entered.", user_id),
            message_type: MessageType::ChatRoomEntryUser as i32,
            create_time: Some(timestamp),
        };

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
            .set_latest_message(&room_id, &message_id)
            .unwrap();

        Ok(SpeechResponse {
            result: Some(message),
        })
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