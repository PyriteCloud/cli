use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        TeamMember, TeamMemberById, TeamMembers, TeamMembersByTeamId, UpdateTeamMemberDto,
        team_member_service_client::TeamMemberServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamMembersService;

impl TeamMembersService {
    async fn client() -> Result<TeamMemberServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamMemberServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(team_id: String) -> Result<TeamMembers, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamMembersByTeamId { team_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_team_members(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<TeamMember, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamMemberById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_team_member(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn update(
        dto: UpdateTeamMemberDto,
    ) -> Result<TeamMember, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<UpdateTeamMemberDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .update_team_member(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(id: String) -> Result<TeamMember, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamMemberById { id },
            AuthService::get_metadata().await?,
        );
        client
            .delete_team_member(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
