pub mod account;
pub mod auth;
pub mod channel;
pub mod connect;
pub mod message;
pub mod server;
pub mod server_category;
pub mod server_member;
pub mod user;

pub mod ycchat {
    pub mod v1 {
        pub mod models {
            tonic::include_proto!("ycchat.v1.models");
        }

        pub mod services {
            pub mod connect {
                tonic::include_proto!("ycchat.v1.services.connect");
            }

            pub mod message {
                tonic::include_proto!("ycchat.v1.services.message");
            }

            pub mod account {
                tonic::include_proto!("ycchat.v1.services.account");
            }

            pub mod auth {
                tonic::include_proto!("ycchat.v1.services.auth");
            }

            pub mod user {
                tonic::include_proto!("ycchat.v1.services.user");
            }

            pub mod server {
                tonic::include_proto!("ycchat.v1.services.server");

                pub mod category {
                    tonic::include_proto!("ycchat.v1.services.server.category");
                }

                pub mod member {
                    tonic::include_proto!("ycchat.v1.services.server.member");
                }
            }

            pub mod channel {
                tonic::include_proto!("ycchat.v1.services.channel");
            }
        }
    }
}
