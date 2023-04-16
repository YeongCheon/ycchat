use tonic::async_trait;

use crate::models::attachment::{Attachment, AttachmentId};

#[async_trait]
pub trait AttachmentRepository {
    async fn get_attachment(self, id: &AttachmentId) -> Result<Option<Attachment>, String>;
    async fn add_attachment(self, attachment: Attachment) -> Result<Attachment, String>;
    async fn delete_attachment(self, id: &AttachmentId) -> Result<u8, String>;
    async fn get_attachments(self) -> Result<Vec<Attachment>, String>;
}
