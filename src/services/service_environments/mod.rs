use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        ServiceEnvironmentById, ServiceEnvironmentByTeamIdOrProjectIdOrServiceId,
        common::v1::{ServiceEnvironment, ServiceEnvironments},
        service_environment_by_team_id_or_project_id_or_service_id::Id,
        service_environment_service_client::ServiceEnvironmentServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct ServiceEnvironmentsService;

impl ServiceEnvironmentsService {
    pub async fn get_service_environments_client()
    -> Result<ServiceEnvironmentServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ServiceEnvironmentServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_service_environments(
        service_id: String,
    ) -> Result<ServiceEnvironments, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServiceEnvironmentByTeamIdOrProjectIdOrServiceId> =
            ReqWithMetadata::with_metadata(
                ServiceEnvironmentByTeamIdOrProjectIdOrServiceId {
                    id: Some(Id::ServiceId(service_id)),
                },
                metadata,
            );

        client
            .find_all_service_environments(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn get_service_environment(
        service_environment_id: String,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServiceEnvironmentById> = ReqWithMetadata::with_metadata(
            ServiceEnvironmentById {
                id: service_environment_id,
            },
            metadata,
        );

        client
            .find_one_service_environment(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
