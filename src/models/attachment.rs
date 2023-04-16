use serde::{Deserialize, Serialize};

pub type AttachmentId = ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub id: uuid::Uuid,
    pub url: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    // pub metadata: Option<Map<String, String>>,
    pub create_time: chrono::NaiveDateTime,
}
