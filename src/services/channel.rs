use std::str::FromStr;

use tonic::{Request, Response, Status};

use crate::db::traits::channel::ChannelRepository;
use crate::db::traits::message::MessageRepository;
use crate::db::traits::server::ServerRepository;
use crate::db::traits::server_category::ServerCategoryRepository;
use crate::models::channel::{ChannelId, DbChannel};
use crate::models::message::DbMessage;
use crate::models::server::ServerId;
use crate::models::server_category::{DbServerCategory, ServerCategoryId};
use crate::models::user::UserId;
use crate::redis::RedisClient;

use super::model::Channel as ChannelModel;
use super::ycchat_channel::channel_server::Channel;
use super::ycchat_channel::{
    CreateChannelRequest, DeleteChannelRequest, ListChannelMembersRequest,
    ListChannelMembersResponse, ListChannelMessagesRequest, ListChannelMessagesResponse,
    ListServerChannelsRequest, ListServerChannelsResponse, SpeechRequest, SpeechResponse,
    UpdateChannelRequest,
};

pub struct ChannelService<M, C, S, SC>
where
    M: MessageRepository,
    C: ChannelRepository,
    S: ServerRepository,
    SC: ServerCategoryRepository,
{
    message_repository: M,
    channel_repository: C,
    server_repository: S,
    server_category_repository: SC,
    redis_client: RedisClient,
}

impl<M, C, S, SC> ChannelService<M, C, S, SC>
where
    M: MessageRepository,
    C: ChannelRepository,
    S: ServerRepository,
    SC: ServerCategoryRepository,
{
    pub fn new(
        message_repository: M,
        channel_repository: C,
        server_repository: S,
        server_category_repository: SC,
    ) -> Self {
        let redis_client = RedisClient::new();

        ChannelService {
            message_repository,
            channel_repository,
            server_repository,
            server_category_repository,
            redis_client,
        }
    }
}

#[tonic::async_trait]
impl<M, C, S, SC> Channel for ChannelService<M, C, S, SC>
where
    M: MessageRepository + 'static,
    C: ChannelRepository + 'static,
    S: ServerRepository + 'static,
    SC: ServerCategoryRepository + 'static,
{
    async fn list_server_channels(
        &self,
        request: Request<ListServerChannelsRequest>,
    ) -> Result<Response<ListServerChannelsResponse>, Status> {
        let parent = request.into_inner().parent;

        let parent = parent.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(parent[1]).unwrap();

        let channels = self
            .channel_repository
            .get_server_channels(&server_id)
            .await
            .unwrap()
            .into_iter()
            .map(|channel| channel.to_message())
            .collect::<Vec<ChannelModel>>();

        Ok(Response::new(ListServerChannelsResponse {
            channels,
            page: None,
        }))
    }

    async fn create_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<ChannelModel>, Status> {
        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let channel = match request.into_inner().channel {
            Some(channel) => channel,
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let name_list: Vec<&str> = channel.name.split('/').collect::<Vec<&str>>();
        let server_index = name_list
            .iter()
            .position(|&s| s == "servers")
            .map(|idx| idx + 1);
        let category_index = name_list
            .iter()
            .position(|&s| s == "categories")
            .map(|idx| idx + 1);

        let server = match server_index {
            Some(idx) => {
                let server_id: ServerId = ServerId::from_str(name_list[idx]).unwrap();

                let server = self.server_repository.get_server(&server_id).await;

                match server {
                    Ok(server) => Some(server),
                    Err(err) => return Err(Status::not_found(err.as_str())),
                }
            }
            None => None,
        };

        let category: Option<DbServerCategory> = match category_index {
            Some(idx) => {
                let server_category_id: ServerCategoryId =
                    ServerCategoryId::from_str(name_list[idx]).unwrap();

                let category = self
                    .server_category_repository
                    .get(&server_category_id)
                    .await;

                match category {
                    Ok(category) => match category {
                        Some(category) => Some(category),
                        None => return Err(Status::not_found("server category not found.")),
                    },
                    Err(err) => return Err(Status::not_found(err.as_str())),
                }
            }
            None => None,
        };

        let channel = DbChannel::new(user_id, channel, server.map(|server| server.id));

        let added = self.channel_repository.add(&channel).await.unwrap();

        Ok(Response::new(added.to_message()))
    }

    async fn list_channel_members(
        &self,
        request: Request<ListChannelMembersRequest>,
    ) -> Result<Response<ListChannelMembersResponse>, Status> {
        todo!()
    }

    async fn update_channel(
        &self,
        request: Request<UpdateChannelRequest>,
    ) -> Result<Response<ChannelModel>, Status> {
        let req = request.into_inner();
        let channel = req.channel.unwrap();

        let name = &channel.name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();

        let channel_index = name
            .iter()
            .position(|&s| s == "channels")
            .map(|idx| idx + 1);

        let mut exist = match channel_index {
            Some(idx) => {
                let channel_id = ChannelId::from_string(name[idx]).unwrap();

                match self.channel_repository.get(&channel_id).await {
                    Ok(channel) => match channel {
                        Some(channel) => channel,
                        None => return Err(Status::not_found("channel not found.")),
                    },
                    Err(err) => return Err(Status::internal(err.to_string())),
                }
            }
            None => return Err(Status::invalid_argument("invalid arguments.")),
        };

        exist.display_name = channel.display_name;
        exist.description = channel.description;

        let res = self.channel_repository.update(&exist).await.unwrap();

        Ok(Response::new(res.to_message()))
    }

    async fn delete_channel(
        &self,
        request: Request<DeleteChannelRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().name;
        let name = name.split('/').collect::<Vec<&str>>();

        let channel_index = name
            .iter()
            .position(|&s| s == "channels")
            .map(|idx| idx + 1);

        let channel_id: ChannelId = match channel_index {
            Some(idx) => ChannelId::from_string(name[idx]).unwrap(),
            None => return Err(Status::invalid_argument("invalid arguments.")),
        };

        self.channel_repository.delete(&channel_id).await.unwrap();

        Ok(Response::new(()))
    }

    async fn list_channel_messages(
        &self,
        request: Request<ListChannelMessagesRequest>,
    ) -> Result<Response<ListChannelMessagesResponse>, Status> {
        todo!()
    }

    async fn speech(
        &self,
        request: Request<SpeechRequest>,
    ) -> Result<Response<SpeechResponse>, Status> {
        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let req = request.into_inner();
        let name = req.name;
        let content = req.content;

        let channel_id = ChannelId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let message = DbMessage::new(user_id, channel_id, content);

        let message = self.message_repository.add(&message).await.unwrap();
        let message = message.to_message();

        self.redis_client.chat_publish(&message).unwrap();

        Ok(Response::new(SpeechResponse {
            result: Some(message),
        }))
    }
}
