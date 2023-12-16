use prost::Message as _;
use surrealdb::{engine::remote::ws::Client, Surreal};
use tonic::{Request, Response, Status};

use crate::{
    db::{
        surreal::conn,
        traits::{
            channel::ChannelRepository, message::MessageRepository,
            message_acknowledge::MessageAcknowledgeRepository,
            server_member::ServerMemberRepository,
        },
    },
    models::{
        channel::{ChannelId, ChannelType},
        message::MessageId,
        message_acknowledge::DbMessageAcknowledge,
        user::UserId,
    },
    util::{self, pager::PageTokenizer},
};

use super::{
    ycchat::v1::models::Message,
    ycchat::v1::services::message::{
        message_service_server::MessageService as ProtoMessageService, AcknowledgeMessageRequest,
        DeleteMessageRequest, ListMessagesRequest, ListMessagesResponse, UpdateMessageRequest,
    },
};

pub struct MessageService<M, ACK, SM, CH>
where
    M: MessageRepository<Surreal<Client>>,
    ACK: MessageAcknowledgeRepository<Surreal<Client>>,
    SM: ServerMemberRepository<Surreal<Client>>,
    CH: ChannelRepository<Surreal<Client>>,
{
    channel_repository: CH,
    message_repository: M,
    server_member_repository: SM,
    message_acknowledge_repository: ACK,
}

impl<M, ACK, SM, CH> MessageService<M, ACK, SM, CH>
where
    M: MessageRepository<Surreal<Client>>,
    ACK: MessageAcknowledgeRepository<Surreal<Client>>,
    SM: ServerMemberRepository<Surreal<Client>>,
    CH: ChannelRepository<Surreal<Client>>,
{
    pub fn new(
        message_repository: M,
        message_acknowledge_repository: ACK,
        server_member_repository: SM,
        channel_repository: CH,
    ) -> Self {
        MessageService {
            message_repository,
            server_member_repository,
            channel_repository,
            message_acknowledge_repository,
        }
    }
}

#[tonic::async_trait]
impl<M, ACK, SM, CH> ProtoMessageService for MessageService<M, ACK, SM, CH>
where
    M: MessageRepository<Surreal<Client>> + 'static,
    ACK: MessageAcknowledgeRepository<Surreal<Client>> + 'static,
    SM: ServerMemberRepository<Surreal<Client>> + 'static,
    CH: ChannelRepository<Surreal<Client>> + 'static,
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

    async fn list_messages(
        &self,
        request: Request<ListMessagesRequest>,
    ) -> Result<Response<ListMessagesResponse>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let request = request.into_inner();
        let name = request.parent;
        let page_token = match request.page_token.clone() {
            Some(page_token) => {
                let page_token = util::pager::get_page_token(page_token);

                Some(page_token.unwrap())
            }
            None => None,
        };

        let (page_size, offset_id, prev_page_token) = match page_token {
            Some(page_token) => (
                page_token.page_size,
                page_token
                    .offset_id
                    .map(|offset_id| MessageId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        let channel_id = ChannelId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();
        let channel = match self.channel_repository.get(&db, &channel_id).await.unwrap() {
            Some(channel) => channel,
            None => return Err(Status::not_found("invalid arguments.")),
        };

        let is_have_permission: bool = match channel.channel_type {
            ChannelType::Saved { owner } => owner == user_id,
            ChannelType::Direct => todo!(),
            ChannelType::Server { server } => {
                let server_member = self
                    .server_member_repository
                    .get_server_member_by_server_id_and_user_id(&db, &server, &user_id)
                    .await
                    .unwrap();

                server_member.is_some()
            }
        };

        if !is_have_permission {
            return Err(Status::permission_denied("permission denied."));
        }

        let message_list = self
            .message_repository
            .get_list_by_chnanel_id(&db, &channel_id, page_size + 1, offset_id)
            .await;

        let mut message_list = match message_list {
            Ok(message_list) => message_list,
            Err(err) => return Err(Status::internal("message list internal server")),
        };

        let next_page_token = if message_list.len() > usize::try_from(page_size).unwrap() {
            message_list.pop();

            let next_page_token = message_list.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                crate::util::base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        let list_message_response = ListMessagesResponse {
            messages: message_list
                .into_iter()
                .map(|message| message.to_message())
                .collect(),
            next_page_token,
            prev_page_token,
        };

        Ok(Response::new(list_message_response))
    }
}
