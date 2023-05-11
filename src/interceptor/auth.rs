use serde::{Deserialize, Serialize};
use tonic::{metadata::AsciiMetadataValue, Request, Status};

use crate::auth::jwt::decode;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

pub fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
    if let Some(t) = req.metadata().get("authorization") {
        let b = t.as_bytes().to_vec();
        let token = String::from_utf8(b).unwrap();

        let token_data = match decode(&token) {
            Ok(res) => res,
            Err(err) => {
                return Err(Status::unauthenticated(err.to_string()));
            }
        };

        let aud = token_data.claims.aud;

        let val: AsciiMetadataValue = match AsciiMetadataValue::try_from(aud.to_string()) {
            Ok(val) => val,
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        req.metadata_mut().append("user_id", val);

        Ok(req)
    } else {
        Err(Status::unauthenticated("No valid auth token"))
    }
}
