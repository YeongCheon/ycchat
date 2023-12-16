use prost::Message;
use surrealdb::{engine::remote::ws::Client, sql::Datetime, Surreal};
use tonic::{Request, Response, Result, Status};

use crate::{
    db::{
        surreal::conn,
        traits::{server::ServerRepository, server_member::ServerMemberRepository},
    },
    models::{
        server::{DbServer, ServerId},
        server_member::DbServerMember,
        user::UserId,
    },
    util::{self, base64_encoder, pager::PageTokenizer},
};

use super::ycchat::v1::models::{Server, ServerMember};
use super::ycchat::v1::services::server::server_service_server::ServerService as ServerServer;
use super::ycchat::v1::services::server::{
    CreateServerRequest, DeleteServerRequest, EnterServerRequest, GetServerRequest,
    LeaveServerRequest, ListServersRequest, ListServersResponse, UpdateServerRequest,
};

pub struct ServerService<U, M>
where
    U: ServerRepository<Surreal<Client>>,
    M: ServerMemberRepository<Surreal<Client>>,
{
    server_repository: U,
    server_member_repository: M,
}

impl<U, M> ServerService<U, M>
where
    U: ServerRepository<Surreal<Client>>,
    M: ServerMemberRepository<Surreal<Client>>,
{
    pub fn new(server_repository: U, server_member_repository: M) -> Self {
        ServerService {
            server_repository,
            server_member_repository,
        }
    }
}

#[tonic::async_trait]
impl<U, M> ServerServer for ServerService<U, M>
where
    U: ServerRepository<Surreal<Client>> + 'static,
    M: ServerMemberRepository<Surreal<Client>> + 'static,
{
    async fn list_servers(
        &self,
        request: Request<ListServersRequest>,
    ) -> Result<Response<ListServersResponse>, Status> {
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
            .get_servers(&db, page_size + 1, offset_id)
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

        let res = ListServersResponse {
            servers,
            next_page_token,
            prev_page_token,
        };

        Ok(Response::new(res))
    }

    async fn get_server(
        &self,
        request: Request<GetServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let db = conn().await;

        let req = request.into_inner();
        let name = req.name;

        let id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let server = self.server_repository.get_server(&db, &id).await.unwrap();

        match server {
            Some(server) => Ok(Response::new(server.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn create_server(
        &self,
        request: Request<CreateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let req = request.into_inner();

        let server = match req.server {
            Some(server) => DbServer::new(user_id, server),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let server_res = self
            .server_repository
            .add_server(&db, &server)
            .await
            .unwrap();
        let server_res = match server_res {
            Some(server) => server,
            None => return Err(Status::internal("failed to create server")),
        };

        let display_name = "username".to_string(); // FIXME
        let description = "server_description".to_string(); // FIXME
        let server_id = server_res.id;

        let server_member = DbServerMember::new(display_name, description, server_id, user_id);

        let server_member = self
            .server_member_repository
            .add_server_member(&db, &server_member)
            .await;

        match server_member {
            Ok(_) => {}
            Err(_err) => return Err(Status::internal("failed to create server_member")),
        }

        Ok(Response::new(server_res.to_message()))
    }

    async fn update_server(
        &self,
        request: Request<UpdateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let db = conn().await;

        let req = request.into_inner();

        let server = match req.server {
            Some(server) => DbServer::from(server),
            None => return Err(Status::invalid_argument("invalid_arguments")),
        };

        let exist_server = self
            .server_repository
            .get_server(&db, &server.id)
            .await
            .unwrap();

        let mut exist_server = match exist_server {
            Some(exist_server) => exist_server,
            None => return Err(Status::not_found("not found")),
        };

        exist_server.display_name = server.display_name;
        exist_server.description = server.description;
        exist_server.update_time = Some(Datetime::default());

        let res = self
            .server_repository
            .update_server(&db, &exist_server)
            .await
            .unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn delete_server(
        &self,
        request: Request<DeleteServerRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;
        let req = request.into_inner();
        let name = req.name;

        let id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        self.server_repository
            .delete_server(&db, &id)
            .await
            .unwrap();

        Ok(Response::new(()))
    }

    async fn enter_server(
        &self,
        request: Request<EnterServerRequest>,
    ) -> Result<Response<ServerMember>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(&user_id).unwrap();

        let req = request.into_inner();
        let name = req.name;
        let display_name = req.display_name;
        let description = req.description;

        let server_id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        {
            // check exist
            let exist = self
                .server_member_repository
                .get_server_member_by_server_id_and_user_id(&db, &server_id, &user_id)
                .await
                .unwrap();

            if exist.is_some() {
                return Err(Status::already_exists("already exist."));
            }
        }

        let server_member = DbServerMember::new(display_name, description, server_id, user_id);

        let server_member = self
            .server_member_repository
            .add_server_member(&db, &server_member)
            .await
            .unwrap();

        match server_member {
            Some(server_member) => Ok(Response::new(server_member.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn leave_server(
        &self,
        request: Request<LeaveServerRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

        let user_id = request.metadata().get("user_id").unwrap().to_str().unwrap();
        let user_id = UserId::from_string(user_id).unwrap();

        let req = request.into_inner();
        let name = req.name;

        let server_id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let exist = self
            .server_member_repository
            .get_server_member_by_server_id_and_user_id(&db, &server_id, &user_id)
            .await
            .unwrap();

        match exist {
            Some(exist) => {
                self.server_member_repository
                    .delete(&db, &exist.id)
                    .await
                    .unwrap();
            }
            None => return Err(Status::not_found("not found.")),
        }

        Ok(Response::new(()))
    }
}
