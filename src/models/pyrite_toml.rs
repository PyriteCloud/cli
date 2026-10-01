use pyrite_client_rs::pyrite::v1::services::v1::deployments::v1::{
    DeploymentHealthCheckDto, DeploymentPortDto, DeploymentRegionDto, DeploymentVolumeDto,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Item {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PyriteToml {
    pub project_id: String,
    pub services: Vec<TomlService>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlImageSource {
    pub r#ref: String,
    pub registry_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlDockerBuildConfig {
    pub builder: String,
    pub context: String,
    pub dockerfile_path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlGitSource {
    pub repo_url: String,
    pub branch: String,
    pub sha: Option<String>,
    pub with_build: Option<bool>,
    pub build: Option<TomlDockerBuildConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlFile {
    pub mount_path: String,
    pub permission: String,
    pub content: Option<String>,
    pub content_from_file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlDockerConfig {
    pub source_type: String,
    pub image: Option<TomlImageSource>,
    pub git: Option<TomlGitSource>,
    pub runtime: String,
    pub command: Option<String>,
    pub args: Option<String>,
    pub regions: Option<Vec<DeploymentRegionDto>>,
    pub ports: Option<Vec<DeploymentPortDto>>,
    pub health_checks: Option<Vec<DeploymentHealthCheckDto>>,
    pub volumes: Option<Vec<DeploymentVolumeDto>>,
    pub files: Option<Vec<TomlFile>>,
    pub env: Option<Vec<Item>>,
    pub with_project_env: Option<bool>,
    pub is_private: Option<bool>,
    pub is_privileged: Option<bool>,
    pub plan: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlPostgresConfig {
    pub version: String,
    pub size: i32,
    pub region: String,
    pub password: String,
    pub plan: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlService {
    pub name: String,
    pub environment: String,
    pub r#type: String,
    pub docker: Option<TomlDockerConfig>,
    pub postgres: Option<TomlPostgresConfig>,
}
