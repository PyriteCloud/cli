use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::teams::v1::{
    CreateTeamVolumeDto, TeamVolume, TeamVolumeMetrics, UpdateTeamVolumeDto,
};

use crate::commands::common::{select_team, select_value};
use crate::services::{TeamRegionsService, TeamVolumesService};
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamVolumesCommands {
    #[command(about = "List team volumes", visible_alias = "ls")]
    List {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
        #[arg(
            long,
            help = "Filter by service environment id",
            visible_alias = "env-id"
        )]
        service_environment_id: Option<String>,
    },
    #[command(about = "Get a team volume", visible_alias = "g")]
    Get {
        #[arg(long, help = "Volume id")]
        volume_id: Option<String>,
        #[arg(long, help = "Team id used to select a volume")]
        team_id: Option<String>,
    },
    #[command(about = "Create a team volume", visible_alias = "c")]
    Create {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
        #[arg(long, help = "Volume name")]
        name: Option<String>,
        #[arg(long, help = "Volume size")]
        size: Option<i32>,
        #[arg(long, help = "Volume region")]
        region: Option<String>,
    },
    #[command(about = "Update a team volume", visible_alias = "u")]
    Update {
        #[arg(long, help = "Volume id")]
        volume_id: Option<String>,
        #[arg(long, help = "Team id used to select a volume")]
        team_id: Option<String>,
        #[arg(long, help = "Volume size")]
        size: Option<i32>,
    },
    #[command(about = "Delete a team volume", visible_alias = "d")]
    Delete {
        #[arg(long, help = "Volume id")]
        volume_id: Option<String>,
        #[arg(long, help = "Team id used to select a volume")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Get volume metrics")]
    Metrics {
        #[arg(long, help = "Volume id")]
        volume_id: Option<String>,
        #[arg(long, help = "Team id used to select a volume")]
        team_id: Option<String>,
    },
}

impl TeamVolumesCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List {
                team_id,
                service_environment_id,
            } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                print_volumes(
                    TeamVolumesService::list(team_id, service_environment_id)
                        .await?
                        .team_volumes,
                )?;
            }
            Self::Get { volume_id, team_id } => {
                let volume_id = match volume_id {
                    Some(volume_id) => volume_id,
                    None => select_volume(team_id).await?,
                };
                print_volumes(vec![TeamVolumesService::get(volume_id).await?])?;
            }
            Self::Create {
                team_id,
                name,
                size,
                region,
            } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Volume name").interact()?,
                };
                let size = match size {
                    Some(size) => size,
                    None => Input::new("Volume size").interact()?,
                };
                let region = match region {
                    Some(region) => region,
                    None => select_region(&team_id).await?,
                };
                print_volumes(vec![
                    TeamVolumesService::create(CreateTeamVolumeDto {
                        team_id,
                        name,
                        size,
                        region,
                    })
                    .await?,
                ])?;
            }
            Self::Update {
                volume_id,
                team_id,
                size,
            } => {
                let volume_id = match volume_id {
                    Some(volume_id) => volume_id,
                    None => select_volume(team_id).await?,
                };
                let current = if size.is_none() {
                    Some(TeamVolumesService::get(volume_id.clone()).await?)
                } else {
                    None
                };
                let size = match size {
                    Some(size) => size,
                    None => {
                        let default_size = current
                            .as_ref()
                            .ok_or("Current volume data is unavailable")?
                            .size
                            .to_string();
                        Input::new("Volume size")
                            .default_input(&default_size)
                            .interact()?
                    }
                };
                print_volumes(vec![
                    TeamVolumesService::update(UpdateTeamVolumeDto {
                        id: volume_id,
                        size: Some(size),
                    })
                    .await?,
                ])?;
            }
            Self::Delete {
                volume_id,
                team_id,
                yes,
            } => {
                let volume_id = match volume_id {
                    Some(volume_id) => volume_id,
                    None => select_volume(team_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete volume {volume_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Volume deletion cancelled")?;
                    return Ok(());
                }
                print_volumes(vec![TeamVolumesService::delete(volume_id).await?])?;
            }
            Self::Metrics { volume_id, team_id } => {
                let volume_id = match volume_id {
                    Some(volume_id) => volume_id,
                    None => select_volume(team_id).await?,
                };
                print_metrics(TeamVolumesService::metrics(volume_id).await?);
            }
        }
        Ok(())
    }
}

async fn select_region(team_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let regions = TeamRegionsService::list(team_id.to_owned()).await?;
    let options = regions
        .public_regions
        .into_iter()
        .map(|region| (region.clone(), region, "Public".to_owned()))
        .chain(
            regions
                .private_regions
                .into_iter()
                .map(|region| (region.clone(), region, "Private".to_owned())),
        )
        .collect();
    select_value("Select a region", "No regions found", options)
}

async fn select_volume(team_id: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => select_team(None).await?.ok_or("A team must be selected")?,
    };
    let options = TeamVolumesService::list(team_id, None)
        .await?
        .team_volumes
        .into_iter()
        .map(|volume| {
            (
                volume.id,
                volume.name,
                format!("{} · {}", volume.region, volume.size),
            )
        })
        .collect();
    select_value("Select a volume", "No volumes found", options)
}

fn print_volumes(volumes: Vec<TeamVolume>) -> Result<(), Box<dyn std::error::Error>> {
    if volumes.is_empty() {
        cliclack::outro("No volumes found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Volume Id",
            "Name",
            "Size",
            "Region",
            "Environment Id",
            "Status",
            "Created At",
            "Updated At",
        ]);
    for volume in volumes {
        let created_at = DateTime::parse_from_rfc3339(&volume.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let updated_at = DateTime::parse_from_rfc3339(&volume.updated_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(volume.id),
            Cell::new(volume.name),
            Cell::new(volume.size),
            Cell::new(volume.region),
            Cell::new(volume.service_environment_id.unwrap_or_default()),
            Cell::new(volume.status),
            Cell::new(created_at),
            Cell::new(updated_at),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn print_metrics(metrics: TeamVolumeMetrics) {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Capacity",
            "Used",
            "Available",
            "Used %",
            "Available %",
        ])
        .add_row(vec![
            Cell::new(metrics.capacity),
            Cell::new(metrics.used),
            Cell::new(metrics.available),
            Cell::new(format!("{:.2}%", metrics.used_percentage)),
            Cell::new(format!("{:.2}%", metrics.available_percentage)),
        ]);
    println!("{table}");
}
