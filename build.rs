fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .compile(
            &[
                "protobuf/ycchat/v1/services/user/user.proto",
                "protobuf/ycchat/v1/services/server/server.proto",
                "protobuf/ycchat/v1/services/server/member/server_member.proto",
                "protobuf/ycchat/v1/services/server/category/category.proto",
                "protobuf/ycchat/v1/services/channel/channel.proto",
                "protobuf/ycchat/v1/services/message/message.proto",
                "protobuf/ycchat/v1/services/message/reaction.proto",
                "protobuf/ycchat/v1/services/auth/auth.proto",
                "protobuf/ycchat/v1/services/account/account.proto",
                "protobuf/ycchat/v1/services/connect/connect.proto",
                "protobuf/ycchat/v1/services/me/server/me_server.proto",
                "protobuf/ycchat/v1/services/me/user/me_user.proto",
            ],
            &["protobuf/"],
        )?;
    // tonic_build::compile_protos("protobuf/chatService.proto")?;
    Ok(())
}
