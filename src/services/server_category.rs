use prost::Message as _;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

use crate::db::surreal::conn;
use crate::db::traits::server::ServerRepository;
use crate::db::traits::server_category::ServerCategoryRepository;
use crate::models::server::DbServer;
use crate::models::server::ServerId;
use crate::models::server_category::DbServerCategory;
use crate::models::server_category::ServerCategoryId;
use crate::util::pager::PageTokenizer;

use super::model::Category as CategoryModel;
use super::ycchat_server::category::category_server::Category;
use super::ycchat_server::category::CreateCategoryRequest;
use super::ycchat_server::category::{
    DeleteCategoryRequest, GetCategoryRequest, GetCategoryResponse, ListCategoriesRequest,
    ListCategoriesResponse, UpdateCategoryRequest,
};

pub struct ServerCategoryService<SC, S>
where
    SC: ServerCategoryRepository<Surreal<Client>>,
    S: ServerRepository<Surreal<Client>>,
{
    server_repository: S,
    server_category_repository: SC,
}

impl<SC, S> ServerCategoryService<SC, S>
where
    S: ServerRepository<Surreal<Client>>,
    SC: ServerCategoryRepository<Surreal<Client>>,
{
    pub fn new(server_repository: S, server_category_repository: SC) -> Self {
        ServerCategoryService {
            server_repository,
            server_category_repository,
        }
    }
}

#[tonic::async_trait]
impl<SC, S> Category for ServerCategoryService<SC, S>
where
    S: ServerRepository<Surreal<Client>> + 'static,
    SC: ServerCategoryRepository<Surreal<Client>> + 'static,
{
    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        let db = conn().await;

        let request = request.into_inner();
        let parent = request.parent;
        let parent = parent.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(parent[1]).unwrap();

        let page_token = match request.page_token.clone() {
            Some(page_token) => {
                let page_token = crate::util::pager::get_page_token(page_token);
                Some(page_token.unwrap())
            }
            None => None,
        };

        let (page_size, offset_id, prev_page_token) = match page_token {
            Some(page_token) => (
                page_token.page_size,
                page_token
                    .offset_id
                    .map(|offset_id| ServerCategoryId::from_string(&offset_id).unwrap()),
                page_token.prev_page_token,
            ),
            None => (request.page_size, None, None),
        };

        let mut list = self
            .server_category_repository
            .get_server_categories(&db, &server_id, page_size + 1, offset_id)
            .await
            .unwrap();

        let next_page_token = if list.len() > usize::try_from(page_size).unwrap() {
            list.pop();

            let next_page_token = list.generate_page_token(page_size, request.page_token);
            next_page_token.map(|token| {
                let mut pb_buf = vec![];
                let _ = token.encode(&mut pb_buf);

                crate::util::base64_encoder::encode_string(pb_buf)
            })
        } else {
            None
        };

        Ok(Response::new(ListCategoriesResponse {
            categories: list
                .into_iter()
                .map(|category| category.to_message())
                .collect::<Vec<CategoryModel>>(),
            next_page_token,
            prev_page_token,
        }))
    }

    async fn get_category(
        &self,
        request: Request<GetCategoryRequest>,
    ) -> Result<Response<GetCategoryResponse>, Status> {
        let db = conn().await;

        let name = request.into_inner().name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = ServerCategoryId::from_string(name[3]).unwrap();

        let category = self
            .server_category_repository
            .get(&db, &server_category_id)
            .await
            .unwrap();

        let res = GetCategoryResponse {
            category: category.map(|item| item.to_message()),
            channels: vec![],
        };

        Ok(Response::new(res))
    }

    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CategoryModel>, Status> {
        let db = conn().await;
        let req = request.into_inner();

        let category = req.category.unwrap();

        let name = &category.name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();

        let server = match self.server_repository.get_server(&db, &server_id).await {
            Ok(server) => server,
            Err(_) => return Err(Status::not_found("server not found")),
        };
        let server: DbServer = match server {
            Some(server) => server,
            None => {
                return Err(Status::not_found("not exist"));
            }
        };

        let server_category = DbServerCategory::new(server, category);

        let res = self
            .server_category_repository
            .add(&db, &server_category)
            .await
            .unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CategoryModel>, Status> {
        let db = conn().await;
        let req = request.into_inner();
        let category = req.category.unwrap();

        let name = &category.name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = ServerCategoryId::from_string(name[3]).unwrap();

        let mut exist_category = self
            .server_category_repository
            .get(&db, &server_category_id)
            .await
            .unwrap();

        if exist_category.is_none() {
            return Err(Status::not_found("entity not found."));
        }

        let mut exist_category = exist_category.unwrap();

        exist_category.update(category);

        let res = self
            .server_category_repository
            .update(&db, &exist_category)
            .await
            .unwrap();

        match res {
            Some(res) => Ok(Response::new(res.to_message())),
            None => Err(Status::internal("internal error")),
        }
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<()>, Status> {
        let db = conn().await;

        let req = request.into_inner();
        let name = req.name; // servers/{serverId}/members/{serverMemberId}
        let name = name.split('/').collect::<Vec<&str>>();

        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = ServerCategoryId::from_string(name[3]).unwrap();

        self.server_category_repository
            .delete(&db, &server_category_id)
            .await
            .unwrap();

        Ok(Response::new(()))
    }
}
