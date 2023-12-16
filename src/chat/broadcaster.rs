use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::services::ycchat::v1::services::connect::{
    server_signal::Payload, ChannelReceiveMessage,
};
use crate::{
    models::user::UserId,
    services::{
        ycchat::v1::models::Message,
        ycchat::v1::services::connect::{ConnectResponse, ServerSignal},
    },
};
use tokio::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender},
    RwLock,
};
use ulid::Ulid;

pub struct Broadcaster {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    streams: RwLock<HashMap<UserId, HashSet<Stream>>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let streams = RwLock::new(HashMap::new());
        let (sender, mut receiver) = channel::<Message>(10);

        Self {
            sender,
            receiver,
            streams,
        }
    }

    pub async fn send_msg(&self, user_ids: &[UserId], message: Message) {
        self.sender
            .send(message)
            .await
            .expect("Failed to send data");
    }

    pub async fn send_message(&self, user_ids: &[UserId], message: Message) {
        let streams = self.streams.read().await;

        while let Some(user_id) = user_ids.first() {
            let hash_set = streams.get(user_id);

            if let Some(hash_set) = hash_set {
                while let Some(stream) = hash_set.iter().next() {
                    let channel_receive_message =
                        Payload::ChannelReceiveMessage(ChannelReceiveMessage {
                            message: Some(message.clone()),
                        });

                    let server_signal = ServerSignal {
                        payload: Some(channel_receive_message),
                    };

                    let conn_response = ConnectResponse {
                        server_signal: Some(server_signal),
                    };
                    let _ = stream.sender.send(conn_response).await;
                }
            }
        }
    }

    pub async fn set_stream(&mut self, user_id: UserId, stream: Stream) {
        let mut streams = self.streams.write().await;
        let hash_set = streams.get_mut(&user_id);

        match hash_set {
            Some(hash_set) => {
                hash_set.insert(stream);
            }
            None => {
                let mut hash_set = HashSet::new();
                hash_set.insert(stream);

                streams.insert(user_id, hash_set);
            }
        };
    }

    pub async fn remove_stream(&self, user_id: UserId, stream: Stream) {
        let mut streams = self.streams.write().await;
        let hash_set = streams.get_mut(&user_id);

        if let Some(hash_set) = hash_set {
            hash_set.remove(&stream);
        };
    }
}

pub struct Stream {
    id: Ulid,
    sender: Sender<ConnectResponse>,
}

impl Stream {
    pub fn new(sender: Sender<ConnectResponse>) -> Self {
        Self {
            id: Ulid::new(),
            sender,
        }
    }
}

impl PartialEq for Stream {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Stream {}

impl Hash for Stream {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.id.to_bytes());
    }
}
