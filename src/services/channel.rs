use std::str::FromStr;

use prost::Message as _;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::channel::ChannelRepository;
use crate::db::traits::message::MessageRepository;
use crate::db::traits::server::ServerRepository;
use crate::db::traits::server_category::ServerCategoryRepository;
use crate::db::traits::server_member::ServerMemberRepository;
use crate::models::channel::{ChannelId, ChannelType, DbChannel};
use crate::models::message::DbMessage;
use crate::models::server::ServerId;
use crate::models::server_category::{DbServerCategory, ServerCategoryId};
use crate::models::user::UserId;
use crate::util::pager::PageTokenizer;
// use crate::redis::RedisClient;

use super::model::Channel as ChannelModel;
use super::ycchat_channel::channel_server::Channel;
use super::ycchat_channel::{
    CreateChannelRequest, DeleteChannelRequest, ListServerChannelsRequest,
    ListServerChannelsResponse, SpeechRequest, SpeechResponse, UpdateChannelRequest,
};

pub struct ChannelService<SM, M, C, S, SC>
where
    SM: ServerMemberRepository<Surreal<Client>>,
    M: MessageRepository<Surreal<Client>>,
    C: ChannelRepository<Surreal<Client>>,
    S: ServerRepository<Surreal<Client>>,
    SC: ServerCategoryRepository<Surreal<Client>>,
{
    server_member_repository: SM,
    message_repository: M,
    channel_repository: C,
    server_repository: S,
    server_category_repository: SC,
    // redis_client: RedisClient,
}

impl<SM, M, C, S, SC> ChannelService<SM, M, C, S, SC>
where
    SM: ServerMemberRepository<Surreal<Client>>,
    M: MessageRepository<Surreal<Client>>,
    C: ChannelRepository<Surreal<Client>>,
    S: ServerRepository<Surreal<Client>>,
    SC: ServerCategoryRepository<Surreal<Client>>,
{
    pub fn new(
        server_member_repository: SM,
        message_repository: M,
        channel_repository: C,
        server_repository: S,
        server_category_repository: SC,
    ) -> Self {
        // let redis_client = RedisClient::new();

        ChannelService {
            server_member_repository,
            message_repository,
            channel_repository,
            server_repository,
            server_category_repository,
            // redis_client,
        }
    }
}

#[tonic::async_trait]
impl<SM, M, C, S, SC> Channel for ChannelService<SM, M, C, S, SC>
where
    SM: ServerMemberRepository<Surreal<Client>> + 'static,
    M: MessageRepository<Surreal<Client>> + 'static,
    C: ChannelRepository<Surreal<Client>> + 'static,
    S: ServerRepository<Surreal<Client>> + 'static,
    SC: ServerCategoryRepository<Surreal<Client>> + 'static,
{
    async fn list_server_channels(
        &self,
        request: Request<ListServerChannelsRequest>,
    ) -> Result<Response<ListServerChannelsResponse>, Status> {
        let db = conn().await;

        let request = request.into_inner();
        let parent = request.parent;

        let page_token = match request.page_token.clone() {
            Some(page_token) => {
                let page_token = crate::util::pager::get_page_token(page_token);
                Some(page_token.unwrap())
            }
            None => None,
        };

        let (page_size, offset_id, prev_page_token) = match page_token {
            Some(page_token) => (
                page_token.page_size,
                page_token
                    .offset_id
                    .map(|offset_id| ChannelId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        let parent = parent.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(parent[1]).unwrap();

        // page_size + 1 갯수만큼 데이터 로드 후 next_page_token None, Some 처리
        let mut channels = self
            .channel_repository
            .get_server_channels(&db, &server_id, page_size + 1, offset_id)
            .await
            .unwrap();

        let next_page_token = if channels.len() > usize::try_from(page_size).unwrap() {
            channels.pop();

            let next_page_token = channels.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                crate::util::base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        Ok(Response::new(ListServerChannelsResponse {
            channels: channels
                .into_iter()
                .map(|channel| channel.to_message())
                .collect::<Vec<ChannelModel>>(),
            next_page_token,
            prev_page_token,
        }))
    }

    async fn create_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<ChannelModel>, Status> {
        let db = conn().await;

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

                let server = self.server_repository.get_server(&db, &server_id).await;

                match server {
                    Ok(server) => server,
                    Err(err) => return Err(Status::not_found(err.as_str())),
                }
            }
            None => None,
        };

        let category: Option<DbServerCategory> = match category_index {
            Some(idx) => {
                let db = conn().await;

                let server_category_id: ServerCategoryId =
                    ServerCategoryId::from_str(name_list[idx]).unwrap();

                let category = self
                    .server_category_repository
                    .get(&db, &server_category_id)
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

        let added = self.channel_repository.add(&db, &channel).await.unwrap();

        match added {
            Some(channel) => Ok(Response::new(channel.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn update_channel(
        &self,
        request: Request<UpdateChannelRequest>,
    ) -> Result<Response<ChannelModel>, Status> {
        let db = conn().await;

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

                match self.channel_repository.get(&db, &channel_id).await {
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

        let res = self.channel_repository.update(&db, &exist).await.unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn delete_channel(
        &self,
        request: Request<DeleteChannelRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

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

        self.channel_repository
            .delete(&db, &channel_id)
            .await
            .unwrap();

        Ok(Response::new(()))
    }

    async fn speech(
        &self,
        request: Request<SpeechRequest>,
    ) -> Result<Response<SpeechResponse>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let req = request.into_inner();
        let name = req.name;
        let content = req.content;

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

        let message = DbMessage::new(user_id, channel_id, content);

        let message = self.message_repository.add(&db, &message).await.unwrap();

        match message {
            Some(message) => Ok(Response::new(SpeechResponse {
                result: Some(message.to_message()),
            })),
            None => Err(Status::internal("internal error")),
        }
    }
}
