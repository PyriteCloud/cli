use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        CreateTeamRegistryDto, TeamRegistries, TeamRegistriesByTeamId, TeamRegistry,
        TeamRegistryById, UpdateTeamRegistryDto,
        team_registry_service_client::TeamRegistryServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamRegistriesService;

impl TeamRegistriesService {
    async fn client() -> Result<TeamRegistryServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamRegistryServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(team_id: String) -> Result<TeamRegistries, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamRegistriesByTeamId { team_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_registries(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<TeamRegistry, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamRegistryById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_registry(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn create(
        dto: CreateTeamRegistryDto,
    ) -> Result<TeamRegistry, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<CreateTeamRegistryDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .create_registry(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn update(
        dto: UpdateTeamRegistryDto,
    ) -> Result<TeamRegistry, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<UpdateTeamRegistryDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .update_registry(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(id: String) -> Result<TeamRegistry, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamRegistryById { id },
            AuthService::get_metadata().await?,
        );
        client
            .delete_registry(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
