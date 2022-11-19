use std::{collections::HashMap, pin::Pin, sync::Arc, time::SystemTime};

use prost::encoding::message;
use prost_types::Timestamp;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{codegen::InterceptedService, Request, Response, Status};
use ulid::Ulid;
use ycchat::chat_service_server::ChatService;

use self::ycchat::{
    chat_message::MessageType, chat_service_server::ChatServiceServer, connect_response::Payload,
    ChatMessage, ChatRoom, ChatUser, ConnectResponse, ListChatRoomUsersRequest,
    ListChatRoomUsersResponse, ListChatRoomsRequest, ListChatRoomsResponse, SpeechResponse,
};

mod interceptor;
pub mod ycchat {
    tonic::include_proto!("ycchat");
}

use crate::redis::{self as yc_redis, RedisClient};

type UserId = String;

const METADATA_AUTH_KEY: &str = "authorization";

#[derive(Debug)]
struct Shared {
    redis_client: RedisClient,
    senders: RwLock<HashMap<UserId, mpsc::Sender<ConnectResponse>>>,
}

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
        let parent_slice: Vec<&str> = parent.split("/").collect();
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

pub struct MyChatService {
    shared: Arc<Shared>,
}

impl MyChatService {
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

        MyChatService { shared }
    }
}

#[tonic::async_trait]
impl ChatService for MyChatService {
    async fn list_chat_rooms(
        &self,
        request: tonic::Request<ListChatRoomsRequest>,
    ) -> Result<tonic::Response<ListChatRoomsResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let room_ids = self.shared.redis_client.get_rooms(&user_id).unwrap();

        let rooms = room_ids
            .iter()
            .map(|room_id| ChatRoom {
                name: format!("rooms/{}", room_id),
            })
            .collect();

        Ok(Response::new(ListChatRoomsResponse {
            rooms,
            total_size: 0,         // FIXME
            next_page_token: None, // FIXME
            prev_page_token: None, // FIXME
        }))
    }

    async fn list_chat_room_users(
        &self,
        request: tonic::Request<ListChatRoomUsersRequest>,
    ) -> Result<tonic::Response<ListChatRoomUsersResponse>, tonic::Status> {
        let parent = request.into_inner().parent;
        let parent_slice: Vec<&str> = parent.split("/").collect();
        let room_id = parent_slice[1].to_string();

        let room_members = self.shared.redis_client.get_room_members(&room_id).unwrap();

        let users = room_members
            .iter()
            .map(|room_member| ChatUser {
                name: format!("users/{}", room_member),
            })
            .collect();

        Ok(Response::new(ListChatRoomUsersResponse {
            users,
            total_size: 0,
            next_page_token: None,
            prev_page_token: None,
        }))
    }

    async fn entry_chat_room(
        &self,
        request: tonic::Request<ycchat::EntryChatRoomRequest>,
    ) -> Result<tonic::Response<ycchat::EntryChatRoomResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let parent: String = request.into_inner().parent;
        let parent_slice: Vec<&str> = parent.split("/").collect();
        let room_id = parent_slice[1].to_string();

        self.shared
            .redis_client
            .add_room_member(&room_id, &user_id)
            .unwrap();

        let message = ChatMessage {
            name: format!("rooms/{}/messages/{}", room_id, Ulid::new().to_string()),
            owner: user_id,
            room_id,
            message: "success".to_string(),
            message_type: MessageType::Message as i32,
            create_time: Some(Timestamp::from(SystemTime::now())),
        };

        Ok(Response::new(ycchat::EntryChatRoomResponse {
            result: Some(message),
        }))
    }

    async fn leave_chat_room(
        &self,
        request: tonic::Request<ycchat::LeaveChatRoomRequest>,
    ) -> Result<tonic::Response<ycchat::LeaveChatRoomResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let parent: String = request.into_inner().parent;
        let parent_slice: Vec<&str> = parent.split("/").collect();
        let room_id = parent_slice[1].to_string();

        self.shared
            .redis_client
            .delete_room_member(&room_id, &user_id)
            .unwrap();

        let message = ycchat::ChatMessage {
            name: Ulid::new().to_string(),
            owner: user_id,
            room_id,
            message: "success".to_string(),
            message_type: MessageType::Message as i32,
            create_time: Some(Timestamp::from(SystemTime::now())),
        };

        Ok(Response::new(ycchat::LeaveChatRoomResponse {
            result: Some(message),
        }))
    }

    type ConnStream =
        Pin<Box<dyn Stream<Item = Result<ConnectResponse, Status>> + Send + Sync + 'static>>;

    async fn conn(
        &self,
        request: tonic::Request<ycchat::ConnectRequest>,
    ) -> Result<tonic::Response<Self::ConnStream>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let (stream_tx, stream_rx) = mpsc::channel(1); // Fn usage

        let (tx, mut rx) = mpsc::channel(1);
        {
            self.shared
                .senders
                .write()
                .await
                .insert(user_id.clone(), tx);
        }

        let shared_clone = self.shared.clone();
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

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(stream_rx),
        )))
    }

    async fn speech(
        &self,
        request: tonic::Request<ycchat::SpeechRequest>,
    ) -> Result<tonic::Response<ycchat::SpeechResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let speech_request = request.into_inner();

        let parent: String = speech_request.parent;
        let parent_slice: Vec<&str> = parent.split("/").collect();
        let room_id = parent_slice[1].to_string();

        let message = speech_request.message;
        let create_time = Timestamp::from(SystemTime::now());

        let message = ChatMessage {
            name: format!("{}/messages/{}", parent, Ulid::new().to_string()),
            owner: user_id,
            room_id,
            message,
            message_type: MessageType::Message as i32,
            create_time: Some(create_time),
        };

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ChatMessage(message.clone())),
        };

        self.shared.send_message(&connect_response);

        Ok(Response::new(SpeechResponse {
            result: Some(message),
        }))
    }

    async fn read_chat_message(
        &self,
        request: tonic::Request<ycchat::ReadChatMessageRequest>,
    ) -> Result<tonic::Response<ycchat::ReadChatMessageResponse>, tonic::Status> {
        todo!("")
    }

    async fn list_chat_messages(
        &self,
        request: tonic::Request<ycchat::ListChatMessagesRequest>,
    ) -> Result<tonic::Response<ycchat::ListChatMessagesResponse>, tonic::Status> {
        todo!("")
    }
}

type Interceptor = fn(Request<()>) -> Result<Request<()>, Status>;

pub fn get_chat_service_server() -> InterceptedService<ChatServiceServer<MyChatService>, Interceptor>
{
    let chat_service = MyChatService::new();
    ChatServiceServer::with_interceptor(chat_service, interceptor::auth::check_auth)
}
