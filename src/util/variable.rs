use std::env;

lazy_static! {
    pub static ref JWT_SECRET: String =
        env::var("YCCHAT_JWT_SECRET").expect("Missing YCCHAT_JWT_SECRET environment variable.");
}
