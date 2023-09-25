use surrealdb::{engine::remote::ws::Client, sql::Datetime, Surreal};
use tonic::{Request, Response, Result, Status};

use crate::{
    db::{
        surreal::{conn, server_member},
        traits::{server::ServerRepository, server_member::ServerMemberRepository},
    },
    models::{
        server::{DbServer, ServerId},
        server_member::DbServerMember,
        user::UserId,
    },
};

use super::model::{Server, ServerMember};
use super::ycchat_server::server_server::Server as ServerServer;
use super::ycchat_server::{
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

        let list = self.server_repository.get_servers(&db).await.unwrap();

        let servers: Vec<Server> = list.iter().map(|item| item.clone().to_message()).collect();

        let res = ListServersResponse {
            servers,
            page: None,
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

        Ok(Response::new(server.to_message()))
    }

    async fn create_server(
        &self,
        request: Request<CreateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let db = conn().await;

        let req = request.into_inner();

        let server = match req.server {
            Some(server) => DbServer::new(server),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let server_res = self
            .server_repository
            .add_server(&db, &server)
            .await
            .unwrap();

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

        let mut exist_server = self
            .server_repository
            .get_server(&db, &server.id)
            .await
            .unwrap();

        exist_server.display_name = server.display_name;
        exist_server.description = server.description;
        exist_server.update_time = Some(Datetime::default());

        let res = self
            .server_repository
            .update_server(&db, &exist_server)
            .await
            .unwrap();

        Ok(Response::new(res.to_message()))
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

        Ok(Response::new(server_member.to_message()))
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
