use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{codegen::InterceptedService, Request, Response, Status};
use ycchat::chat_service_server::ChatService;

use self::{
    chat::ChatServerService,
    ycchat::{
        chat_service_server::ChatServiceServer, ConnectResponse, ListChatRoomUsersRequest,
        ListChatRoomUsersResponse, ListChatRoomsRequest, ListChatRoomsResponse,
    },
};

mod chat;
mod chat_message_pager;
mod chat_room_pager;
mod chat_room_user_pager;
mod paging;

pub mod ycchat {
    tonic::include_proto!("ycchat");
}

const METADATA_AUTH_KEY: &str = "authorization";

pub struct MyChatService {
    chat_service: ChatServerService,
}

impl MyChatService {
    pub fn new() -> Self {
        let chat_service = ChatServerService::new();
        MyChatService { chat_service }
    }
}

#[tonic::async_trait]
impl ChatService for MyChatService {
    async fn list_chat_rooms(
        &self,
        request: tonic::Request<ListChatRoomsRequest>,
    ) -> Result<tonic::Response<ListChatRoomsResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        return self
            .chat_service
            .list_chat_rooms(user_id, request.into_inner())
            .map(Response::new);
    }

    async fn list_chat_room_users(
        &self,
        request: tonic::Request<ListChatRoomUsersRequest>,
    ) -> Result<tonic::Response<ListChatRoomUsersResponse>, tonic::Status> {
        return self
            .chat_service
            .list_chat_room_users(request.into_inner())
            .map(Response::new);
    }

    async fn list_chat_messages(
        &self,
        request: tonic::Request<ycchat::ListChatMessagesRequest>,
    ) -> Result<tonic::Response<ycchat::ListChatMessagesResponse>, tonic::Status> {
        return self
            .chat_service
            .list_chat_messages(request.into_inner())
            .map(Response::new);
    }

    async fn read_chat_message(
        &self,
        request: tonic::Request<ycchat::ReadChatMessageRequest>,
    ) -> Result<tonic::Response<ycchat::ReadChatMessageResponse>, tonic::Status> {
        todo!("")
    }

    async fn entry_chat_room(
        &self,
        request: tonic::Request<ycchat::EntryChatRoomRequest>,
    ) -> Result<tonic::Response<ycchat::EntryChatRoomResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        return self
            .chat_service
            .entry_chat_room(&user_id, request.into_inner())
            .map(Response::new);
    }

    async fn leave_chat_room(
        &self,
        request: tonic::Request<ycchat::LeaveChatRoomRequest>,
    ) -> Result<tonic::Response<ycchat::LeaveChatRoomResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        return self
            .chat_service
            .leave_chat_room(&user_id, request.into_inner())
            .map(Response::new);
    }

    type ConnStream =
        Pin<Box<dyn Stream<Item = Result<ConnectResponse, Status>> + Send + Sync + 'static>>;

    async fn conn(
        &self,
        request: tonic::Request<ycchat::ConnectRequest>,
    ) -> Result<tonic::Response<Self::ConnStream>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        let stream = self.chat_service.conn(&user_id, request.into_inner()).await;

        return match stream {
            Ok(res) => Ok(Response::new(res)),
            Err(_) => Err(Status::internal("internal server error")),
        };
    }

    async fn speech(
        &self,
        request: tonic::Request<ycchat::SpeechRequest>,
    ) -> Result<tonic::Response<ycchat::SpeechResponse>, tonic::Status> {
        let user_id = request
            .metadata()
            .get(METADATA_AUTH_KEY)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(); // FIXME

        return self
            .chat_service
            .speech(&user_id, request.into_inner())
            .map(Response::new);
    }
}

type Interceptor = fn(Request<()>) -> Result<Request<()>, Status>;

pub fn get_chat_service_server() -> InterceptedService<ChatServiceServer<MyChatService>, Interceptor>
{
    let chat_service = MyChatService::new();
    ChatServiceServer::with_interceptor(chat_service, interceptor::auth::check_auth)
}
