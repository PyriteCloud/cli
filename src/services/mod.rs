pub mod auth;
pub mod billing;
pub mod misc;
pub mod notifications;
pub mod projects;
pub mod service_domains;
pub mod service_environments;
pub mod service_logs;
pub mod service_metrics;
#[allow(clippy::module_inception)]
pub mod services;
pub mod team_api_keys;
pub mod team_invitations;
pub mod team_members;
pub mod team_regions;
pub mod team_registries;
pub mod team_volumes;
pub mod teams;
pub mod utils;

use pyrite_client_rs::pyrite::v1::common::v1::CursorPagination;

pub(crate) use auth::*;
pub(crate) use billing::*;
pub(crate) use misc::*;
pub(crate) use notifications::*;
pub(crate) use projects::*;
pub(crate) use service_domains::*;
pub(crate) use service_environments::*;
pub(crate) use service_logs::*;
pub(crate) use service_metrics::*;
pub(crate) use services::*;
pub(crate) use team_api_keys::*;
pub(crate) use team_invitations::*;
pub(crate) use team_members::*;
pub(crate) use team_regions::*;
pub(crate) use team_registries::*;
pub(crate) use team_volumes::*;
pub(crate) use teams::*;
pub(crate) use utils::*;

pub(crate) enum ListQuery {
    Browse(Option<CursorPagination>),
    Search(String),
}
