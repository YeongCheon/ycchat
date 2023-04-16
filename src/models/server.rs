use chrono::{DateTime, Utc};

use super::attachment::Attachment;

pub type ServerId = ulid::Ulid;

pub struct Server {
    pub id: ServerId,
    pub display_name: String, // TITLE
    pub description: String,
    pub icon: Option<Attachment>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}
