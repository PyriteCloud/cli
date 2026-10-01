use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        CreateTeamVolumeDto, TeamVolume, TeamVolumeById, TeamVolumeMetrics, TeamVolumes,
        TeamVolumesByTeamIdAndServiceId, UpdateTeamVolumeDto,
        team_volume_service_client::TeamVolumeServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamVolumesService;

impl TeamVolumesService {
    async fn client() -> Result<TeamVolumeServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamVolumeServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(
        team_id: String,
        service_environment_id: Option<String>,
    ) -> Result<TeamVolumes, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamVolumesByTeamIdAndServiceId {
                team_id,
                service_environment_id,
            },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_volumes(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<TeamVolume, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamVolumeById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_volume(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn create(
        dto: CreateTeamVolumeDto,
    ) -> Result<TeamVolume, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<CreateTeamVolumeDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .create_volume(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn update(
        dto: UpdateTeamVolumeDto,
    ) -> Result<TeamVolume, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<UpdateTeamVolumeDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .update_volume(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(id: String) -> Result<TeamVolume, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamVolumeById { id },
            AuthService::get_metadata().await?,
        );
        client
            .delete_volume(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn metrics(id: String) -> Result<TeamVolumeMetrics, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamVolumeById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_volume_metrics(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
