use tonic::{Request, Response, Status};

use crate::db::traits::server::ServerRepository;
use crate::db::traits::server_category::ServerCategoryRepository;
use crate::models::server::DbServer;
use crate::models::server::ServerId;
use crate::models::server_category::DbServerCategory;
use crate::models::server_category::ServerCategoryId;

use super::model::Category as CategoryModel;
use super::ycchat_server::category::category_server::Category;
use super::ycchat_server::category::CreateCategoryRequest;
use super::ycchat_server::category::{
    DeleteCategoryRequest, GetCategoryRequest, GetCategoryResponse, ListCategoriesRequest,
    ListCategoriesResponse, UpdateCategoryRequest,
};

pub struct ServerCategoryService<SC, S>
where
    SC: ServerCategoryRepository,
    S: ServerRepository,
{
    server_repository: S,
    server_category_repository: SC,
}

impl<SC, S> ServerCategoryService<SC, S>
where
    S: ServerRepository,
    SC: ServerCategoryRepository,
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
    S: ServerRepository + 'static,
    SC: ServerCategoryRepository + 'static,
{
    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        let parent = request.into_inner().parent;
        let parent = parent.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(parent[1]).unwrap();

        let list = self
            .server_category_repository
            .get_server_categories(&server_id)
            .await
            .unwrap()
            .into_iter()
            .map(|category| category.to_message())
            .collect::<Vec<CategoryModel>>();

        Ok(Response::new(ListCategoriesResponse {
            categories: list,
            page: None,
        }))
    }

    async fn get_category(
        &self,
        request: Request<GetCategoryRequest>,
    ) -> Result<Response<GetCategoryResponse>, Status> {
        let name = request.into_inner().name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = name[3].to_string();

        let category = self
            .server_category_repository
            .get(&server_category_id)
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
        let req = request.into_inner();

        let category = req.category.unwrap();

        let name = &category.name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();

        let server: DbServer = match self.server_repository.get_server(&server_id).await {
            Ok(server) => server,
            Err(_) => return Err(Status::not_found("server not found")),
        };

        let server_category = DbServerCategory::new(server, category);

        let res = self
            .server_category_repository
            .add(&server_category)
            .await
            .unwrap();

        Ok(Response::new(res.to_message()))
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CategoryModel>, Status> {
        let req = request.into_inner();
        let category = req.category.unwrap();

        let name = &category.name; // servers/{UUID}/categories/{UUID}
        let name = name.split('/').collect::<Vec<&str>>();
        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = name[3].to_string();

        let mut exist_category = self
            .server_category_repository
            .get(&server_category_id)
            .await
            .unwrap();

        if exist_category.is_none() {
            return Err(Status::not_found("entity not found."));
        }

        let mut exist_category = exist_category.unwrap();

        exist_category.update(category);

        let res = self
            .server_category_repository
            .update(&exist_category)
            .await
            .unwrap();

        Ok(Response::new(res.to_message()))
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        let name = req.name; // servers/{serverId}/members/{serverMemberId}
        let name = name.split('/').collect::<Vec<&str>>();

        let server_id = ServerId::from_string(name[1]).unwrap();
        let server_category_id: ServerCategoryId = name[3].to_string();

        self.server_category_repository
            .delete(&server_category_id)
            .await
            .unwrap();

        Ok(Response::new(()))
    }
}
