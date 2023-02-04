use tonic::{metadata::MetadataValue, Request, Status};

pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        _ => Ok(req),
        // Some(t) if token == t => Ok(req),
        // _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
