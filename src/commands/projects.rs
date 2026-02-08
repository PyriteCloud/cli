use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::Select;
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::projects::v1::Project;

use crate::services::ProjectsService;
use crate::services::TeamsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
#[command(
    about = "Manage projects", 
    visible_aliases = ["p"],
    arg_required_else_help = false
)]
pub(crate) enum ProjectsCommands {
    #[command(about = "List all projects", visible_alias = "ls")]
    List {
        #[arg(short, long, help = "List projects by team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get project", visible_alias = "g")]
    Get {
        #[arg(short, long, help = "Get project by project id")]
        project_id: String,
    },
}

impl ProjectsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ProjectsCommands::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => Some(team_id),
                    None => Self::select_team().await?,
                };

                let projects_res = ProjectsService::list_projects(team_id).await?;
                let projects = projects_res.projects;
                if projects.is_empty() {
                    cliclack::outro("No projects found")?;
                } else {
                    let table = Self::get_projects_table(projects)?;
                    println!("{table}");
                }
            }
            ProjectsCommands::Get { project_id } => {
                let project = ProjectsService::get_project(project_id).await?;
                let table = Self::get_projects_table(vec![project])?;
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

    fn get_projects_table(projects: Vec<Project>) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();

        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Project Id",
                "Project Name",
                "Team Id",
                "Created At",
                "Updated At",
            ]);

        for project in projects {
            let created_at = DateTime::parse_from_rfc3339(project.created_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            let updated_at = DateTime::parse_from_rfc3339(project.updated_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            table.add_row(vec![
                Cell::new(project.id),
                Cell::new(project.name).fg(comfy_table::Color::White),
                Cell::new(project.team_id),
                Cell::new(created_at),
                Cell::new(updated_at),
            ]);
        }

        Ok(table)
    }
}
