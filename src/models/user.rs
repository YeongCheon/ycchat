use chrono::{DateTime, Utc};

use super::attachment::{Attachment, AttachmentId};

pub type UserId = ulid::Ulid;

pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub description: String,
    pub avatar: Option<AttachmentId>, // FIXME
    pub region_code: Option<String>,
    pub language_code: Option<String>,
    pub time_zone: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
