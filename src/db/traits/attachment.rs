use tonic::async_trait;

use crate::models::attachment::{Attachment, AttachmentId};

#[async_trait]
pub trait AttachmentRepository {
    async fn get_attachment(id: AttachmentId) -> Result<Option<Attachment>, String>;
    async fn add_attachment(attachment: Attachment) -> Result<Attachment, String>;
    async fn delete_attachment(id: AttachmentId) -> Result<u8, String>;
    async fn get_attachments() -> Result<Vec<Attachment>, String>;
}
