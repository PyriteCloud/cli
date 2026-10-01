use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::{
        common::v1::Empty,
        misc::v1::{
            FixedPrices, Items, Plans, PlansByServiceType, Regions, TeamSubscriptions,
            misc_service_client::MiscServiceClient,
        },
    },
};
use tonic::{Request, transport::channel::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct MiscService;

impl MiscService {
    async fn get_misc_client() -> Result<MiscServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = MiscServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn list_service_types() -> Result<Items, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_service_types(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_roles() -> Result<Items, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_roles(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_protocols() -> Result<Items, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_protocols(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_runtimes() -> Result<Items, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_runtimes(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_builders() -> Result<Items, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_builders(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_regions() -> Result<Regions, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_regions(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_plans(
        service_type: Option<String>,
    ) -> Result<Plans, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<PlansByServiceType> =
            ReqWithMetadata::with_metadata(PlansByServiceType { service_type }, metadata);

        client
            .find_all_plans(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_team_subscriptions() -> Result<TeamSubscriptions, Box<dyn std::error::Error>>
    {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_team_subscriptions(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }

    pub async fn list_fixed_prices() -> Result<FixedPrices, Box<dyn std::error::Error>> {
        let mut client = Self::get_misc_client().await?;
        let metadata = AuthService::get_metadata().await?;
        let req: Request<Empty> = ReqWithMetadata::with_metadata(Empty {}, metadata);

        client
            .find_all_fixed_prices(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
