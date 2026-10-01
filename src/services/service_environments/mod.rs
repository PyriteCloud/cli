use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        CreateServiceEnvironmentDto, Pods, ServiceDetails, ServiceEnvironmentById,
        ServiceEnvironmentByIdWithDeploymentId, ServiceEnvironmentByTeamIdOrProjectIdOrServiceId,
        UpdateServiceEnvironmentDto,
        common::v1::{ServiceEnvironment, ServiceEnvironments},
        deployments::v1::Deployments,
        service_environment_by_team_id_or_project_id_or_service_id::Id,
        service_environment_service_client::ServiceEnvironmentServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::{AuthService, ListQuery};

#[derive(Debug, Clone)]
pub(crate) struct ServiceEnvironmentsService;

impl ServiceEnvironmentsService {
    fn request_by_id(id: String) -> ServiceEnvironmentById {
        ServiceEnvironmentById {
            id,
            search: None,
            status: None,
            pagination: None,
        }
    }

    pub async fn get_service_environments_client()
    -> Result<ServiceEnvironmentServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ServiceEnvironmentServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_service_environments(
        service_id: Option<String>,
        query: ListQuery,
    ) -> Result<ServiceEnvironments, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let (search, pagination) = match query {
            ListQuery::Browse(pagination) => (None, pagination),
            ListQuery::Search(search) => (Some(search), None),
        };
        let req: Request<ServiceEnvironmentByTeamIdOrProjectIdOrServiceId> =
            ReqWithMetadata::with_metadata(
                ServiceEnvironmentByTeamIdOrProjectIdOrServiceId {
                    id: service_id.map(Id::ServiceId),
                    search,
                    status: None,
                    pagination,
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
        let req: Request<ServiceEnvironmentById> =
            ReqWithMetadata::with_metadata(Self::request_by_id(service_environment_id), metadata);

        client
            .find_one_service_environment(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn create_service_environment(
        create_service_environment_dto: CreateServiceEnvironmentDto,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<CreateServiceEnvironmentDto> =
            ReqWithMetadata::with_metadata(create_service_environment_dto, metadata);

        client
            .create_service_environment(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn update_service_environment(
        update_service_environment_dto: UpdateServiceEnvironmentDto,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<UpdateServiceEnvironmentDto> =
            ReqWithMetadata::with_metadata(update_service_environment_dto, metadata);

        client
            .update_service_environment(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn delete_service_environment(
        service_environment_id: String,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServiceEnvironmentById> =
            ReqWithMetadata::with_metadata(Self::request_by_id(service_environment_id), metadata);

        client
            .delete_service_environment(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_deployments(
        service_environment_id: String,
    ) -> Result<Deployments, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .find_all_deployments(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn redeploy(
        service_environment_id: String,
        deployment_id: Option<String>,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            ServiceEnvironmentByIdWithDeploymentId {
                id: service_environment_id,
                deployment_id,
            },
            AuthService::get_metadata().await?,
        );
        client
            .redeploy_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn pause(
        service_environment_id: String,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .pause_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn resume(
        service_environment_id: String,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .resume_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn list_pods(
        service_environment_id: String,
    ) -> Result<Pods, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .find_pods_for_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get_details(
        service_environment_id: String,
    ) -> Result<ServiceDetails, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .find_details_for_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn sync_network(
        service_environment_id: String,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_service_environments_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_environment_id),
            AuthService::get_metadata().await?,
        );
        client
            .sync_network_for_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
