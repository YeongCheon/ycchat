use std::env;

lazy_static! {
    // openssl rand -base64 32 > jwt_secret.key
    pub static ref JWT_SECRET: String =
        env::var("YCCHAT_JWT_SECRET").expect("Missing YCCHAT_JWT_SECRET environment variable.");
    pub static ref PG_HOST: String =
        env::var("YCCHAT_PG_HOST").expect("Missing YCCHAT_PG_HOST environment variable.");
    pub static ref PG_USER: String =
        env::var("YCCHAT_PG_USER").expect("Missing YCCHAT_PG_USER environment variable.");
    pub static ref PG_PASSWORD: String =
        env::var("YCCHAT_PG_PASSWORD").expect("Missing YCCHAT_PG_PASSWORD environment variable.");
    pub static ref PG_DBNAME: String =
        env::var("YCCHAT_PG_DBNAME").expect("Missing YCCHAT_PG_DBNAME environment variable.");
}
