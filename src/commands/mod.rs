pub mod auth;
pub mod deploy;
pub mod docker;
pub mod projects;
pub mod teams;

use clap::{Parser, Subcommand};
use docker::DockerCommands;
use projects::ProjectsCommands;
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
    Deploy {
        #[arg(
            short,
            help = "Path to the Pyrite file",
            default_value =Some("pyrite.json")
        )]
        file: Option<String>,
    },
}
