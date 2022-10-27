use std::{collections::HashMap, pin::Pin, sync::Arc};

use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{Response, Status};
use ulid::Ulid;
use ycchat::chat_service_server::ChatService;

use self::ycchat::{
    chat_service_server::ChatServiceServer, connect_response::Payload, ConnectResponse,
    ReceiveMessageResponse, SpeechResponse,
};

pub mod ycchat {
    tonic::include_proto!("ycchat");
}

type UserId = String;
type RoomId = String;

#[derive(Debug)]
struct Shared {
    senders: HashMap<UserId, mpsc::Sender<ConnectResponse>>,
    room_members: HashMap<RoomId, Vec<UserId>>,
}

impl Shared {
    fn new() -> Self {
        let mut room_members = HashMap::new();

        let room_id = "111".to_string(); // temp
        let user_id = "tempUserId".to_string(); // temp
        room_members.insert(room_id, vec![user_id]);

        Shared {
            senders: HashMap::new(),
            room_members,
        }
    }

    async fn broadcast(&self, msg: ConnectResponse) {
        let room_id: Option<String> = if let Some(ref payload) = msg.payload {
            match payload {
                Payload::ConnectSuccess(item) => None,
                Payload::ReceiveMessage(item) => Some(item.room_id.clone()),
            }
        } else {
            None
        };

        if let Some(room_id) = room_id {
            let room_members = if let Some(members) = self.room_members.get(&room_id) {
                members.to_owned()
            } else {
                Vec::new()
            };

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
    type ConnStream =
        Pin<Box<dyn Stream<Item = Result<ConnectResponse, Status>> + Send + Sync + 'static>>;

    async fn conn(
        &self,
        request: tonic::Request<ycchat::ConnectRequest>,
    ) -> Result<tonic::Response<Self::ConnStream>, tonic::Status> {
        let user_id = request.into_inner().user_id;
        // let user_id = request.into().user_id;

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
        let speech_request = request.into_inner();

        let owner = "FIXME".to_string();
        let room_id = speech_request.room_id;
        let message = speech_request.message;

        let message = ReceiveMessageResponse {
            id: Ulid::new().to_string(),
            owner,
            room_id,
            message,
        };

        let connect_response = ConnectResponse {
            id: Ulid::new().to_string(),
            payload: Some(Payload::ReceiveMessage(message.clone())),
        };

        self.shared.read().await.broadcast(connect_response).await;

        Ok(Response::new(SpeechResponse {
            result: Some(message),
        }))
    }
}

pub fn get_chat_service_server() -> ChatServiceServer<MyChatService> {
    let chat_service = MyChatService::new();
    let result: ChatServiceServer<MyChatService> = ChatServiceServer::new(chat_service);

    result
}
