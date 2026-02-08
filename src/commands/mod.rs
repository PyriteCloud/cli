pub mod auth;
pub mod deploy;
pub mod docker;
pub mod environments;
pub mod projects;
pub mod services;
pub mod teams;

use clap::{Parser, Subcommand};
use docker::DockerCommands;
use environments::EnvironmentsCommands;
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
