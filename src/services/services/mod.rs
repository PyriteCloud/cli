use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        ServiceById, ServicesByTeamIdOrProjectId, UpsertServiceDto,
        common::v1::{Service, Services},
        services_by_team_id_or_project_id::Id,
        services_service_client::ServicesServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct ServicesService;

impl ServicesService {
    pub async fn get_services_client()
    -> Result<ServicesServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ServicesServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_services(
        team_id: Option<String>,
        project_id: Option<String>,
    ) -> Result<Services, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<ServicesByTeamIdOrProjectId> = ReqWithMetadata::with_metadata(
            ServicesByTeamIdOrProjectId {
                id: project_id.map(Id::ProjectId).or(team_id.map(Id::TeamId)),
                with_meta: None,
                for_team_volume: None,
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
        let req: Request<ServiceById> = ReqWithMetadata::with_metadata(
            ServiceById {
                id: service_id,
                with_meta: Some(true),
            },
            metadata,
        );

        client
            .find_one_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn upsert_service(
        upsert_service_dto: UpsertServiceDto,
    ) -> Result<Service, Box<dyn std::error::Error>> {
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
}
