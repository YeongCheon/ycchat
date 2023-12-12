use super::ycchat_connect::{self, connect_server::Connect, ConnectResponse};
use crate::chat::broadcaster::{Broadcaster, Stream as BroadcastStream};
use crate::db::traits::channel::ChannelRepository;
use crate::models::user::UserId;
use futures::lock::Mutex;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tonic::{Response, Result, Status};

pub struct ConnectService {
    broadcaster: Arc<Mutex<Broadcaster>>,
}

impl ConnectService {
    pub fn new(broadcaster: Arc<Mutex<Broadcaster>>) -> Self {
        ConnectService { broadcaster }
    }
}

#[tonic::async_trait]
impl Connect for ConnectService {
    type ConnStream =
        Pin<Box<dyn Stream<Item = Result<ConnectResponse, Status>> + Send + Sync + 'static>>;

    async fn conn(
        &self,
        request: tonic::Request<ycchat_connect::ConnectRequest>,
    ) -> Result<tonic::Response<Self::ConnStream>, tonic::Status> {
        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let (stream_tx, stream_rx) = mpsc::channel(1); // Fn usage

        let (tx, mut rx) = mpsc::channel(1);
        {
            let stream = BroadcastStream::new(tx);
            self.broadcaster.lock().await.set_stream(user_id, stream);
        }

        let user_id = user_id.to_owned();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match stream_tx.send(Ok(msg)).await {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("[Remote] stream tx sending error. Remote {}", &user_id);
                    }
                }
            }
        });

        println!("connect complete!!!");

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(stream_rx),
        )))
    }
}
