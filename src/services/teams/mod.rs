use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::{
        common::v1::Empty,
        teams::v1::{Team, TeamById, Teams, team_service_client::TeamServiceClient},
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct TeamsService;

impl TeamsService {
    pub async fn get_teams_client() -> Result<TeamServiceClient<Channel>, Box<dyn std::error::Error>>
    {
        let client = TeamServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_teams() -> Result<Teams, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty::default(), metadata);

        client
            .find_all_teams(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn get_team(team_id: String) -> Result<Team, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<TeamById> =
            ReqWithMetadata::with_metadata(TeamById { id: team_id }, metadata);

        client
            .find_one_team(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
