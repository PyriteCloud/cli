use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::modifiers;
use comfy_table::presets;
use comfy_table::Table;
use pyrite_client_rs::pyrite::v1::teams::v1::Team;

use crate::services::TeamsService;

#[derive(Subcommand, Debug, Clone)]
#[command(about = "Manage teams", arg_required_else_help = false)]
pub(crate) enum TeamsCommands {
    #[command(about = "List all teams")]
    List,
    Get {
        team_id: String,
    },
}

impl TeamsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            TeamsCommands::List => {
                let teams_res = TeamsService::list_teams().await?;
                let table = Self::get_teams_table(teams_res.teams)?;
                println!("{table}");
            }
            TeamsCommands::Get { team_id } => {
                let team = TeamsService::get_team(team_id).await?;
                let table = Self::get_teams_table(vec![team])?;
                println!("{table}");
            }
        }
        Ok(())
    }

    fn get_teams_table(teams: Vec<Team>) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();

        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Team Id",
                "Name",
                "Created at",
                "Owner",
                "Subscription",
            ]);

        for team in teams {
            let owner = team.meta.map_or(team.owner, |meta| meta.owner_email);
            let created_at = DateTime::parse_from_rfc3339(team.created_at.as_str())?
                .with_timezone(&Local)
                .format("%d-%m-%Y %I:%M:%S %p %:z");

            table.add_row(vec![
                team.id,
                team.name,
                created_at.to_string(),
                owner,
                team.subscription,
            ]);
        }

        Ok(table)
    }
}
