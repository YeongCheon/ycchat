use prost::Message as _;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::server_member::ServerMemberRepository;
use crate::models::server::ServerId;
use crate::models::server_member::ServerMemberId;
use crate::util::pager::PageTokenizer;

use super::ycchat::v1::models::ServerMember;

use super::ycchat::v1::services::server::member::server_member_service_server::ServerMemberService as ServerMemberServer;
use super::ycchat::v1::services::server::member::{
    GetServerMemberRequest, ListServerMembersRequest, ListServerMembersResponse,
};

pub struct ServerMemberService<U>
where
    U: ServerMemberRepository<Surreal<Client>>,
{
    server_member_repository: U,
}

impl<U> ServerMemberService<U>
where
    U: ServerMemberRepository<Surreal<Client>>,
{
    pub fn new(server_member_repository: U) -> Self {
        ServerMemberService {
            server_member_repository,
        }
    }
}

#[tonic::async_trait]
impl<U> ServerMemberServer for ServerMemberService<U>
where
    U: ServerMemberRepository<Surreal<Client>> + 'static,
{
    async fn list_server_members(
        &self,
        request: Request<ListServerMembersRequest>,
    ) -> Result<Response<ListServerMembersResponse>, Status> {
        let db = conn().await;
        let request = request.into_inner();
        let name = request.parent;
        let server_id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

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
                    .map(|offset_id| ServerMemberId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        let mut list = self
            .server_member_repository
            .get_server_members(&db, &server_id, page_size + 1, offset_id)
            .await
            .unwrap();

        let next_page_token = if list.len() > usize::try_from(page_size).unwrap() {
            list.pop();

            let next_page_token = list.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                crate::util::base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        let server_members: Vec<ServerMember> =
            list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListServerMembersResponse {
            server_members,
            next_page_token,
            prev_page_token,
        };

        Ok(Response::new(res))
    }

    async fn get_server_member(
        &self,
        request: Request<GetServerMemberRequest>,
    ) -> Result<Response<ServerMember>, Status> {
        let db = conn().await;

        let user_id = request
            .metadata()
            .get("user_id")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let req = request.into_inner();
        let name = req.name; // servers/{serverId}/members/{serverMemberId}
        let name = name.split('/').collect::<Vec<&str>>();

        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_member_id = ServerMemberId::from_string(name[3]).unwrap();

        let server_member = self
            .server_member_repository
            .get_server_member(&db, &server_member_id)
            .await
            .unwrap();

        let server_member = match server_member {
            Some(server_member) => server_member,
            None => {
                return Err(Status::not_found("not exist"));
            }
        };

        Ok(Response::new(server_member.to_message()))
    }
}
