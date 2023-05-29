use serde::{Serialize, Serializer};
use surrealdb::{
    engine::remote::ws::Client,
    sql::{Id, Thing},
    Surreal,
};

use super::server::COLLECTION_NAME as SERVER_COLLECTION_NAME;
use super::user::COLLECTION_NAME as USER_COLLECTION_NAME;
use crate::{
    db::traits::server_member::ServerMemberRepository,
    models::server::ServerId,
    models::server_member::{DbServerMember, ServerMemberId},
    models::user::UserId,
};

use super::conn;

#[derive(Clone)]
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

    async fn delete(&self, id: &ServerMemberId) -> Result<u8, String> {
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

    async fn get_server_members_by_server_id(
        &self,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerMember>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let res = self
            .db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE server == $server"
            ))
            .bind(("server", server))
            .await
            .unwrap()
            .take::<Vec<DbServerMember>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_server_member_by_server_id_and_user_id(
        &self,
        server_id: &ServerId,
        user_id: &UserId,
    ) -> Result<Option<DbServerMember>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let user = Thing {
            tb: USER_COLLECTION_NAME.to_string(),
            id: Id::String(user_id.to_string()),
        };

        let res = self
            .db
            .query(format!(
                "SELECT * FROM {COLLECTION_NAME} WHERE server == $server AND user == $user"
            ))
            .bind(("server", server))
            .bind(("user", user))
            .await
            .unwrap()
            .take::<Option<DbServerMember>>(0);

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn serialize_id<S>(id: &ServerMemberId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let surreal_id = Thing::from((COLLECTION_NAME.to_string(), id.to_string()));
    surreal_id.serialize(s)
}
