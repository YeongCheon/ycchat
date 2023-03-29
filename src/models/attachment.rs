use std::iter::Map;

use chrono::{DateTime, Utc};

pub type AttachmentId = ulid::Ulid;

pub struct Attachment {
    pub id: AttachmentId,
    pub url: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u128,
    pub metadata: Map<String, String>,
    pub create_time: DateTime<Utc>,
}
