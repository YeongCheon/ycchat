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

#[derive(Clone)]
pub struct ServerMemberRepositoryImpl {}

impl ServerMemberRepositoryImpl {
    pub async fn new() -> Self {
        ServerMemberRepositoryImpl {}
    }
}

const COLLECTION_NAME: &str = "server_member";

#[tonic::async_trait]
impl ServerMemberRepository<Surreal<Client>> for ServerMemberRepositoryImpl {
    async fn get_server_member(
        &self,
        db: &Surreal<Client>,
        id: &ServerMemberId,
    ) -> Result<DbServerMember, String> {
        let res = db
            .select::<Option<DbServerMember>>((COLLECTION_NAME, id.to_string()))
            .await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn add_server_member(
        &self,
        db: &Surreal<Client>,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String> {
        let created: DbServerMember = db
            .create((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member)
            .await
            .unwrap();

        Ok(created)
    }

    async fn update_server_member(
        &self,
        db: &Surreal<Client>,
        server_member: &DbServerMember,
    ) -> Result<DbServerMember, String> {
        let res: Option<DbServerMember> = db
            .update((COLLECTION_NAME, server_member.id.to_string()))
            .content(server_member.clone())
            .await
            .unwrap();

        return Ok(res.unwrap());
    }

    async fn delete(&self, db: &Surreal<Client>, id: &ServerMemberId) -> Result<u8, String> {
        db.delete::<Option<DbServerMember>>((COLLECTION_NAME, id.to_string()))
            .await
            .unwrap();

        Ok(1)
    }

    async fn get_server_members(
        &self,
        db: &Surreal<Client>,
    ) -> Result<Vec<DbServerMember>, String> {
        let res = db.select::<Vec<DbServerMember>>(COLLECTION_NAME).await;

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_server_members_by_server_id(
        &self,
        db: &Surreal<Client>,
        server_id: &ServerId,
    ) -> Result<Vec<DbServerMember>, String> {
        let server = Thing {
            tb: SERVER_COLLECTION_NAME.to_string(),
            id: Id::String(server_id.to_string()),
        };

        let res = db
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
        db: &Surreal<Client>,
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

        let res = db
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
