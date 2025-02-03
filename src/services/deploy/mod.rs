use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::services::v1::{
        deployments::v1::{
            DeploymentPortList, DeploymentRegionList, DeploymentVolumeList, DockerDeploymentDto,
        },
        services_service_client::ServicesServiceClient,
        upsert_service_dto::DeploymentConfig,
        Service, UpsertServiceDto,
    },
};
use tonic::{transport::channel::Channel, Request};

use crate::{utils::PYRITE_API_BASE_URL, PyriteJson};

use super::AuthService;

#[derive(Debug, Clone)]
pub(crate) struct DeployService;

impl DeployService {
    pub async fn get_services_client(
    ) -> Result<ServicesServiceClient<Channel>, Box<dyn std::error::Error>> {
        let client = ServicesServiceClient::connect(PYRITE_API_BASE_URL).await?;
        Ok(client)
    }

    pub async fn deploy(pyrite_json: PyriteJson) -> Result<Service, Box<dyn std::error::Error>> {
        let mut client = Self::get_services_client().await?;
        let metadata = AuthService::get_metadata().await?;

        let service = pyrite_json.services.first().unwrap();
        let message = UpsertServiceDto {
            name: service.name.to_owned(),
            r#type: service.r#type.to_owned(),
            project_id: pyrite_json.project_id,

            deployment_config: Some(DeploymentConfig::DockerConfig(DockerDeploymentDto {
                image: service.image.to_owned(),
                registry_id: service.registry_id.to_owned(),
                command: service.command.to_owned(),
                args: service.args.to_owned().map(|args| args.join(" ")),
                runtime: service.runtime.to_owned(),
                is_private: service.is_private.to_owned(),
                is_privileged: service.is_privileged.to_owned(),
                plan: service.plan.to_owned(),
                with_project_env: service.with_project_env.to_owned(),
                env: service
                    .env
                    .to_owned()
                    .map(|env| pbjson_types::Struct { fields: env }),
                ports_list: service
                    .ports
                    .to_owned()
                    .map(|ports| DeploymentPortList { ports }),
                regions_list: service
                    .regions
                    .to_owned()
                    .map(|regions| DeploymentRegionList { regions }),
                volumes_list: service
                    .volumes
                    .to_owned()
                    .map(|volumes| DeploymentVolumeList { volumes }),
                status: 0,
            })),
        };

        let req: Request<UpsertServiceDto> = ReqWithMetadata::with_metadata(message, metadata);

        client
            .upsert_service(req)
            .await
            .map(|res| res.into_inner())
            .map_err(|err| err.message().into())
    }
}
