use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::teams::v1::{CreateTeamInvitationDto, TeamInvitation};

use crate::commands::common::{select_role, select_team, select_value};
use crate::services::TeamInvitationsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum TeamInvitationsCommands {
    #[command(
        about = "List invitations received by the current user",
        visible_alias = "ls"
    )]
    List,
    #[command(about = "List invitations sent by a team")]
    Sent {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get an invitation", visible_alias = "g")]
    Get {
        #[arg(long, help = "Invitation id")]
        invitation_id: Option<String>,
    },
    #[command(about = "Create an invitation", visible_alias = "c")]
    Create {
        #[arg(long, help = "Team id")]
        team_id: Option<String>,
        #[arg(long, help = "Invitee email")]
        email: Option<String>,
        #[arg(long, help = "Team role")]
        role: Option<String>,
    },
    #[command(about = "Delete a sent invitation", visible_alias = "d")]
    Delete {
        #[arg(long, help = "Invitation id")]
        invitation_id: Option<String>,
        #[arg(long, help = "Team id used to select an invitation")]
        team_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Accept an invitation")]
    Accept {
        #[arg(long, help = "Invitation id")]
        invitation_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl TeamInvitationsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List => print_invitations(TeamInvitationsService::list().await?.invitations)?,
            Self::Sent { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                print_invitations(
                    TeamInvitationsService::list_sent(team_id)
                        .await?
                        .invitations,
                )?;
            }
            Self::Get { invitation_id } => {
                let invitation_id = match invitation_id {
                    Some(invitation_id) => invitation_id,
                    None => select_received_invitation().await?,
                };
                print_invitations(vec![TeamInvitationsService::get(invitation_id).await?])?;
            }
            Self::Create {
                team_id,
                email,
                role,
            } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let email = match email {
                    Some(email) => email,
                    None => Input::new("Invitee email")
                        .validate(|input: &String| {
                            if input.trim().is_empty() {
                                Err("Email is required")
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?,
                };
                let role = match role {
                    Some(role) => role,
                    None => select_role(None).await?,
                };
                print_invitations(vec![
                    TeamInvitationsService::create(CreateTeamInvitationDto {
                        team_id,
                        email,
                        role,
                    })
                    .await?,
                ])?;
            }
            Self::Delete {
                invitation_id,
                team_id,
                yes,
            } => {
                let invitation_id = match invitation_id {
                    Some(invitation_id) => invitation_id,
                    None => select_sent_invitation(team_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete invitation {invitation_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Invitation deletion cancelled")?;
                    return Ok(());
                }
                print_invitations(vec![TeamInvitationsService::delete(invitation_id).await?])?;
            }
            Self::Accept { invitation_id, yes } => {
                let invitation_id = match invitation_id {
                    Some(invitation_id) => invitation_id,
                    None => select_received_invitation().await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Accept invitation {invitation_id}?"))
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Invitation acceptance cancelled")?;
                    return Ok(());
                }
                print_invitations(vec![TeamInvitationsService::accept(invitation_id).await?])?;
            }
        }
        Ok(())
    }
}

async fn select_received_invitation() -> Result<String, Box<dyn std::error::Error>> {
    let options = TeamInvitationsService::list()
        .await?
        .invitations
        .into_iter()
        .map(|invitation| {
            let team = invitation
                .meta
                .and_then(|meta| meta.team)
                .map(|team| team.name)
                .unwrap_or_else(|| invitation.team_id.clone());
            (invitation.id, team, invitation.role)
        })
        .collect();
    select_value("Select an invitation", "No invitations found", options)
}

async fn select_sent_invitation(
    team_id: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let team_id = match team_id {
        Some(team_id) => team_id,
        None => select_team(None).await?.ok_or("A team must be selected")?,
    };
    let options = TeamInvitationsService::list_sent(team_id)
        .await?
        .invitations
        .into_iter()
        .map(|invitation| (invitation.id, invitation.email, invitation.role))
        .collect();
    select_value("Select an invitation", "No sent invitations found", options)
}

fn print_invitations(invitations: Vec<TeamInvitation>) -> Result<(), Box<dyn std::error::Error>> {
    if invitations.is_empty() {
        cliclack::outro("No invitations found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Invitation Id",
            "Team",
            "Email",
            "Role",
            "Created At",
            "Expires At",
        ]);
    for invitation in invitations {
        let team = invitation
            .meta
            .and_then(|meta| meta.team)
            .map(|team| team.name)
            .unwrap_or_else(|| invitation.team_id.clone());
        let created_at = DateTime::parse_from_rfc3339(&invitation.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let expires_at = DateTime::parse_from_rfc3339(&invitation.expires_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(invitation.id),
            Cell::new(team),
            Cell::new(invitation.email),
            Cell::new(invitation.role),
            Cell::new(created_at),
            Cell::new(expires_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
