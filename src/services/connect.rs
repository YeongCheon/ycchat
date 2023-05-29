use crate::db::traits::channel::ChannelRepository;
use crate::db::traits::server::ServerRepository;
use crate::db::traits::server_member::ServerMemberRepository;
use crate::models::channel::{ChannelId, DbChannel};
use crate::models::server::ServerId;
use crate::models::server_member::DbServerMember;
use crate::models::user::UserId;
use crate::redis::{self as yc_redis, RedisClient};
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::Stream;
use tonic::{Response, Result, Status};

use super::model::Message;
use super::ycchat_connect::server_signal::Payload;
use super::ycchat_connect::{self, connect_server::Connect, ConnectResponse};
use super::ycchat_connect::{ChannelReceiveMessage, Ping, ServerSignal};

#[derive(Debug)]
pub struct Shared<C, S, U>
where
    C: ChannelRepository,
    S: ServerRepository,
    U: ServerMemberRepository,
{
    pub redis_client: RedisClient,
    pub senders: RwLock<HashMap<UserId, mpsc::Sender<ConnectResponse>>>,
    pub channel_repository: C,
    pub server_repository: S,
    pub server_member_repository: U,
}

impl<C, S, U> Shared<C, S, U>
where
    C: ChannelRepository,
    S: ServerRepository,
    U: ServerMemberRepository,
{
    fn new(channel_repository: C, server_repository: S, server_member_repository: U) -> Self {
        let redis = yc_redis::RedisClient::new();

        let senders = RwLock::new(HashMap::new());

        Shared {
            redis_client: redis,
            senders,
            channel_repository,
            server_repository,
            server_member_repository,
        }
    }

    fn send_message(&self, msg: &ConnectResponse) {
        let message: Option<Message> = if let Some(ref server_signal) = msg.server_signal {
            if let Some(ref payload) = server_signal.payload {
                let payload = payload.clone();

                match payload {
                    Payload::Ping(_) => None,
                    Payload::ServerEntryUser(_) => todo!(),
                    Payload::ServerLeaveUser(_) => todo!(),
                    Payload::ChannelReceiveMessage(item) => item.message,
                    // Payload::ChatMessage(item) => Some(item),
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(chat_message) = message {
            // let room_id = chat_message.room_id.clone();
            // let message_id = chat_message.name.clone();

            // self.redis_client.chat_publish(chat_message).unwrap();
        }
    }

    async fn broadcast(&self, msg: &Message) {
        let name = &msg.name;
        // let parent_slice: Vec<&str> = parent.split('/').collect();
        // let channel_id = ChannelId::from_string(parent_slice[1]);
        let channel_id = ChannelId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();
        // let channel_id = ChannelId::new(); // FIXME

        let channel: DbChannel = self
            .channel_repository
            .get(&channel_id)
            .await
            .unwrap()
            .unwrap();

        let server_id: ServerId = match channel.channel_type {
            crate::models::channel::ChannelType::Saved { owner } => todo!(),
            crate::models::channel::ChannelType::Direct => todo!(),
            crate::models::channel::ChannelType::Server { server } => server,
        };

        let server_members: Vec<DbServerMember> = self
            .server_member_repository
            .get_server_members_by_server_id(&server_id)
            .await
            .unwrap();

        let read_guard = self.senders.read().await;

        let users = read_guard.clone(); // FIXME

        for (user_id, tx) in &users {
            let is_contain = server_members
                .iter()
                .any(|server_member| server_member.server == server_id);

            if !is_contain {
                continue;
            }

            let conn_response = ConnectResponse {
                server_signal: Some(ServerSignal {
                    payload: Some(Payload::ChannelReceiveMessage(ChannelReceiveMessage {
                        message: Some(msg.clone()),
                    })),
                }),
            };

            // let conn_response = ConnectResponse {
            //     server_signal: Some(ServerSignal {
            //         payload: Some(Payload::ChannelReceiveMessage(&msg.clone())),
            //     }),
            // };

            match tx.send(conn_response).await {
                Ok(_) => {}
                Err(_) => {
                    println!("[Broadcast] SendError: to {}, {:?}", user_id, msg)
                }
            }
        }
    }

    pub async fn incr_unread_message_count(&self, channel_id: &ChannelId) {
        let members = self.redis_client.get_room_members_all(channel_id).unwrap();

        members.iter().for_each(|user_id| {
            self.redis_client.incr(user_id, channel_id).unwrap();
        });
    }
}

pub struct ConnectService<C, S, U>
where
    C: ChannelRepository,
    S: ServerRepository,
    U: ServerMemberRepository,
{
    shared: Arc<Shared<C, S, U>>,
}

impl<C, S, U> ConnectService<C, S, U>
where
    C: ChannelRepository + 'static,
    S: ServerRepository + 'static,
    U: ServerMemberRepository + 'static,
{
    pub fn new(channel_repository: C, server_repository: S, server_member_repository: U) -> Self {
        let shared = Shared::new(
            channel_repository,
            server_repository,
            server_member_repository,
        );
        let (tx, mut rx) = mpsc::channel(32);

        shared.redis_client.chat_subscribe(tx);

        let shared = Arc::new(shared);

        let shared_clone = shared.clone();
        let shared_clone2 = shared.clone();
        let shared_clone3 = shared.clone();
        let shared_clone4 = shared.clone();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // let room_id = &(msg.room_id);
                let channel_id: ChannelId = ChannelId::new(); // FIXME
                shared_clone.incr_unread_message_count(&channel_id).await;
                shared_clone.broadcast(&msg).await;
            }
        });

        ConnectService { shared }
    }
}

#[tonic::async_trait]
impl<C, S, U> Connect for ConnectService<C, S, U>
where
    C: ChannelRepository + 'static,
    S: ServerRepository + 'static,
    U: ServerMemberRepository + 'static,
{
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

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(stream_rx),
        )))
    }
}
