use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::modifiers;
use comfy_table::presets;
use comfy_table::Table;
use pyrite_client_rs::pyrite::v1::projects::v1::Project;

use crate::services::ProjectsService;

#[derive(Subcommand, Debug, Clone)]
#[command(about = "Manage projects", arg_required_else_help = false)]
pub(crate) enum ProjectsCommands {
    #[command(about = "List all projects")]
    List {
        #[arg(short, long, help = "List projects by team id")]
        team_id: Option<String>,
    },
    #[command(about = "Get project")]
    Get { project_id: String },
}

impl ProjectsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ProjectsCommands::List { team_id } => {
                let projects_res = ProjectsService::list_projects(team_id).await?;
                let projects = projects_res.projects;
                if projects.is_empty() {
                    println!("No projects found");
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

    fn get_projects_table(projects: Vec<Project>) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();

        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec!["Project Id", "Team Id", "Name", "Created at"]);

        for project in projects {
            let created_at = DateTime::parse_from_rfc3339(project.created_at.as_str())?
                .with_timezone(&Local)
                .format("%d-%m-%Y %I:%M:%S %p %:z");

            table.add_row(vec![
                project.id,
                project.team_id,
                project.name,
                created_at.to_string(),
            ]);
        }

        Ok(table)
    }
}
