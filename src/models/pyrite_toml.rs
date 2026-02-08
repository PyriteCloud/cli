use std::collections::HashMap;

use pbjson_types::Value;
use pyrite_client_rs::pyrite::v1::services::v1::deployments::v1::{
    DeploymentFileDto, DeploymentPortDto, DeploymentRegionDto, DeploymentVolumeDto,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct PyriteToml {
    pub project_id: String,
    pub services: Vec<TomlService>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TomlService {
    pub name: String,
    pub environment: String,
    pub r#type: String,
    pub image: String,
    pub plan: String,
    pub runtime: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub regions: Option<Vec<DeploymentRegionDto>>,
    pub ports: Option<Vec<DeploymentPortDto>>,
    pub volumes: Option<Vec<DeploymentVolumeDto>>,
    pub files: Option<Vec<DeploymentFileDto>>,
    pub env: Option<HashMap<String, Value>>,
    pub with_project_env: Option<bool>,
    pub registry_id: Option<String>,
    pub is_private: Option<bool>,
    pub is_privileged: Option<bool>,
}
