use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::Confirm;
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::teams::v1::{
    CreateTeamApiKeyDto, TeamApiKey, UpdateTeamApiKeyDto,
};

use crate::commands::common::{redact_secret, select_role, select_team, select_value};
use crate::services::TeamApiKeysService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamApiKeysCommands {
    #[command(about = "List team API keys", visible_alias = "ls")]
    List {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get a team API key", visible_alias = "g")]
    Get {
        #[arg(long, help = "API key")]
        key: Option<String>,
        #[arg(long, help = "Team id used to select an API key")]
        team_id: Option<String>,
    },
    #[command(about = "Create a team API key", visible_alias = "c")]
    Create {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
        #[arg(long, help = "API key role")]
        role: Option<String>,
    },
    #[command(about = "Update a team API key", visible_alias = "u")]
    Update {
        #[arg(long, help = "API key")]
        key: Option<String>,
        #[arg(long, help = "Team id used to select an API key")]
        team_id: Option<String>,
        #[arg(long, help = "API key role")]
        role: Option<String>,
    },
    #[command(about = "Delete a team API key", visible_alias = "d")]
    Delete {
        #[arg(long, help = "API key")]
        key: Option<String>,
        #[arg(long, help = "Team id used to select an API key")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl TeamApiKeysCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                print_keys(TeamApiKeysService::list(team_id).await?.team_api_keys)?;
            }
            Self::Get { key, team_id } => {
                let key = match key {
                    Some(key) => key,
                    None => select_key(team_id).await?,
                };
                let api_key = TeamApiKeysService::get(key).await?;
                cliclack::note("API key", &api_key.key)?;
                print_keys(vec![api_key])?;
            }
            Self::Create { team_id, role } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let role = match role {
                    Some(role) => role,
                    None => select_role(None).await?,
                };
                let api_key =
                    TeamApiKeysService::create(CreateTeamApiKeyDto { team_id, role }).await?;
                cliclack::note("API key", &api_key.key)?;
                print_keys(vec![api_key])?;
            }
            Self::Update { key, team_id, role } => {
                let key = match key {
                    Some(key) => key,
                    None => select_key(team_id).await?,
                };
                let current = if role.is_none() {
                    Some(TeamApiKeysService::get(key.clone()).await?)
                } else {
                    None
                };
                let role = match role {
                    Some(role) => role,
                    None => select_role(current.map(|api_key| api_key.role)).await?,
                };
                print_keys(vec![
                    TeamApiKeysService::update(UpdateTeamApiKeyDto { key, role }).await?,
                ])?;
            }
            Self::Delete { key, team_id, yes } => {
                let key = match key {
                    Some(key) => key,
                    None => select_key(team_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete API key {}?", redact_secret(&key)))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("API key deletion cancelled")?;
                    return Ok(());
                }
                print_keys(vec![TeamApiKeysService::delete(key).await?])?;
            }
        }
        Ok(())
    }
}

async fn select_key(team_id: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => select_team(None).await?.ok_or("A team must be selected")?,
    };
    let options = TeamApiKeysService::list(team_id)
        .await?
        .team_api_keys
        .into_iter()
        .map(|api_key| {
            let label = api_key.name.unwrap_or_else(|| redact_secret(&api_key.key));
            (api_key.key, label, api_key.role)
        })
        .collect();
    select_value("Select an API key", "No API keys found", options)
}

fn print_keys(keys: Vec<TeamApiKey>) -> Result<(), Box<dyn std::error::Error>> {
    if keys.is_empty() {
        cliclack::outro("No API keys found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Name",
            "Key",
            "Role",
            "Team Id",
            "Created By",
            "Created At",
        ]);
    for key in keys {
        let created_at = DateTime::parse_from_rfc3339(&key.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let displayed_key = redact_secret(&key.key);
        let created_by = key
            .meta
            .map_or(key.created_by, |meta| meta.created_by_email);
        table.add_row(vec![
            Cell::new(key.name.unwrap_or_default()),
            Cell::new(displayed_key),
            Cell::new(key.role),
            Cell::new(key.team_id),
            Cell::new(created_by),
            Cell::new(created_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
