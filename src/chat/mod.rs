use std::{collections::HashMap, pin::Pin, sync::Arc, time::SystemTime};

use prost_types::Timestamp;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{codegen::InterceptedService, Request, Response, Status};
use ulid::Ulid;
use ycchat::chat_service_server::ChatService;

use self::ycchat::{
    chat_service_server::ChatServiceServer, connect_response::Payload, ConnectResponse,
    ReceiveMessageResponse, SpeechResponse,
};

mod interceptor;
pub mod ycchat {
    tonic::include_proto!("ycchat");
}

use crate::redis::{self as yc_redis, RedisClient};

type UserId = String;
type RoomId = String;

const METADATA_AUTH_KEY: &str = "authorization";

#[derive(Debug)]
struct Shared {
    redis_client: RedisClient,
    senders: HashMap<UserId, mpsc::Sender<ConnectResponse>>,
}

impl Shared {
    fn new() -> Self {
        let redis = yc_redis::RedisClient::new();

        let (tx, _) = mpsc::channel(1);

        redis.chat_subscribe(tx);

        Shared {
            redis_client: redis,
            senders: HashMap::new(),
        }
    }

    fn send_message(&self, msg: &ConnectResponse) {
        let message: Option<(RoomId, String)> = if let Some(ref payload) = msg.payload {
            match payload {
                Payload::ConnectSuccess(_) => None,
                Payload::ReceiveMessage(item) => Some((item.room_id.clone(), item.message.clone())),
            }
        } else {
            None
        };

        if let Some((room_id, message)) = message {
            self.redis_client.chat_publish(&room_id, &message).unwrap();
        }
    }

    async fn broadcast(&self, msg: ConnectResponse) {
        let room_id: Option<String> = if let Some(ref payload) = msg.payload {
            match payload {
                Payload::ConnectSuccess(_) => None,
                Payload::ReceiveMessage(item) => Some(item.room_id.clone()),
            }
        } else {
            None
        };

        if let Some(room_id) = room_id {
            let room_members = self.redis_client.get_room_members(&room_id).unwrap();

            println!("{:?}", room_members);
            for (user_id, tx) in &self.senders {
                println!("{}", user_id);
                if !room_members.contains(user_id) {
                    continue;
                }
                match tx.send(msg.clone()).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("[Broadcast] SendError: to {}, {:?}", user_id, msg)
                    }
                }
            }
        }
    }
}

pub struct MyChatService {
    shared: Arc<RwLock<Shared>>,
}

impl MyChatService {
    pub fn new() -> Self {
        let shared = Arc::new(RwLock::new(Shared::new()));
        MyChatService { shared }
    }
}

#[tonic::async_trait]
impl ChatService for MyChatService {
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
        let room_id = request.into_inner().room_id;

        self.shared
            .write()
            .await
            .redis_client
            .add_room_member(&room_id, user_id)
            .unwrap();

        Ok(Response::new(ycchat::EntryChatRoomResponse {
            message: "success".to_string(),
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
        let room_id = request.into_inner().room_id;

        self.shared
            .write()
            .await
            .redis_client
            .delete_room_member(&room_id, user_id)
            .unwrap();

        Ok(Response::new(ycchat::LeaveChatRoomResponse {
            message: "success".to_string(),
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
                .write()
                .await
                .senders
                .insert(user_id.clone(), tx);
        }

        let shared_clone = self.shared.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match stream_tx.send(Ok(msg)).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("[Remote] stream tx sending error. Remote {}", &user_id);
                        shared_clone.write().await.senders.remove(&user_id);
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

        let room_id = speech_request.room_id;
        let message = speech_request.message;
        let created_at = Timestamp::from(SystemTime::now());

        let message = ReceiveMessageResponse {
            id: Ulid::new().to_string(),
            owner: user_id,
            room_id,
            message,
            created_at: Some(created_at),
        };

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ReceiveMessage(message.clone())),
        };

        self.shared.read().await.send_message(&connect_response);

        self.shared.read().await.broadcast(connect_response).await;

        Ok(Response::new(SpeechResponse {
            result: Some(message),
        }))
    }
}

type Interceptor = fn(Request<()>) -> Result<Request<()>, Status>;

pub fn get_chat_service_server() -> InterceptedService<ChatServiceServer<MyChatService>, Interceptor>
{
    let chat_service = MyChatService::new();
    ChatServiceServer::with_interceptor(chat_service, interceptor::auth::check_auth)
}
