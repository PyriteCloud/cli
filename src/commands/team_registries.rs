use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input, Password};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::teams::v1::{
    CreateTeamRegistryDto, TeamRegistry, UpdateTeamRegistryDto,
};

use crate::commands::common::{redact_secret, select_team, select_value};
use crate::services::TeamRegistriesService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamRegistriesCommands {
    #[command(about = "List team registries", visible_alias = "ls")]
    List {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get a team registry", visible_alias = "g")]
    Get {
        #[arg(long, help = "Registry id")]
        registry_id: Option<String>,
        #[arg(long, help = "Team id used to select a registry")]
        team_id: Option<String>,
    },
    #[command(about = "Create a team registry", visible_alias = "c")]
    Create {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
        #[arg(long, help = "Registry name")]
        name: Option<String>,
        #[arg(long, help = "Registry URL")]
        url: Option<String>,
        #[arg(long, help = "Registry username")]
        username: Option<String>,
        #[arg(long, help = "Registry password")]
        password: Option<String>,
    },
    #[command(about = "Update a team registry", visible_alias = "u")]
    Update {
        #[arg(long, help = "Registry id")]
        registry_id: Option<String>,
        #[arg(long, help = "Team id used to select a registry")]
        team_id: Option<String>,
        #[arg(long, help = "Registry name")]
        name: Option<String>,
        #[arg(long, help = "Registry URL")]
        url: Option<String>,
        #[arg(long, help = "Registry username")]
        username: Option<String>,
        #[arg(long, help = "Registry password")]
        password: Option<String>,
    },
    #[command(about = "Delete a team registry", visible_alias = "d")]
    Delete {
        #[arg(long, help = "Registry id")]
        registry_id: Option<String>,
        #[arg(long, help = "Team id used to select a registry")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl TeamRegistriesCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                print_registries(TeamRegistriesService::list(team_id).await?.team_registries)?;
            }
            Self::Get {
                registry_id,
                team_id,
            } => {
                let registry_id = match registry_id {
                    Some(registry_id) => registry_id,
                    None => select_registry(team_id).await?,
                };
                print_registries(vec![TeamRegistriesService::get(registry_id).await?])?;
            }
            Self::Create {
                team_id,
                name,
                url,
                username,
                password,
            } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Registry name").interact()?,
                };
                let url = match url {
                    Some(url) => url,
                    None => Input::new("Registry URL").interact()?,
                };
                let username = match username {
                    Some(username) => username,
                    None => Input::new("Registry username").interact()?,
                };
                let password = match password {
                    Some(password) => password,
                    None => Password::new("Registry password").interact()?,
                };
                print_registries(vec![
                    TeamRegistriesService::create(CreateTeamRegistryDto {
                        team_id,
                        name,
                        url,
                        username,
                        password,
                    })
                    .await?,
                ])?;
            }
            Self::Update {
                registry_id,
                team_id,
                name,
                url,
                username,
                password,
            } => {
                let registry_id = match registry_id {
                    Some(registry_id) => registry_id,
                    None => select_registry(team_id).await?,
                };
                let current = if name.is_none() || url.is_none() || username.is_none() {
                    Some(TeamRegistriesService::get(registry_id.clone()).await?)
                } else {
                    None
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Registry name")
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current registry data is unavailable")?
                                .name,
                        )
                        .interact()?,
                };
                let url = match url {
                    Some(url) => url,
                    None => Input::new("Registry URL")
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current registry data is unavailable")?
                                .url,
                        )
                        .interact()?,
                };
                let username = match username {
                    Some(username) => username,
                    None => Input::new("Registry username")
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current registry data is unavailable")?
                                .username,
                        )
                        .interact()?,
                };
                let password = match password {
                    Some(password) => Some(password),
                    None => {
                        let password = Password::new("New password (leave empty to keep current)")
                            .allow_empty()
                            .interact()?;
                        (!password.is_empty()).then_some(password)
                    }
                };
                print_registries(vec![
                    TeamRegistriesService::update(UpdateTeamRegistryDto {
                        id: registry_id,
                        name: Some(name),
                        url: Some(url),
                        username: Some(username),
                        password,
                    })
                    .await?,
                ])?;
            }
            Self::Delete {
                registry_id,
                team_id,
                yes,
            } => {
                let registry_id = match registry_id {
                    Some(registry_id) => registry_id,
                    None => select_registry(team_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete registry {registry_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Registry deletion cancelled")?;
                    return Ok(());
                }
                print_registries(vec![TeamRegistriesService::delete(registry_id).await?])?;
            }
        }
        Ok(())
    }
}

async fn select_registry(team_id: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => select_team(None).await?.ok_or("A team must be selected")?,
    };
    let options = TeamRegistriesService::list(team_id)
        .await?
        .team_registries
        .into_iter()
        .map(|registry| (registry.id, registry.name, registry.url))
        .collect();
    select_value("Select a registry", "No registries found", options)
}

fn print_registries(registries: Vec<TeamRegistry>) -> Result<(), Box<dyn std::error::Error>> {
    if registries.is_empty() {
        cliclack::outro("No registries found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Registry Id",
            "Name",
            "URL",
            "Username",
            "Password",
            "Created At",
            "Updated At",
        ]);
    for registry in registries {
        let created_at = DateTime::parse_from_rfc3339(&registry.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let updated_at = DateTime::parse_from_rfc3339(&registry.updated_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(registry.id),
            Cell::new(registry.name),
            Cell::new(registry.url),
            Cell::new(registry.username),
            Cell::new(redact_secret(&registry.password)),
            Cell::new(created_at),
            Cell::new(updated_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
