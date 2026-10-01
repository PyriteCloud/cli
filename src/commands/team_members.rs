use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::Confirm;
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::teams::v1::{TeamMember, UpdateTeamMemberDto};

use crate::commands::common::{select_role, select_team, select_value};
use crate::services::TeamMembersService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamMembersCommands {
    #[command(about = "List team members", visible_alias = "ls")]
    List {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get a team member", visible_alias = "g")]
    Get {
        #[arg(long, help = "Team member id")]
        member_id: Option<String>,
        #[arg(long, help = "Team id used to select a member")]
        team_id: Option<String>,
    },
    #[command(about = "Update a team member", visible_alias = "u")]
    Update {
        #[arg(long, help = "Team member id")]
        member_id: Option<String>,
        #[arg(long, help = "Team id used to select a member")]
        team_id: Option<String>,
        #[arg(long, help = "Team role")]
        role: Option<String>,
    },
    #[command(about = "Delete a team member", visible_alias = "d")]
    Delete {
        #[arg(long, help = "Team member id")]
        member_id: Option<String>,
        #[arg(long, help = "Team id used to select a member")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl TeamMembersCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                print_members(TeamMembersService::list(team_id).await?.team_members)?;
            }
            Self::Get { member_id, team_id } => {
                let member_id = match member_id {
                    Some(member_id) => member_id,
                    None => select_member(team_id).await?,
                };
                print_members(vec![TeamMembersService::get(member_id).await?])?;
            }
            Self::Update {
                member_id,
                team_id,
                role,
            } => {
                let member_id = match member_id {
                    Some(member_id) => member_id,
                    None => select_member(team_id).await?,
                };
                let current = if role.is_none() {
                    Some(TeamMembersService::get(member_id.clone()).await?)
                } else {
                    None
                };
                let role = match role {
                    Some(role) => role,
                    None => select_role(current.map(|member| member.role)).await?,
                };
                print_members(vec![
                    TeamMembersService::update(UpdateTeamMemberDto {
                        id: member_id,
                        role,
                    })
                    .await?,
                ])?;
            }
            Self::Delete {
                member_id,
                team_id,
                yes,
            } => {
                let member_id = match member_id {
                    Some(member_id) => member_id,
                    None => select_member(team_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete team member {member_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Team member deletion cancelled")?;
                    return Ok(());
                }
                print_members(vec![TeamMembersService::delete(member_id).await?])?;
            }
        }
        Ok(())
    }
}

async fn select_member(team_id: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => select_team(None).await?.ok_or("A team must be selected")?,
    };
    let options = TeamMembersService::list(team_id)
        .await?
        .team_members
        .into_iter()
        .map(|member| {
            let user = member.meta.and_then(|meta| meta.user);
            let label = user
                .as_ref()
                .map(|user| user.name.clone())
                .unwrap_or_else(|| member.user_id.clone());
            let hint = user.map(|user| user.email).unwrap_or(member.role.clone());
            (member.id, label, hint)
        })
        .collect();
    select_value("Select a team member", "No team members found", options)
}

fn print_members(members: Vec<TeamMember>) -> Result<(), Box<dyn std::error::Error>> {
    if members.is_empty() {
        cliclack::outro("No team members found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Member Id",
            "Name",
            "Email",
            "Role",
            "Created At",
            "Updated At",
        ]);
    for member in members {
        let user = member.meta.and_then(|meta| meta.user);
        let name = user
            .as_ref()
            .map(|user| user.name.as_str())
            .unwrap_or_default();
        let email = user
            .as_ref()
            .map(|user| user.email.as_str())
            .unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&member.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let updated_at = DateTime::parse_from_rfc3339(&member.updated_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(member.id),
            Cell::new(name),
            Cell::new(email),
            Cell::new(member.role),
            Cell::new(created_at),
            Cell::new(updated_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
