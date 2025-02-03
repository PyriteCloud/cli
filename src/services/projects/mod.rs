use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::projects::v1::{
        project_service_client::ProjectServiceClient, Project, ProjectById, Projects,
        ProjectsByTeamId,
    },
};
use tonic::{transport::channel::Channel, Request};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct ProjectsService;

impl ProjectsService {
    pub async fn get_projects_client(
    ) -> Result<ProjectServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ProjectServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_projects(
        team_id: Option<String>,
    ) -> Result<Projects, Box<dyn std::error::Error>> {
        let mut client = Self::get_projects_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ProjectsByTeamId> =
            ReqWithMetadata::with_metadata(ProjectsByTeamId { team_id }, metadata);

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
}
