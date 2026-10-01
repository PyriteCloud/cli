use clap::Subcommand;
use comfy_table::{Cell, Table, modifiers, presets};

use crate::commands::common::select_team;
use crate::services::TeamRegionsService;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamRegionsCommands {
    #[command(about = "List regions available to a team", visible_alias = "ls")]
    List {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
    },
}

impl TeamRegionsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let regions = TeamRegionsService::list(team_id).await?;
                if regions.public_regions.is_empty() && regions.private_regions.is_empty() {
                    cliclack::outro("No regions found")?;
                    return Ok(());
                }
                let mut table = Table::new();
                table
                    .load_preset(presets::UTF8_FULL)
                    .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
                    .set_header(vec!["Region", "Visibility"]);
                for region in regions.public_regions {
                    table.add_row(vec![Cell::new(region), Cell::new("Public")]);
                }
                for region in regions.private_regions {
                    table.add_row(vec![Cell::new(region), Cell::new("Private")]);
                }
                println!("{table}");
            }
        }
        Ok(())
    }
}
