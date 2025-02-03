use clap::Parser;
use cliclack::set_theme;
use commands::{auth::AuthCommands, deploy::DeployCommands, Cli, Commands};
use models::pyrite_json::PyriteJson;
use utils::PyriteTheme;

pub mod commands;
pub mod models;
mod services;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_theme(PyriteTheme);

    let args = Cli::parse();

    match args.cmd {
        Commands::Login => AuthCommands::login().await?,
        Commands::Logout => AuthCommands::logout().await?,
        Commands::Docker { docker_cmd } => docker_cmd.run().await?,
        Commands::Teams { teams_cmd } => teams_cmd.run().await?,
        Commands::Projects { projects_cmd } => projects_cmd.run().await?,
        Commands::Deploy { file } => DeployCommands::run(file).await?,
    }

    Ok(())
}
