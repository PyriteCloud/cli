use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::teams::v1::Team;

use crate::services::TeamsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
#[command(
    about = "Manage teams",
    visible_aliases = ["t"],
    arg_required_else_help = false
)]
pub(crate) enum TeamsCommands {
    #[command(about = "List all teams", visible_alias = "ls")]
    List,
    #[command(about = "Get team", visible_alias = "g")]
    Get {
        #[arg(short, long, help = "Get team by team id")]
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
                "Team Name",
                "Subscription",
                "Owner",
                "Created At",
                "Updated At",
            ]);

        for team in teams {
            let owner = team.meta.map_or(team.owner, |meta| meta.owner_email);
            let created_at = DateTime::parse_from_rfc3339(team.created_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            let updated_at = DateTime::parse_from_rfc3339(team.updated_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            table.add_row(vec![
                Cell::new(team.id),
                Cell::new(team.name).fg(comfy_table::Color::White),
                Cell::new(team.subscription.to_uppercase()).fg(comfy_table::Color::White),
                Cell::new(owner).fg(comfy_table::Color::White),
                Cell::new(created_at),
                Cell::new(updated_at),
            ]);
        }

        Ok(table)
    }
}
