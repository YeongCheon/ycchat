use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::server_member::ServerMemberRepository;
use crate::models::server::ServerId;
use crate::models::server_member::ServerMemberId;

use super::model::ServerMember;

use super::ycchat_server::member::server_member_server::ServerMember as ServerMemberServer;
use super::ycchat_server::member::{
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

        let list = self
            .server_member_repository
            .get_server_members(&db)
            .await
            .unwrap();

        let server_members: Vec<ServerMember> =
            list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListServerMembersResponse {
            server_members,
            page: None,
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
