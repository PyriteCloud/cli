pub mod docker;

use clap::{Parser, Subcommand};
use docker::DockerCommands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
    Docker {
        #[command(subcommand)]
        docker_cmd: DockerCommands,
    },
}
