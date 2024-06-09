use clap::Parser;
use cliclack::set_theme;
use commands::{Cli, Commands};
use utils::PyriteTheme;

pub mod commands;
pub mod models;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_theme(PyriteTheme);

    let args = Cli::parse();

    match args.cmd {
        Commands::Docker { docker_cmd } => docker_cmd.run().await?,
    }

    Ok(())
}
