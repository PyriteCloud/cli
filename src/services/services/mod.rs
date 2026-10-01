use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        CreateServiceDto, ServiceById, ServiceByIdWithEnvironment, ServiceToken,
        ServicesByTeamIdOrProjectId, UpsertServiceDto, UpsertServiceResponseDto,
        common::v1::{Service, ServiceEnvironment, Services},
        deployments::v1::Deployments,
        services_by_team_id_or_project_id::Id,
        services_service_client::ServicesServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::{AuthService, ListQuery};

#[derive(Debug, Clone)]
pub(crate) struct ServicesService;

impl ServicesService {
    fn request_by_id(id: String, with_meta: Option<bool>) -> ServiceById {
        ServiceById {
            id,
            with_meta,
            r#type: None,
            status: None,
            search: None,
            pagination: None,
        }
    }

    fn request_by_id_with_environment(
        id: String,
        environment: Option<String>,
    ) -> ServiceByIdWithEnvironment {
        ServiceByIdWithEnvironment {
            id,
            environment,
            with_meta: None,
            r#type: None,
            status: None,
            search: None,
            pagination: None,
        }
    }

    pub async fn get_services_client()
    -> Result<ServicesServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ServicesServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_services(
        team_id: Option<String>,
        project_id: Option<String>,
        query: ListQuery,
    ) -> Result<Services, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let (search, pagination) = match query {
            ListQuery::Browse(pagination) => (None, pagination),
            ListQuery::Search(search) => (Some(search), None),
        };
        let with_meta = project_id.is_none().then_some(true);
        let req: Request<ServicesByTeamIdOrProjectId> = ReqWithMetadata::with_metadata(
            ServicesByTeamIdOrProjectId {
                id: project_id.map(Id::ProjectId).or(team_id.map(Id::TeamId)),
                with_meta,
                for_team_volume: None,
                r#type: None,
                status: None,
                search,
                pagination,
            },
            metadata,
        );

        client
            .find_all_services(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn get_service(service_id: String) -> Result<Service, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServiceById> =
            ReqWithMetadata::with_metadata(Self::request_by_id(service_id, Some(true)), metadata);

        client
            .find_one_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn upsert_service(
        upsert_service_dto: UpsertServiceDto,
    ) -> Result<UpsertServiceResponseDto, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;

        let req: Request<UpsertServiceDto> =
            ReqWithMetadata::with_metadata(upsert_service_dto, metadata);

        client
            .upsert_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn create_service(
        create_service_dto: CreateServiceDto,
    ) -> Result<Service, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<CreateServiceDto> =
            ReqWithMetadata::with_metadata(create_service_dto, metadata);

        client
            .create_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn delete_service(service_id: String) -> Result<Service, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServiceById> =
            ReqWithMetadata::with_metadata(Self::request_by_id(service_id, None), metadata);

        client
            .delete_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_deployments(
        service_id: String,
        environment: Option<String>,
    ) -> Result<Deployments, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id_with_environment(service_id, environment),
            AuthService::get_metadata().await?,
        );
        client
            .find_all_deployments(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn redeploy(
        service_id: String,
        environment: Option<String>,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id_with_environment(service_id, environment),
            AuthService::get_metadata().await?,
        );
        client
            .redeploy_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn pause(
        service_id: String,
        environment: Option<String>,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id_with_environment(service_id, environment),
            AuthService::get_metadata().await?,
        );
        client
            .pause_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn resume(
        service_id: String,
        environment: Option<String>,
    ) -> Result<ServiceEnvironment, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id_with_environment(service_id, environment),
            AuthService::get_metadata().await?,
        );
        client
            .resume_service_environment(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get_token(service_id: String) -> Result<ServiceToken, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_id, None),
            AuthService::get_metadata().await?,
        );
        client
            .find_token_for_service(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn regenerate_token(
        service_id: String,
    ) -> Result<ServiceToken, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_id, None),
            AuthService::get_metadata().await?,
        );
        client
            .regenerate_token_for_service(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn sync_network(service_id: String) -> Result<Service, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            Self::request_by_id(service_id, None),
            AuthService::get_metadata().await?,
        );
        client
            .sync_network_for_all_service_environments(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
