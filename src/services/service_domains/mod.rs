use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        CreateServiceDomainDto, ServiceDomain, ServiceDomainById, ServiceDomains,
        ServiceDomainsByServiceId, UpdateServiceDomainDto,
        service_domain_service_client::ServiceDomainServiceClient,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct ServiceDomainsService;

impl ServiceDomainsService {
    async fn client() -> Result<ServiceDomainServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(ServiceDomainServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list(service_id: String) -> Result<ServiceDomains, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            ServiceDomainsByServiceId { service_id },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_service_domains(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<ServiceDomain, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            ServiceDomainById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_service_domain(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn create(
        dto: CreateServiceDomainDto,
    ) -> Result<ServiceDomain, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .create_service_domain(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn update(
        dto: UpdateServiceDomainDto,
    ) -> Result<ServiceDomain, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .update_service_domain(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn delete(id: String) -> Result<ServiceDomain, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            ServiceDomainById { id },
            AuthService::get_metadata().await?,
        );
        client
            .delete_service_domain(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
