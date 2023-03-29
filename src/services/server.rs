use tonic::{Request, Response, Result, Status};

use self::ycchat_server::{
    CreateServerRequest, DeleteServerRequest, EnterServerRequest, EnterServerResponse,
    GetServerRequest, GetServerResponse, LeaveServerRequest, ListServersRequest,
    ListServersResponse, UpdateServerRequest,
};
use model::Server;
use ycchat_server::server_server::Server as ServerServer;

pub mod model {
    tonic::include_proto!("ycchat.model");
}

pub mod ycchat_server {
    tonic::include_proto!("ycchat.server");
}

pub struct ServerService {}

impl ServerService {
    pub fn new() -> Self {
        ServerService {}
    }
}

#[tonic::async_trait]
impl ServerServer for ServerService {
    async fn list_servers(
        &self,
        request: Request<ListServersRequest>,
    ) -> Result<Response<ListServersResponse>, Status> {
        todo!("not implemented yet.");
    }

    async fn get_server(
        &self,
        request: Request<GetServerRequest>,
    ) -> Result<Response<GetServerResponse>, Status> {
        todo!("not implemented yet.");
    }

    async fn create_server(
        &self,
        request: Request<CreateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        todo!("not implemented yet.");
    }

    async fn update_server(
        &self,
        request: Request<UpdateServerRequest>,
    ) -> Result<Response<Server>, Status> {
        todo!("not implemented yet.");
    }

    async fn delete_server(
        &self,
        request: Request<DeleteServerRequest>,
    ) -> Result<Response<()>, Status> {
        todo!("not implemented yet.");
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
