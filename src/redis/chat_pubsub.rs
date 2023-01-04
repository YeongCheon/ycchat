use crate::chat::ycchat::ChatMessage;

use super::RedisClient;
use redis::Commands;
use tokio::sync::mpsc::Sender;

impl RedisClient {
    pub fn chat_subscribe(&self, tx: Sender<ChatMessage>) {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                let mut pubsub = con.as_pubsub();
                pubsub.subscribe(channel).unwrap();

                while let Ok(msg) = pubsub.get_message() {
                    let payload: ChatMessage = msg.get_payload().unwrap();
                    tx.blocking_send(payload).unwrap();
                }
            })
            .await
            .unwrap();
        });
    }

    pub fn chat_publish(&self, message: &ChatMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_connection().unwrap();

        let channel = self.generate_chat_pubsub_key();

        con.publish(channel, message)?;

        Ok(())
    }

    fn generate_chat_pubsub_key(&self) -> String {
        "ycchat:pubsub".to_string()
    }
}
