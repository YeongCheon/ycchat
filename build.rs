fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "protobuf/user/user.proto",
            "protobuf/server/server.proto",
            "protobuf/server/member/server_member.proto",
            "protobuf/server/category/category.proto",
            "protobuf/channel/channel.proto",
            "protobuf/message/message.proto",
            "protobuf/message/reaction.proto",
            "protobuf/auth/auth.proto",
            "protobuf/account/account.proto",
            "protobuf/connect/connect.proto",
        ],
        &["protobuf/"],
    )?;
    // tonic_build::compile_protos("protobuf/chatService.proto")?;
    Ok(())
}
