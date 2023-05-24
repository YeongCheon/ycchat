use crate::models::{
    server::ServerId,
    server_member::{DbServerMember, ServerMemberId},
};

#[tonic::async_trait]
pub trait ServerMemberRepository: Sync + Send {
    async fn get_server_member(&self, id: &ServerMemberId) -> Result<DbServerMember, String>;
    async fn add_server_member(
        &self,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String>;
    async fn update_server_member(
        &self,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String>;
    async fn delete_server(&self, id: &ServerMemberId) -> Result<u8, String>;
    async fn get_server_members(&self) -> Result<Vec<DbServerMember>, String>;
    async fn get_server_members_by_server_id(
        &self,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerMember>, String>;
}
