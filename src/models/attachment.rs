use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::UserId;

pub type AttachmentId = ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub id: AttachmentId,
    pub url: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    // pub metadata: Option<Map<String, String>>,
    pub create_time: DateTime<Utc>,
}

pub struct AttachmentUplaoded {
    pub user: UserId,             // surreal relate 'in'
    pub attachment: AttachmentId, // surreal relate 'out'
}
