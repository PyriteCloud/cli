use std::path::Path;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use comfy_table::{Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::services::v1::{
    UpsertServiceDto,
    common::v1::Service,
    deployments::v1::{
        DeploymentFileList, DeploymentPortList, DeploymentRegionList, DeploymentVolumeList,
        DockerDeploymentDto,
    },
    upsert_service_dto::DeploymentConfig,
};

use crate::{
    models::pyrite_toml::{PyriteToml, TomlService},
    services::ServicesService,
};

#[derive(Debug, Clone)]
pub(crate) struct DeployCommands;

impl DeployCommands {
    pub async fn run(file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = file.unwrap();

        if !Path::new(&file_path).exists() {
            return Err(format!("File {} does not exist", file_path).into());
        }

        let file_data = std::fs::read_to_string(file_path)?;
        let pyrite_json: PyriteToml = toml::from_str(&file_data)?;

        let service = pyrite_json.services.first().unwrap();
        let upsert_service_dto =
            Self::get_upsert_service_dto_from_service(pyrite_json.project_id, service);

        let service = ServicesService::upsert_service(upsert_service_dto).await?;

        let table = Self::get_service_table(service).await?;
        println!("{table}");

        Ok(())
    }

    async fn get_service_table(service: Service) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();
        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Service Id",
                "Team Name",
                "Project Name",
                "Service Name",
            ]);

        table.add_row(vec![
            service.id,
            service
                .meta
                .as_ref()
                .and_then(|meta| meta.team.as_ref())
                .map(|team| team.name.to_owned())
                .unwrap_or_default(),
            service
                .meta
                .as_ref()
                .and_then(|meta| meta.project.as_ref())
                .map(|project| project.name.to_owned())
                .unwrap_or_default(),
            service.name,
        ]);

        Ok(table)
    }

    fn get_upsert_service_dto_from_service(
        project_id: String,
        service: &TomlService,
    ) -> UpsertServiceDto {
        UpsertServiceDto {
            name: service.name.to_owned(),
            environment: Some(service.environment.to_owned()),
            r#type: service.r#type.to_owned(),
            project_id,

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
                    .map(|env| serde_json::to_string(&env).unwrap())
                    .map(|env_str| BASE64_STANDARD_NO_PAD.encode(env_str)),
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
                files_list: service
                    .files
                    .to_owned()
                    .map(|files| DeploymentFileList { files }),
            })),
        }
    }
}
