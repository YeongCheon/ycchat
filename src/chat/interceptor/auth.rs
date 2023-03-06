use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use tonic::{metadata::AsciiMetadataValue, Request, Status};

use crate::util::variable::JWT_SECRET;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

pub fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
    let secret =
        env::var("YCCHAT_JWT_SECRET").expect("Missing YCCHAT_JWT_SECRET environment variable.");

    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    // JWT_SECRET

    if let Some(t) = req.metadata().get("authorization") {
        let b = t.as_bytes().to_vec();
        let token = String::from_utf8(b).unwrap();

        let token_data = match decode::<Claims>(&token, &key, &validation) {
            Ok(res) => res,
            Err(err) => {
                return Err(Status::unauthenticated(err.to_string()));
            }
        };

        let sub = token_data.claims.sub;
        let val: AsciiMetadataValue = match AsciiMetadataValue::try_from(sub.as_str()) {
            Ok(val) => val,
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        req.metadata_mut().append("user_id", val);

        Ok(req)
    } else {
        Err(Status::unauthenticated("No valid auth token"))
    }
}
