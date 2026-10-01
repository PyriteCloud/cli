use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        TeamRegions, TeamRegionsByTeamId, team_region_service_client::TeamRegionServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamRegionsService;

impl TeamRegionsService {
    async fn client() -> Result<TeamRegionServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamRegionServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(team_id: String) -> Result<TeamRegions, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamRegionsByTeamId { team_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_regions(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
