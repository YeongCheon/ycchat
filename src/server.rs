use tonic::transport::Server;

mod chat;
mod redis;

pub mod ycchat {
    tonic::include_proto!("ycchat");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();

    // let chat_service = chat::MyChatService::new();

    let chat_service_server = chat::get_chat_service_server();
    // ChatServiceServer::new(chat_service);

    println!("Start Server...");
    Server::builder()
        .add_service(chat_service_server)
        .serve(addr)
        .await?;

    Ok(())
}
