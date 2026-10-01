use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::{
        common::v1::Empty,
        teams::v1::{
            CreateTeamInvitationDto, TeamInvitation, TeamInvitationById, TeamInvitations,
            TeamInvitationsByTeamId, team_invitation_service_client::TeamInvitationServiceClient,
        },
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamInvitationsService;

impl TeamInvitationsService {
    async fn client() -> Result<TeamInvitationServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamInvitationServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list() -> Result<TeamInvitations, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(Empty {}, AuthService::get_metadata().await?);
        client
            .find_all_invitations(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn list_sent(team_id: String) -> Result<TeamInvitations, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamInvitationsByTeamId { team_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_sent_invitations(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<TeamInvitation, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamInvitationById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_invitation(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn create(
        dto: CreateTeamInvitationDto,
    ) -> Result<TeamInvitation, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<CreateTeamInvitationDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .create_invitation(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(id: String) -> Result<TeamInvitation, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamInvitationById { id },
            AuthService::get_metadata().await?,
        );
        client
            .delete_invitation(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn accept(id: String) -> Result<TeamInvitation, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamInvitationById { id },
            AuthService::get_metadata().await?,
        );
        client
            .accept_invitation(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
