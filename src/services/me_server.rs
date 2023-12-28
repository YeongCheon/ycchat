use prost::Message;
use surrealdb::{engine::remote::ws::Client, sql::Datetime, Surreal};
use tonic::{Request, Response, Result, Status};

use crate::{
    db::{
        surreal::conn,
        traits::{server::ServerRepository, server_member::ServerMemberRepository},
    },
    models::{server::ServerId, user::UserId},
    util::{self, base64_encoder, pager::PageTokenizer},
};

use super::ycchat::v1::models::Server;
use super::ycchat::v1::services::me::server::{
    me_server_service_server::MeServerService as MeServerServiceServer, ListMeServersRequest,
    ListMeServersResponse,
};

pub struct MeServerService<U, M>
where
    U: ServerRepository<Surreal<Client>>,
    M: ServerMemberRepository<Surreal<Client>>,
{
    server_repository: U,
    server_member_repository: M,
}

impl<U, M> MeServerService<U, M>
where
    U: ServerRepository<Surreal<Client>>,
    M: ServerMemberRepository<Surreal<Client>>,
{
    pub fn new(server_repository: U, server_member_repository: M) -> Self {
        MeServerService {
            server_repository,
            server_member_repository,
        }
    }
}

#[tonic::async_trait]
impl<U, M> MeServerServiceServer for MeServerService<U, M>
where
    U: ServerRepository<Surreal<Client>> + 'static,
    M: ServerMemberRepository<Surreal<Client>> + 'static,
{
    async fn list_me_servers(
        &self,
        request: Request<ListMeServersRequest>,
    ) -> Result<Response<ListMeServersResponse>, Status> {
        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let db = conn().await;
        let request = request.into_inner();
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
                    .map(|offset_id| ServerId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        // page_size + 1 갯수만큼 데이터 로드 후 next_page_token None, Some 처리
        let mut list = self
            .server_repository
            .get_joined_servers(&db, &user_id, page_size + 1, offset_id)
            .await
            .unwrap();

        let next_page_token = if list.len() > usize::try_from(page_size).unwrap() {
            list.pop();
            let next_page_token = list.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        let servers: Vec<Server> = list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListMeServersResponse {
            servers,
            next_page_token,
            prev_page_token,
        };

        Ok(Response::new(res))
    }
}
