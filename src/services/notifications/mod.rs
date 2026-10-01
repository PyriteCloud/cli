use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::{
        common::v1::Empty,
        notifications::v1::{
            Notification, NotificationById, Notifications,
            notification_service_client::NotificationServiceClient,
        },
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct NotificationsService;

impl NotificationsService {
    async fn client() -> Result<NotificationServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(NotificationServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn list() -> Result<Notifications, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(Empty {}, AuthService::get_metadata().await?);
        client
            .find_all_notifications(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get(id: String) -> Result<Notification, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            NotificationById { id },
            AuthService::get_metadata().await?,
        );
        client
            .find_one_notification(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
