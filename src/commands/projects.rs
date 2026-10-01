use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input};
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::projects::v1::{CreateProjectDto, Project, UpdateProjectDto};

use crate::commands::common::{SELECT_ALL_TEAMS, select_project, select_team};
use crate::services::{ListQuery, ProjectsService};
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
    #[command(about = "Create project", visible_alias = "c")]
    Create {
        #[arg(short, long, help = "Team id")]
        team_id: Option<String>,
        #[arg(short, long, help = "Project name")]
        name: Option<String>,
        #[arg(short, long, help = "Project environment")]
        env: Option<String>,
    },
    #[command(about = "Update project", visible_alias = "u")]
    Update {
        #[arg(short, long, help = "Project id")]
        project_id: Option<String>,
        #[arg(short, long, help = "Project name")]
        name: Option<String>,
        #[arg(short, long, help = "Project environment")]
        env: Option<String>,
    },
    #[command(about = "Delete project", visible_alias = "d")]
    Delete {
        #[arg(short, long, help = "Project id")]
        project_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl ProjectsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ProjectsCommands::List { team_id } => {
                let team_id = match team_id {
                    Some(team_id) => Some(team_id),
                    None => select_team(Some(SELECT_ALL_TEAMS)).await?,
                };

                let projects_res =
                    ProjectsService::list_projects(team_id, ListQuery::Browse(None)).await?;
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
            ProjectsCommands::Create { team_id, name, env } => {
                let team_id = match team_id {
                    Some(team_id) => team_id,
                    None => select_team(None).await?.ok_or("A team must be selected")?,
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Project name")
                        .validate(|input: &String| {
                            if input.trim().is_empty() {
                                Err("Project name is required")
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?,
                };
                let env = match env {
                    Some(env) => Some(env),
                    None => {
                        let env: String = Input::new("Project environment")
                            .required(false)
                            .interact()?;
                        (!env.trim().is_empty()).then_some(env)
                    }
                };

                let project =
                    ProjectsService::create_project(CreateProjectDto { name, team_id, env })
                        .await?;
                let table = Self::get_projects_table(vec![project])?;
                println!("{table}");
            }
            ProjectsCommands::Update {
                project_id,
                name,
                env,
            } => {
                let project_id = match project_id {
                    Some(project_id) => project_id,
                    None => select_project(None, None)
                        .await?
                        .ok_or("A project must be selected")?,
                };
                let current_project = if name.is_none() || env.is_none() {
                    Some(ProjectsService::get_project(project_id.clone()).await?)
                } else {
                    None
                };

                let name = match name {
                    Some(name) => name,
                    None => {
                        let current_project = current_project
                            .as_ref()
                            .ok_or("Current project data is unavailable")?;
                        Input::new("Project name")
                            .default_input(&current_project.name)
                            .validate(|input: &String| {
                                if input.trim().is_empty() {
                                    Err("Project name is required")
                                } else {
                                    Ok(())
                                }
                            })
                            .interact()?
                    }
                };
                let env = match env {
                    Some(env) => Some(env),
                    None => {
                        let current_project = current_project
                            .as_ref()
                            .ok_or("Current project data is unavailable")?;
                        let mut input = Input::new("Project environment").required(false);
                        if let Some(current_env) = &current_project.env {
                            input = input.default_input(current_env);
                        }
                        let env: String = input.interact()?;
                        (!env.trim().is_empty()).then_some(env)
                    }
                };

                let project = ProjectsService::update_project(UpdateProjectDto {
                    id: project_id,
                    name: Some(name),
                    env,
                })
                .await?;
                let table = Self::get_projects_table(vec![project])?;
                println!("{table}");
            }
            ProjectsCommands::Delete { project_id, yes } => {
                let project_id = match project_id {
                    Some(project_id) => project_id,
                    None => select_project(None, None)
                        .await?
                        .ok_or("A project must be selected")?,
                };

                let confirmed = yes
                    || Confirm::new(format!("Delete project {project_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Project deletion cancelled")?;
                    return Ok(());
                }

                let project = ProjectsService::delete_project(project_id).await?;
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
