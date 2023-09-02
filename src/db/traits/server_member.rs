use crate::models::{
    server::ServerId,
    server_member::{DbServerMember, ServerMemberId},
    user::UserId,
};

#[tonic::async_trait]
pub trait ServerMemberRepository<C>: Sync + Send {
    async fn get_server_member(
        &self,
        db: &C,
        id: &ServerMemberId,
    ) -> Result<DbServerMember, String>;

    async fn get_server_member_by_server_id_and_user_id(
        &self,
        db: &C,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> Result<Option<DbServerMember>, String>;

    async fn add_server_member(
        &self,
        db: &C,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String>;

    async fn update_server_member(
        &self,
        db: &C,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String>;

    async fn delete(&self, db: &C, id: &ServerMemberId) -> Result<u8, String>;

    async fn get_server_members(&self, db: &C) -> Result<Vec<DbServerMember>, String>;

    async fn get_server_members_by_server_id(
        &self,
        db: &C,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerMember>, String>;
}
