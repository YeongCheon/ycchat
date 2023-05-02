pub mod account;
pub mod auth;
pub mod server;
pub mod server_member;
pub mod user;

pub mod model {
    tonic::include_proto!("ycchat.model");
}

pub mod ycchat_account {
    tonic::include_proto!("ycchat.account");
}

pub mod ycchat_auth {
    tonic::include_proto!("ycchat.auth");
}

pub mod ycchat_user {
    tonic::include_proto!("ycchat.user");
}

pub mod ycchat_server {
    tonic::include_proto!("ycchat.server");

    pub mod member {
        tonic::include_proto!("ycchat.server.member");
    }
}
