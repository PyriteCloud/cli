use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::Select;
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::services::v1::common::v1::Service;

use crate::services::ProjectsService;
use crate::services::ServicesService;
use crate::services::TeamsService;
use crate::services::UtilsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
#[command(
    about = "Manage services",
    visible_aliases = ["s"],
    arg_required_else_help = false
)]
pub(crate) enum ServicesCommands {
    #[command(about = "List all services", visible_alias = "ls")]
    List {
        #[arg(short, long, help = "List services by team id")]
        team_id: Option<String>,
        #[arg(short, long, help = "List services by project id")]
        project_id: Option<String>,
    },
    #[command(about = "Get service", visible_alias = "g")]
    Get {
        #[arg(short, long, help = "Get service by service id")]
        service_id: String,
    },
}

impl ServicesCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ServicesCommands::List {
                team_id,
                project_id,
            } => {
                let team_id = match (&team_id, &project_id) {
                    (_, Some(_)) => None,
                    (Some(team_id), None) => Some(team_id.to_owned()),
                    (None, None) => Self::select_team().await?,
                };

                let project_id = match &project_id {
                    Some(project_id) => Some(project_id.to_owned()),
                    None => Self::select_project(&team_id).await?,
                };

                let services_res = ServicesService::list_services(team_id, project_id).await?;
                let services = services_res.services;
                if services.is_empty() {
                    cliclack::outro("No services found")?;
                } else {
                    let table = Self::get_services_table(services)?;
                    println!("{table}");
                }
            }
            ServicesCommands::Get { service_id } => {
                let project = ServicesService::get_service(service_id).await?;
                let table = Self::get_services_table(vec![project])?;
                println!("{table}");
            }
        }
        Ok(())
    }

    async fn select_team() -> Result<Option<String>, Box<dyn std::error::Error>> {
        let teams_res = TeamsService::list_teams().await?;
        let teams = teams_res.teams;

        if teams.is_empty() {
            return Err("No teams found".into());
        }

        let items = teams
            .iter()
            .map(|team| {
                (
                    team.id.to_owned(),
                    team.name.to_owned(),
                    team.meta
                        .as_ref()
                        .map_or(team.owner.to_owned(), |meta| meta.owner_email.to_owned()),
                )
            })
            .collect::<Vec<_>>();

        let res = Select::new("Select a team")
            .item("".to_owned(), "All".to_owned(), "")
            .items(items.as_slice())
            .interact()?;

        Ok(if !res.is_empty() { Some(res) } else { None })
    }

    async fn select_project(
        team_id: &Option<String>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let projects_res = ProjectsService::list_projects(team_id.clone()).await?;
        let projects = projects_res.projects;

        if projects.is_empty() {
            return Err("No projects found".into());
        }

        let items = projects
            .iter()
            .map(|project| {
                (
                    project.id.to_owned(),
                    project.name.to_owned(),
                    "".to_owned(),
                )
            })
            .collect::<Vec<_>>();

        let res = Select::new("Select a project")
            .item("".to_owned(), "All".to_owned(), "")
            .items(items.as_slice())
            .interact()?;

        Ok(if !res.is_empty() { Some(res) } else { None })
    }

    fn get_services_table(services: Vec<Service>) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();

        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Service Id",
                "Project Id",
                "Service Name",
                "Type",
                "Status",
                "Created At",
                "Updated At",
            ]);

        for service in services {
            let created_at = DateTime::parse_from_rfc3339(service.created_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            let updated_at = DateTime::parse_from_rfc3339(service.updated_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            table.add_row(vec![
                Cell::new(service.id),
                Cell::new(service.project_id),
                Cell::new(service.name).fg(comfy_table::Color::White),
                Cell::new(service.r#type.to_uppercase()),
                Cell::new(UtilsService::get_service_status_label(service.status))
                    .fg(UtilsService::get_service_status_color(service.status)),
                Cell::new(created_at),
                Cell::new(updated_at),
            ]);
        }

        Ok(table)
    }
}
