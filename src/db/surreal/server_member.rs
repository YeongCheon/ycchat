use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    db::traits::server_member::ServerMemberRepository,
    models::server_member::{DbServerMember, ServerMemberId},
};

use super::conn;

pub struct ServerMemberRepositoryImpl {
    db: Surreal<Client>,
}

impl ServerMemberRepositoryImpl {
    pub async fn new() -> Self {
        ServerMemberRepositoryImpl { db: conn().await }
    }
}

const COLLECTION_NAME: &str = "server_member";

#[tonic::async_trait]
impl ServerMemberRepository for ServerMemberRepositoryImpl {
    async fn get_server_member(&self, id: &ServerMemberId) -> Result<DbServerMember, String> {
        let res = self
            .db
            .select::<Option<DbServerMember>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_server_member(
        &self,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String> {
        let created: DbServerMember = self
            .db
            .create((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member)
            .await
            .unwrap();
        dbg!(&created);

        Ok(created)
    }

    async fn update_server_member(
        &self,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String> {
        let res: Option<DbServerMember> = self
            .db
            .update((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete_server(&self, id: &ServerMemberId) -> Result<u8, String> {
        self.db
            .delete::<Option<DbServerMember>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_server_members(&self) -> Result<Vec<DbServerMember>, String> {
        let res = self.db.select::<Vec<DbServerMember>>(COLLECTION_NAME).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}
