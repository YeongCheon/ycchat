use tonic::async_trait;

use crate::models::attachment::{Attachment, AttachmentId};

#[async_trait]
pub trait AttachmentRepository<C> {
    async fn get_attachment(&self, db: &C, id: &AttachmentId)
        -> Result<Option<Attachment>, String>;

    async fn add_attachment(&self, db: &C, attachment: Attachment) -> Result<Attachment, String>;

    async fn delete_attachment(&self, db: &C, id: &AttachmentId) -> Result<u8, String>;

    async fn get_attachments(&self, db: &C) -> Result<Vec<Attachment>, String>;
}
