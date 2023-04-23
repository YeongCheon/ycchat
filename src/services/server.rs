use tonic::{Request, Response, Result, Status};

use crate::{
    db::traits::server::ServerRepository,
    models::server::{DbServer, ServerId},
};

use super::model::Server;
use super::ycchat_server::server_server::Server as ServerServer;
use super::ycchat_server::{
    CreateServerRequest, DeleteServerRequest, EnterServerRequest, EnterServerResponse,
    GetServerRequest, LeaveServerRequest, ListServersRequest, ListServersResponse,
    UpdateServerRequest,
};

pub struct ServerService<U>
where
    U: ServerRepository,
{
    server_repository: U,
}

impl<U> ServerService<U>
where
    U: ServerRepository,
{
    pub fn new(server_repository: U) -> Self {
        ServerService { server_repository }
    }
}

#[tonic::async_trait]
impl<U> ServerServer for ServerService<U>
where
    U: ServerRepository + 'static,
{
    async fn list_servers(
        &self,
        request: Request<ListServersRequest>,
    ) -> Result<Response<ListServersResponse>, Status> {
        let list = self.server_repository.get_servers().await.unwrap();

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
        let req = request.into_inner();
        let name = req.name;

        let id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        let server = self.server_repository.get_server(&id).await.unwrap();

        Ok(Response::new(server.to_message()))
    }

    async fn create_server(
        &self,
        request: Request<CreateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let req = request.into_inner();

        let server = match req.server {
            Some(server) => DbServer::new(server),
            None => return Err(Status::invalid_argument("invalid arguments")),
        };

        let server_res = self.server_repository.add_server(&server).await.unwrap();

        Ok(Response::new(server_res.to_message()))
    }

    async fn update_server(
        &self,
        request: Request<UpdateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        let req = request.into_inner();

        let server = match req.server {
            Some(server) => DbServer::from(server),
            None => return Err(Status::invalid_argument("invalid_arguments")),
        };

        let mut exist_server = self.server_repository.get_server(&server.id).await.unwrap();

        exist_server.display_name = server.display_name;
        exist_server.description = server.description;
        exist_server.update_time = chrono::offset::Utc::now();

        let res = self
            .server_repository
            .update_server(&exist_server)
            .await
            .unwrap();

        Ok(Response::new(res.to_message()))
    }

    async fn delete_server(
        &self,
        request: Request<DeleteServerRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let name = req.name;

        let id = ServerId::from_string(name.split('/').collect::<Vec<&str>>()[1]).unwrap();

        self.server_repository.delete_server(&id).await.unwrap();

        Ok(Response::new(()))
    }

    async fn enter_server(
        &self,
        request: Request<EnterServerRequest>,
    ) -> Result<Response<EnterServerResponse>, Status> {
        todo!("not implemented yet.");
    }

    async fn leave_server(
        &self,
        request: Request<LeaveServerRequest>,
    ) -> Result<Response<()>, Status> {
        todo!("not implemented yet.");
    }
}
