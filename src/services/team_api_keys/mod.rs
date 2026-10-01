use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::teams::v1::{
        CreateTeamApiKeyDto, TeamApiKey, TeamApiKeyByKey, TeamApiKeys, TeamApiKeysByTeamId,
        UpdateTeamApiKeyDto, team_api_key_service_client::TeamApiKeyServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct TeamApiKeysService;

impl TeamApiKeysService {
    async fn client() -> Result<TeamApiKeyServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(TeamApiKeyServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(team_id: String) -> Result<TeamApiKeys, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamApiKeysByTeamId { team_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_api_keys(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(key: String) -> Result<TeamApiKey, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamApiKeyByKey { key },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_api_key(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn create(
        dto: CreateTeamApiKeyDto,
    ) -> Result<TeamApiKey, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<CreateTeamApiKeyDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .create_api_key(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn update(
        dto: UpdateTeamApiKeyDto,
    ) -> Result<TeamApiKey, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<UpdateTeamApiKeyDto> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .update_api_key(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(key: String) -> Result<TeamApiKey, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TeamApiKeyByKey { key },
            AuthService::get_metadata().await?,
        );
        client
            .delete_api_key(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
