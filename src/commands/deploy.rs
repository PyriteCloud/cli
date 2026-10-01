use std::{
    borrow::Cow,
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use comfy_table::{Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::services::v1::{
    UpsertServiceDto,
    common::v1::{Service, ServiceEnvironment},
    deployments::v1::{
        DeploymentFileDto, DeploymentFileList, DeploymentHealthCheckList, DeploymentPortList,
        DeploymentRegionList, DeploymentVolumeList, DockerDeploymentBuildConfigDto,
        DockerDeploymentDto, DockerDeploymentGitSourceDto, DockerDeploymentImageSourceDto,
        DockerDeploymentSourceDto, PostgresDeploymentDto,
    },
    upsert_service_dto::DeploymentConfig,
};

use crate::{
    models::pyrite_toml::{
        PyriteToml, TomlDockerConfig, TomlFile, TomlGitSource, TomlImageSource, TomlPostgresConfig,
        TomlService,
    },
    services::{ServicesService, UtilsService},
};

const DOCKER_SOURCE_IMAGE: &str = "image";
const DOCKER_SOURCE_GIT: &str = "git";
const DEPLOYMENT_FILE_MAX_SIZE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct DeployCommands;

impl DeployCommands {
    pub async fn run(file: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = file.unwrap();

        if !Path::new(&file_path).exists() {
            return Err(format!("File {} does not exist", file_path).into());
        }

        let file_path = Path::new(&file_path);
        let file_data = std::fs::read_to_string(file_path)?;
        let pyrite_json: PyriteToml = toml::from_str(&file_data)?;
        let config_dir = file_path.parent().unwrap_or_else(|| Path::new("."));

        let services = pyrite_json.services;

        for service in services {
            Self::deploy_service(&pyrite_json.project_id, &service, config_dir).await?;
        }

        Ok(())
    }

    async fn deploy_service(
        project_id: &str,
        service: &TomlService,
        config_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let upsert_service_dto =
            Self::get_upsert_service_dto_from_service(project_id.to_string(), service, config_dir)?;

        let res = UtilsService::with_progress(
            || async { ServicesService::upsert_service(upsert_service_dto).await },
            &format!("Deploying {}", service.name),
            &format!("Deployment of {} successful", service.name),
            &format!("Deployment of {} failed", service.name),
        )
        .await?;

        let table =
            Self::get_service_details_table(res.service.unwrap(), res.service_environment.unwrap())
                .await?;

        println!("{table}");
        Ok(())
    }

    async fn get_service_details_table(
        service: Service,
        service_environment: ServiceEnvironment,
    ) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();
        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Service Id",
                "Team Name",
                "Project Name",
                "Service Name",
                "Environment Name",
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
            service_environment.name,
        ]);

        Ok(table)
    }

    fn get_upsert_service_dto_from_service(
        project_id: String,
        service: &TomlService,
        config_dir: &Path,
    ) -> Result<UpsertServiceDto, Box<dyn std::error::Error>> {
        let deployment_config = match (&service.docker, &service.postgres) {
            (Some(docker), None) => DeploymentConfig::DockerConfig(
                Self::get_docker_deployment_dto_from_config(&service.name, docker, config_dir)?,
            ),
            (None, Some(postgres)) => DeploymentConfig::PostgresConfig(
                Self::get_postgres_deployment_dto_from_config(postgres),
            ),
            _ => {
                return Err(format!(
                    "Service '{}' must define exactly one of 'docker' or 'postgres'",
                    service.name
                )
                .into());
            }
        };

        Ok(UpsertServiceDto {
            name: service.name.to_owned(),
            environment: Some(service.environment.to_owned()),
            r#type: service.r#type.to_owned(),
            project_id,
            deployment_config: Some(deployment_config),
        })
    }

    fn get_docker_deployment_dto_from_config(
        service_name: &str,
        config: &TomlDockerConfig,
        config_dir: &Path,
    ) -> Result<DockerDeploymentDto, Box<dyn std::error::Error>> {
        let source = Self::get_docker_source_dto(service_name, config)?;
        let env = config
            .env
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .map(|item| (item.name.as_str(), item.value.as_str()))
                    .collect::<HashMap<&str, &str>>()
            })
            .map(|env| serde_json::to_string(&env))
            .transpose()?
            .map(|env| BASE64_STANDARD.encode(env));

        Ok(DockerDeploymentDto {
            source_type: config.source_type.to_owned(),
            source: Some(source),
            command: config.command.to_owned(),
            args: config.args.to_owned(),
            runtime: config.runtime.to_owned(),
            is_private: config.is_private,
            is_privileged: config.is_privileged,
            plan: config.plan.to_owned(),
            with_project_env: config.with_project_env,
            env,
            ports_list: config
                .ports
                .to_owned()
                .map(|ports| DeploymentPortList { ports }),
            health_checks_list: config
                .health_checks
                .to_owned()
                .map(|health_checks| DeploymentHealthCheckList { health_checks }),
            regions_list: config
                .regions
                .to_owned()
                .map(|regions| DeploymentRegionList { regions }),
            volumes_list: config
                .volumes
                .to_owned()
                .map(|volumes| DeploymentVolumeList { volumes }),
            files_list: config
                .files
                .as_deref()
                .map(|files| Self::get_deployment_file_list(service_name, files, config_dir))
                .transpose()?,
        })
    }

    fn get_deployment_file_list(
        service_name: &str,
        files: &[TomlFile],
        config_dir: &Path,
    ) -> Result<DeploymentFileList, Box<dyn std::error::Error>> {
        let files = files
            .iter()
            .map(|file| Self::get_deployment_file_dto(service_name, file, config_dir))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DeploymentFileList { files })
    }

    fn get_deployment_file_dto(
        service_name: &str,
        file: &TomlFile,
        config_dir: &Path,
    ) -> Result<DeploymentFileDto, Box<dyn std::error::Error>> {
        let content = match (&file.content, &file.content_from_file) {
            (Some(content), None) => {
                Self::validate_deployment_file_size(
                    service_name,
                    &file.mount_path,
                    content.as_bytes(),
                )?;
                Cow::Borrowed(content.as_str())
            }
            (None, Some(content_from_file)) => {
                let content_path = Self::resolve_content_path(config_dir, content_from_file)
                    .map_err(|error| {
                        format!(
                            "Invalid content_from_file '{}' for deployment file '{}' in service '{}': {}",
                            content_from_file, file.mount_path, service_name, error
                        )
                    })?;
                Cow::Owned(Self::read_deployment_file_content(
                    service_name,
                    &file.mount_path,
                    &content_path,
                )?)
            }
            _ => {
                return Err(format!(
                    "Deployment file '{}' for service '{}' must define exactly one of 'content' or 'content_from_file'",
                    file.mount_path, service_name
                )
                .into());
            }
        };

        Ok(DeploymentFileDto {
            mount_path: file.mount_path.to_owned(),
            content: BASE64_STANDARD.encode(content.as_bytes()),
            permissions: Some(file.permission.to_owned()),
        })
    }

    fn resolve_content_path(
        config_dir: &Path,
        content_from_file: &str,
    ) -> Result<PathBuf, std::io::Error> {
        let content_path = Path::new(content_from_file);

        if content_path.as_os_str().is_empty()
            || content_path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::RootDir | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "must be a nonempty relative path from the directory containing pyrite.toml",
            ));
        }

        Ok(config_dir.join(content_path))
    }

    fn read_deployment_file_content(
        service_name: &str,
        deployment_file_name: &str,
        content_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let source_file = std::fs::File::open(content_path).map_err(|error| {
            format!(
                "Failed to open content file '{}' for deployment file '{}' in service '{}': {}",
                content_path.display(),
                deployment_file_name,
                service_name,
                error
            )
        })?;
        let mut content = Vec::new();

        source_file
            .take((DEPLOYMENT_FILE_MAX_SIZE_BYTES + 1) as u64)
            .read_to_end(&mut content)
            .map_err(|error| {
                format!(
                    "Failed to read content file '{}' for deployment file '{}' in service '{}': {}",
                    content_path.display(),
                    deployment_file_name,
                    service_name,
                    error
                )
            })?;

        Self::validate_deployment_file_size(service_name, deployment_file_name, &content)?;

        String::from_utf8(content).map_err(|error| {
            format!(
                "Content file '{}' for deployment file '{}' in service '{}' must contain valid UTF-8: {}",
                content_path.display(),
                deployment_file_name,
                service_name,
                error
            )
            .into()
        })
    }

    fn validate_deployment_file_size(
        service_name: &str,
        deployment_file_path: &str,
        content: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if content.len() > DEPLOYMENT_FILE_MAX_SIZE_BYTES {
            return Err(format!(
                "Content for deployment file '{}' in service '{}' must not exceed 1 MiB ({} bytes)",
                deployment_file_path, service_name, DEPLOYMENT_FILE_MAX_SIZE_BYTES
            )
            .into());
        }

        Ok(())
    }

    fn get_docker_source_dto(
        service_name: &str,
        config: &TomlDockerConfig,
    ) -> Result<DockerDeploymentSourceDto, Box<dyn std::error::Error>> {
        match (
            config.source_type.as_str(),
            config.image.as_ref(),
            config.git.as_ref(),
        ) {
            (DOCKER_SOURCE_IMAGE, Some(image), None) => Ok(DockerDeploymentSourceDto {
                image: Some(Self::get_docker_image_source_dto(image)),
                git: None,
            }),
            (DOCKER_SOURCE_GIT, None, Some(git)) => Ok(DockerDeploymentSourceDto {
                image: None,
                git: Some(Self::get_docker_git_source_dto(git)),
            }),
            (DOCKER_SOURCE_IMAGE, _, _) => Err(format!(
                "Docker service '{service_name}' with source_type '{DOCKER_SOURCE_IMAGE}' must define only '{DOCKER_SOURCE_IMAGE}'"
            )
            .into()),
            (DOCKER_SOURCE_GIT, _, _) => Err(format!(
                "Docker service '{service_name}' with source_type '{DOCKER_SOURCE_GIT}' must define only '{DOCKER_SOURCE_GIT}'"
            )
            .into()),
            (source_type, _, _) => Err(format!(
                "Docker service '{service_name}' has unsupported source_type '{source_type}'"
            )
            .into()),
        }
    }

    fn get_docker_image_source_dto(image: &TomlImageSource) -> DockerDeploymentImageSourceDto {
        DockerDeploymentImageSourceDto {
            r#ref: image.r#ref.to_owned(),
            registry_id: image.registry_id.to_owned(),
        }
    }

    fn get_docker_git_source_dto(git: &TomlGitSource) -> DockerDeploymentGitSourceDto {
        DockerDeploymentGitSourceDto {
            repo_url: git.repo_url.to_owned(),
            branch: git.branch.to_owned(),
            sha: git.sha.to_owned(),
            with_build: git.with_build,
            build: git
                .build
                .as_ref()
                .map(|build| DockerDeploymentBuildConfigDto {
                    builder: build.builder.to_owned(),
                    context: Some(build.context.to_owned()),
                    dockerfile_path: Some(build.dockerfile_path.to_owned()),
                }),
        }
    }

    fn get_postgres_deployment_dto_from_config(
        config: &TomlPostgresConfig,
    ) -> PostgresDeploymentDto {
        PostgresDeploymentDto {
            size: config.size,
            version: config.version.to_owned(),
            region: config.region.to_owned(),
            plan: config.plan.to_owned(),
            password: config.password.to_owned(),
        }
    }
}
