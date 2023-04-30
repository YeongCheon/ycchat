fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "protobuf/model/attachment.proto",
            "protobuf/model/category.proto",
            "protobuf/model/channel.proto",
            "protobuf/model/message.proto",
            "protobuf/model/page.proto",
            "protobuf/model/reaction.proto",
            "protobuf/model/server.proto",
            "protobuf/model/server_member.proto",
            "protobuf/model/user.proto",
            //---------------------------------------
            "protobuf/user/user.proto",
            "protobuf/server/server.proto",
            "protobuf/server/member/server_member.proto",
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
