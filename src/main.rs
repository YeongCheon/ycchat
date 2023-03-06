use tonic::transport::Server;

mod chat;
mod redis;
mod util;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse().unwrap();

    let chat_service_server = chat::get_chat_service_server();

    println!("Start Server...");
    Server::builder()
        .add_service(chat_service_server)
        .serve(addr)
        .await?;

    Ok(())
}
