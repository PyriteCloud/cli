use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        ServiceLogs, ServiceLogsRequest, service_log_service_client::ServiceLogServiceClient,
    },
};
use tonic::Request;

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct ServiceLogsService;

impl ServiceLogsService {
    pub async fn list(
        request_data: ServiceLogsRequest,
    ) -> Result<ServiceLogs, Box<dyn std::error::Error>> {
        let mut client = ServiceLogServiceClient::connect(PYRITE_API_BASE_URL).await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(request_data, AuthService::get_metadata().await?);
        client
            .find_all_service_logs(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
