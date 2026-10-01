use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::notifications::v1::Notification;

use crate::commands::common::select_value;
use crate::services::NotificationsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
#[command(about = "View notifications")]
pub(crate) enum NotificationsCommands {
    #[command(about = "List notifications", visible_alias = "ls")]
    List,
    #[command(about = "Get a notification", visible_alias = "g")]
    Get {
        #[arg(long, help = "Notification id")]
        notification_id: Option<String>,
    },
}

impl NotificationsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List => print_notifications(NotificationsService::list().await?.notifications)?,
            Self::Get { notification_id } => {
                let notification_id = match notification_id {
                    Some(notification_id) => notification_id,
                    None => {
                        let options = NotificationsService::list()
                            .await?
                            .notifications
                            .into_iter()
                            .map(|notification| {
                                (
                                    notification.id,
                                    notification.title.unwrap_or(notification.r#type),
                                    notification.scope,
                                )
                            })
                            .collect();
                        select_value("Select a notification", "No notifications found", options)?
                    }
                };
                print_notifications(vec![NotificationsService::get(notification_id).await?])?;
            }
        }
        Ok(())
    }
}

fn print_notifications(notifications: Vec<Notification>) -> Result<(), Box<dyn std::error::Error>> {
    if notifications.is_empty() {
        cliclack::outro("No notifications found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Notification Id",
            "Type",
            "Scope",
            "Title",
            "Message",
            "Created At",
            "Expires At",
        ]);
    for notification in notifications {
        let created_at = DateTime::parse_from_rfc3339(&notification.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let expires_at = DateTime::parse_from_rfc3339(&notification.expires_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(notification.id),
            Cell::new(notification.r#type),
            Cell::new(notification.scope),
            Cell::new(notification.title.unwrap_or_default()),
            Cell::new(notification.message.unwrap_or_default()),
            Cell::new(created_at),
            Cell::new(expires_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
