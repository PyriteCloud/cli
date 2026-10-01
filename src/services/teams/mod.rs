use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        CreateTeamDto, Team, TeamById, Teams, TeamsRequest, UpdateTeamDto,
        team_service_client::TeamServiceClient,
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::{AuthService, ListQuery};

#[derive(Debug, Clone)]
pub(crate) struct TeamsService;

impl TeamsService {
    pub async fn get_teams_client() -> Result<TeamServiceClient<Channel>, Box<dyn std::error::Error>>
    {
        let client = TeamServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_teams(query: ListQuery) -> Result<Teams, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let (search, pagination) = match query {
            ListQuery::Browse(pagination) => (None, pagination),
            ListQuery::Search(search) => (Some(search), None),
        };
        let req: Request<TeamsRequest> = ReqWithMetadata::with_metadata(
            TeamsRequest {
                with_meta: Some(true),
                search,
                pagination,
            },
            metadata,
        );

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

    pub async fn create_team(
        create_team_dto: CreateTeamDto,
    ) -> Result<Team, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<CreateTeamDto> = ReqWithMetadata::with_metadata(create_team_dto, metadata);

        client
            .create_team(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn update_team(
        update_team_dto: UpdateTeamDto,
    ) -> Result<Team, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<UpdateTeamDto> = ReqWithMetadata::with_metadata(update_team_dto, metadata);

        client
            .update_team(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn delete_team(team_id: String) -> Result<Team, Box<dyn std::error::Error>> {
        let mut client = Self::get_teams_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<TeamById> =
            ReqWithMetadata::with_metadata(TeamById { id: team_id }, metadata);

        client
            .delete_team(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
