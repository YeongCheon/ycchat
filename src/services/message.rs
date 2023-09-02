use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::{Request, Response, Status};

use crate::{
    db::{
        surreal::conn,
        traits::{message::MessageRepository, message_acknowledge::MessageAcknowledgeRepository},
    },
    models::{message::MessageId, message_acknowledge::DbMessageAcknowledge, user::UserId},
};

use super::{
    model::Message,
    ycchat_message::{
        message_service_server::MessageService as ProtoMessageService, AcknowledgeMessageRequest,
        DeleteMessageRequest, UpdateMessageRequest,
    },
};

pub struct MessageService<M, ACK>
where
    M: MessageRepository<Surreal<Client>>,
    ACK: MessageAcknowledgeRepository<Surreal<Client>>,
{
    message_repository: M,
    message_acknowledge_repository: ACK,
}

impl<M, ACK> MessageService<M, ACK>
where
    M: MessageRepository<Surreal<Client>>,
    ACK: MessageAcknowledgeRepository<Surreal<Client>>,
{
    pub fn new(message_repository: M, message_acknowledge_repository: ACK) -> Self {
        MessageService {
            message_repository,
            message_acknowledge_repository,
        }
    }
}

#[tonic::async_trait]
impl<M, ACK> ProtoMessageService for MessageService<M, ACK>
where
    M: MessageRepository<Surreal<Client>> + 'static,
    ACK: MessageAcknowledgeRepository<Surreal<Client>> + 'static,
{
    async fn acknowledge_message(
        &self,
        request: Request<AcknowledgeMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let req = request.into_inner();
        let name = req.name; // servers/{serverId}/members/{serverMemberId}
        let name = name.split('/').collect::<Vec<&str>>();

        let message_id = MessageId::from_string(name[1]).unwrap();

        let exist = self
            .message_acknowledge_repository
            .get_by_message_and_user(&db, &message_id, &user_id)
            .await
            .unwrap();

        if exist.is_some() {
            return Err(Status::already_exists("already exist."));
        }

        self.message_acknowledge_repository
            .add(&db, &DbMessageAcknowledge::new(message_id, user_id))
            .await
            .unwrap();

        Ok(Response::new(()))
    }

    async fn update_message(
        &self,
        request: Request<UpdateMessageRequest>,
    ) -> Result<Response<Message>, Status> {
        todo!()
    }

    async fn delete_message(
        &self,
        request: Request<DeleteMessageRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let req = request.into_inner();
        let name = req.name; // servers/{serverId}/members/{serverMemberId}
        let name = name.split('/').collect::<Vec<&str>>();

        let message_id = MessageId::from_string(name[1]).unwrap();

        let exist = self.message_repository.get(&db, &message_id).await.unwrap();

        match exist {
            Some(exist) => {
                if exist.author != user_id {
                    return Err(Status::permission_denied("no permission"));
                }
            }
            None => return Err(Status::not_found("message not found.")),
        }

        self.message_repository
            .delete(&db, &message_id)
            .await
            .unwrap();

        Ok(Response::new(()))
    }
}
