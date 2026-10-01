use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input, Select};
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::teams::v1::{CreateTeamDto, Team, UpdateTeamDto};

use crate::commands::common::select_team;
use crate::services::{ListQuery, MiscService, TeamsService};
use crate::utils::TABLE_DATE_FORMAT;

use super::team_api_keys::TeamApiKeysCommands;
use super::team_invitations::TeamInvitationsCommands;
use super::team_members::TeamMembersCommands;
use super::team_regions::TeamRegionsCommands;
use super::team_registries::TeamRegistriesCommands;
use super::team_volumes::TeamVolumesCommands;

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
    #[command(about = "Create team", visible_alias = "c")]
    Create {
        #[arg(short, long, help = "Team name")]
        name: Option<String>,
        #[arg(short, long, help = "Team subscription")]
        subscription: Option<String>,
    },
    #[command(about = "Update team", visible_alias = "u")]
    Update {
        #[arg(short, long, help = "Team id")]
        team_id: Option<String>,
        #[arg(short, long, help = "Team name")]
        name: Option<String>,
        #[arg(short, long, help = "Team subscription")]
        subscription: Option<String>,
    },
    #[command(about = "Delete team", visible_alias = "d")]
    Delete {
        #[arg(short, long, help = "Team id")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Manage team API keys")]
    ApiKeys {
        #[command(subcommand)]
        api_keys_cmd: TeamApiKeysCommands,
    },
    #[command(about = "Manage team invitations")]
    Invitations {
        #[command(subcommand)]
        invitations_cmd: TeamInvitationsCommands,
    },
    #[command(about = "Manage team members")]
    Members {
        #[command(subcommand)]
        members_cmd: TeamMembersCommands,
    },
    #[command(about = "List team regions")]
    Regions {
        #[command(subcommand)]
        regions_cmd: TeamRegionsCommands,
    },
    #[command(about = "Manage team registries")]
    Registries {
        #[command(subcommand)]
        registries_cmd: TeamRegistriesCommands,
    },
    #[command(about = "Manage team volumes")]
    Volumes {
        #[command(subcommand)]
        volumes_cmd: TeamVolumesCommands,
    },
}

impl TeamsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            TeamsCommands::List => {
                let teams_res = TeamsService::list_teams(ListQuery::Browse(None)).await?;
                let table = Self::get_teams_table(teams_res.teams)?;
                println!("{table}");
            }
            TeamsCommands::Get { team_id } => {
                let team = TeamsService::get_team(team_id).await?;
                let table = Self::get_teams_table(vec![team])?;
                println!("{table}");
            }
            TeamsCommands::Create { name, subscription } => {
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Team name")
                        .validate(|input: &String| {
                            if input.trim().is_empty() {
                                Err("Team name is required")
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?,
                };

                let subscription = match subscription {
                    Some(subscription) => subscription,
                    None => {
                        let subscriptions = MiscService::list_team_subscriptions().await?;
                        let options = subscriptions
                            .items
                            .into_iter()
                            .map(|subscription| {
                                let value = subscription.name.clone();
                                let hint = format!("${:.2}", subscription.price);
                                (value, subscription.name, hint)
                            })
                            .collect::<Vec<_>>();

                        Select::new("Team subscription")
                            .items(&options)
                            .interact()?
                    }
                };

                let team = TeamsService::create_team(CreateTeamDto { name, subscription }).await?;
                let table = Self::get_teams_table(vec![team])?;
                println!("{table}");
            }
            TeamsCommands::Update {
                team_id,
                name,
                subscription,
            } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let current_team = if name.is_none() || subscription.is_none() {
                    Some(TeamsService::get_team(team_id.clone()).await?)
                } else {
                    None
                };

                let name = match name {
                    Some(name) => name,
                    None => {
                        let current_team = current_team
                            .as_ref()
                            .ok_or("Current team data is unavailable")?;
                        Input::new("Team name")
                            .default_input(&current_team.name)
                            .validate(|input: &String| {
                                if input.trim().is_empty() {
                                    Err("Team name is required")
                                } else {
                                    Ok(())
                                }
                            })
                            .interact()?
                    }
                };

                let subscription = match subscription {
                    Some(subscription) => subscription,
                    None => {
                        let current_team = current_team
                            .as_ref()
                            .ok_or("Current team data is unavailable")?;
                        let subscriptions = MiscService::list_team_subscriptions().await?;
                        let options = subscriptions
                            .items
                            .into_iter()
                            .map(|subscription| {
                                let value = subscription.name.clone();
                                let hint = format!("${:.2}", subscription.price);
                                (value, subscription.name, hint)
                            })
                            .collect::<Vec<_>>();

                        Select::new("Team subscription")
                            .items(&options)
                            .initial_value(current_team.subscription.clone())
                            .interact()?
                    }
                };

                let team = TeamsService::update_team(UpdateTeamDto {
                    id: team_id,
                    name: Some(name),
                    subscription: Some(subscription),
                })
                .await?;
                let table = Self::get_teams_table(vec![team])?;
                println!("{table}");
            }
            TeamsCommands::Delete { team_id, yes } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };

                let confirmed = yes
                    || Confirm::new(format!("Delete team {team_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Team deletion cancelled")?;
                    return Ok(());
                }

                let team = TeamsService::delete_team(team_id).await?;
                let table = Self::get_teams_table(vec![team])?;
                println!("{table}");
            }
            TeamsCommands::ApiKeys { api_keys_cmd } => api_keys_cmd.run().await?,
            TeamsCommands::Invitations { invitations_cmd } => invitations_cmd.run().await?,
            TeamsCommands::Members { members_cmd } => members_cmd.run().await?,
            TeamsCommands::Regions { regions_cmd } => regions_cmd.run().await?,
            TeamsCommands::Registries { registries_cmd } => registries_cmd.run().await?,
            TeamsCommands::Volumes { volumes_cmd } => volumes_cmd.run().await?,
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
