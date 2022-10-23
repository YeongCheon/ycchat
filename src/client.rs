use std::io::stdin;
use tokio_stream::StreamExt;
use ycchat::{chat_service_client::ChatServiceClient, ConnectRequest, SpeechRequest};

pub mod ycchat {
    tonic::include_proto!("ycchat");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ChatServiceClient::connect("http://127.0.0.1:50051").await?;
    let mut receiver = client.clone();

    tokio::spawn(async move {
        let conn_request = ConnectRequest {};

        let mut stream = receiver.conn(conn_request).await.unwrap().into_inner();

        while let Some(item) = stream.next().await {
            if let Some(payload) = item.unwrap().payload {
                match payload {
                    ycchat::connect_response::Payload::ConnectSuccess(conn_response) => {
                        println!("Connect Complete: {}", conn_response.result)
                    }
                    ycchat::connect_response::Payload::ReceiveMessage(msg) => {
                        println!("{}: {}", msg.owner, msg.message);
                    }
                }
            }
        }
    });

    loop {
        let mut buff = String::new();

        stdin()
            .read_line(&mut buff)
            .expect("REading from stdin failed!");
        let request = tonic::Request::new(SpeechRequest {
            room_id: 1.to_string(),
            message: buff,
        });

        let _response = client.clone().speech(request).await?;
        // println!("RES={:?}", response);
    }

    // // let response = client.clone().speech(request).await?;

    // Ok(())
}
