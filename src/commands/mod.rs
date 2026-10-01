pub mod auth;
pub mod billing;
pub mod common;
pub mod deploy;
pub mod docker;
pub mod environments;
pub mod misc;
pub mod notifications;
pub mod projects;
pub mod service_domains;
pub mod service_runtime;
pub mod services;
pub mod team_api_keys;
pub mod team_invitations;
pub mod team_members;
pub mod team_regions;
pub mod team_registries;
pub mod team_volumes;
pub mod teams;

use billing::BillingCommands;
use clap::{Parser, Subcommand};
use docker::DockerCommands;
use environments::EnvironmentsCommands;
use misc::MiscCommands;
use notifications::NotificationsCommands;
use projects::ProjectsCommands;
use services::ServicesCommands;
use teams::TeamsCommands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
    Login,
    Logout,
    Billing {
        #[command(subcommand)]
        billing_cmd: BillingCommands,
    },
    Notifications {
        #[command(subcommand)]
        notifications_cmd: NotificationsCommands,
    },
    Misc {
        #[command(subcommand)]
        misc_cmd: MiscCommands,
    },
    Docker {
        #[command(subcommand)]
        docker_cmd: DockerCommands,
    },
    Teams {
        #[command(subcommand)]
        teams_cmd: TeamsCommands,
    },
    Projects {
        #[command(subcommand)]
        projects_cmd: ProjectsCommands,
    },
    Services {
        #[command(subcommand)]
        services_cmd: ServicesCommands,
    },
    Environments {
        #[command(subcommand)]
        environments_cmd: EnvironmentsCommands,
    },
    Deploy {
        #[arg(
            short,
            help = "Path to the pyrite.toml file",
            default_value = Some("pyrite.toml")
        )]
        file: Option<String>,
    },
}
