fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().proto_path("protobuf").compile(
        &[
            "protobuf/user/user.proto",
            "protobuf/server/server.proto",
            "protobuf/server/server_member.proto",
            "protobuf/category/category.proto",
            "protobuf/channel/channel.proto",
            "protobuf/message/message.proto",
            "protobuf/message/reaction.proto",
        ],
        &["protobuf"],
    )?;
    // tonic_build::compile_protos("protobuf/chatService.proto")?;
    Ok(())
}
