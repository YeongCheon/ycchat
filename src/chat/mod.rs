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

#[derive(Debug)]
struct Shared {
    senders: HashMap<String, mpsc::Sender<ConnectResponse>>,
}

impl Shared {
    fn new() -> Self {
        Shared {
            senders: HashMap::new(),
        }
    }

    async fn broadcast(&self, msg: ConnectResponse) {
        for (name, tx) in &self.senders {
            match tx.send(msg.clone()).await {
                Ok(_) => {}
                Err(_) => {
                    println!("[Broadcast] SendError: to {}, {:?}", name, msg)
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
        let name = Ulid::new().to_string(); // FIXME

        let (stream_tx, stream_rx) = mpsc::channel(1); // Fn usage

        let (tx, mut rx) = mpsc::channel(1);
        {
            self.shared.write().await.senders.insert(name.clone(), tx);
        }

        let shared_clone = self.shared.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match stream_tx.send(Ok(msg)).await {
                    Ok(_) => {}
                    Err(_) => {
                        println!("[Remote] stream tx sending error. Remote {}", &name);
                        shared_clone.write().await.senders.remove(&name);
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
