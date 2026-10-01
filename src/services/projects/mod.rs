use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::projects::v1::{
        CreateProjectDto, Project, ProjectById, Projects, ProjectsByTeamId, UpdateProjectDto,
        project_service_client::ProjectServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::{AuthService, ListQuery};

#[derive(Debug, Clone)]
pub(crate) struct ProjectsService;

impl ProjectsService {
    pub async fn get_projects_client()
    -> Result<ProjectServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ProjectServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_projects(
        team_id: Option<String>,
        query: ListQuery,
    ) -> Result<Projects, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let (search, pagination) = match query {
            ListQuery::Browse(pagination) => (None, pagination),
            ListQuery::Search(search) => (Some(search), None),
        };
        let req: Request<ProjectsByTeamId> = ReqWithMetadata::with_metadata(
            ProjectsByTeamId {
                team_id,
                with_meta: Some(true),
                search,
                pagination,
            },
            metadata,
        );

        client
            .find_all_projects(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn get_project(project_id: String) -> Result<Project, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ProjectById> = ReqWithMetadata::with_metadata(
            ProjectById {
                id: project_id,
                with_meta: Some(true),
                with_secrets: None,
            },
            metadata,
        );

        client
            .find_one_project(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn create_project(
        create_project_dto: CreateProjectDto,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<CreateProjectDto> =
            ReqWithMetadata::with_metadata(create_project_dto, metadata);

        client
            .create_project(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn update_project(
        update_project_dto: UpdateProjectDto,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<UpdateProjectDto> =
            ReqWithMetadata::with_metadata(update_project_dto, metadata);

        client
            .update_project(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn delete_project(project_id: String) -> Result<Project, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ProjectById> = ReqWithMetadata::with_metadata(
            ProjectById {
                id: project_id,
                with_meta: None,
                with_secrets: None,
            },
            metadata,
        );

        client
            .delete_project(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
