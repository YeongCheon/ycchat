use std::io::stdin;
use tokio_stream::StreamExt;
use tonic::{metadata::MetadataValue, transport::Channel, Request};
use ycchat::{
    chat_service_client::ChatServiceClient, ConnectRequest, EntryChatRoomRequest, SpeechRequest,
};

pub mod ycchat {
    tonic::include_proto!("ycchat");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;

    let token: MetadataValue<_> = "Bearer some-auth-token".parse()?;

    let mut client = ChatServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

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
                    ycchat::connect_response::Payload::ChatMessage(msg) => {
                        println!("{}: {}", msg.owner, msg.message);
                    }
                }
            }
        }
    });

    let room_id = {
        println!("insert room number");
        let mut buff = String::new();

        stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed!");

        client
            .entry_chat_room(EntryChatRoomRequest {
                parent: format!("rooms/{buff}"),
            })
            .await?;

        buff
    };

    loop {
        let mut buff = String::new();

        stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed!");
        let request = tonic::Request::new(SpeechRequest {
            parent: format!("rooms/{room_id}"),
            message: buff,
        });

        let _response = client.clone().speech(request).await?;
        // println!("RES={:?}", response);
    }

    // // let response = client.clone().speech(request).await?;

    // Ok(())
}
