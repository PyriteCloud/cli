use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input, Select};
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::services::v1::{CreateServiceDto, common::v1::Service};

use crate::services::UtilsService;
use crate::services::{ListQuery, MiscService, ServiceEnvironmentsService, ServicesService};
use crate::utils::TABLE_DATE_FORMAT;

use super::common::{
    SELECT_ALL_PROJECTS, SELECT_ALL_TEAMS, select_project, select_service, select_team,
};
use super::service_domains::ServiceDomainsCommands;
use super::service_runtime::{print_deployments, service_environments_table};

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
    #[command(about = "Create service", visible_alias = "c")]
    Create {
        #[arg(short, long, help = "Project id")]
        project_id: Option<String>,
        #[arg(short, long, help = "Service name")]
        name: Option<String>,
        #[arg(short = 't', long = "type", help = "Service type")]
        service_type: Option<String>,
    },
    #[command(about = "Delete service", visible_alias = "d")]
    Delete {
        #[arg(short, long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Manage service domains")]
    Domains {
        #[command(subcommand)]
        domains_cmd: ServiceDomainsCommands,
    },
    #[command(about = "List service deployments")]
    Deployments {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Filter by environment name")]
        environment: Option<String>,
    },
    #[command(about = "Redeploy a service environment")]
    Redeploy {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Environment name")]
        environment: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Pause a service environment")]
    Pause {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Environment name")]
        environment: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Resume a service environment")]
    Resume {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Environment name")]
        environment: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Get the service token")]
    Token {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
    },
    #[command(about = "Regenerate the service token")]
    RegenerateToken {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Synchronize networking for all service environments")]
    SyncNetwork {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
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
                    (None, None) => select_team(Some(SELECT_ALL_TEAMS)).await?,
                };

                let project_id = match &project_id {
                    Some(project_id) => Some(project_id.to_owned()),
                    None => select_project(team_id.to_owned(), Some(SELECT_ALL_PROJECTS)).await?,
                };

                let services_res =
                    ServicesService::list_services(team_id, project_id, ListQuery::Browse(None))
                        .await?;
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
            ServicesCommands::Create {
                project_id,
                name,
                service_type,
            } => {
                let project_id = match project_id {
                    Some(project_id) => project_id,
                    None => {
                        let team_id = select_team(None).await?.ok_or("A team must be selected")?;
                        select_project(Some(team_id), None)
                            .await?
                            .ok_or("A project must be selected")?
                    }
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Service name")
                        .validate(|input: &String| {
                            if input.trim().is_empty() {
                                Err("Service name is required")
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?,
                };
                let service_type = match service_type {
                    Some(service_type) => service_type,
                    None => {
                        let service_types = MiscService::list_service_types().await?;
                        let options = service_types
                            .items
                            .into_iter()
                            .map(|service_type| (service_type.clone(), service_type, String::new()))
                            .collect::<Vec<_>>();

                        Select::new("Service type").items(&options).interact()?
                    }
                };

                let service = ServicesService::create_service(CreateServiceDto {
                    name,
                    r#type: service_type,
                    project_id,
                })
                .await?;
                let table = Self::get_services_table(vec![service])?;
                println!("{table}");
            }
            ServicesCommands::Delete { service_id, yes } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };

                let confirmed = yes
                    || Confirm::new(format!("Delete service {service_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Service deletion cancelled")?;
                    return Ok(());
                }

                let service = ServicesService::delete_service(service_id).await?;
                let table = Self::get_services_table(vec![service])?;
                println!("{table}");
            }
            ServicesCommands::Domains { domains_cmd } => domains_cmd.run().await?,
            ServicesCommands::Deployments {
                service_id,
                environment,
            } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                print_deployments(
                    ServicesService::list_deployments(service_id, environment).await?,
                )?;
            }
            ServicesCommands::Redeploy {
                service_id,
                environment,
                yes,
            } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let environment = match environment {
                    Some(environment) => Some(environment),
                    None => {
                        let environment_id = super::common::select_service_environment(
                            Some(service_id.clone()),
                            None,
                        )
                        .await?
                        .ok_or("A service environment must be selected")?;
                        Some(
                            ServiceEnvironmentsService::get_service_environment(environment_id)
                                .await?
                                .name,
                        )
                    }
                };
                let confirmed = yes
                    || Confirm::new("Redeploy the selected service environment?")
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Redeployment cancelled")?;
                    return Ok(());
                }
                let environment = ServicesService::redeploy(service_id, environment).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            ServicesCommands::Pause {
                service_id,
                environment,
                yes,
            } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let environment = match environment {
                    Some(environment) => Some(environment),
                    None => {
                        let environment_id = super::common::select_service_environment(
                            Some(service_id.clone()),
                            None,
                        )
                        .await?
                        .ok_or("A service environment must be selected")?;
                        Some(
                            ServiceEnvironmentsService::get_service_environment(environment_id)
                                .await?
                                .name,
                        )
                    }
                };
                let confirmed = yes
                    || Confirm::new("Pause the selected service environment?")
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Pause cancelled")?;
                    return Ok(());
                }
                let environment = ServicesService::pause(service_id, environment).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            ServicesCommands::Resume {
                service_id,
                environment,
                yes,
            } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let environment = match environment {
                    Some(environment) => Some(environment),
                    None => {
                        let environment_id = super::common::select_service_environment(
                            Some(service_id.clone()),
                            None,
                        )
                        .await?
                        .ok_or("A service environment must be selected")?;
                        Some(
                            ServiceEnvironmentsService::get_service_environment(environment_id)
                                .await?
                                .name,
                        )
                    }
                };
                let confirmed = yes
                    || Confirm::new("Resume the selected service environment?")
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Resume cancelled")?;
                    return Ok(());
                }
                let environment = ServicesService::resume(service_id, environment).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            ServicesCommands::Token { service_id } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                cliclack::note(
                    "Service token",
                    ServicesService::get_token(service_id).await?.token,
                )?;
            }
            ServicesCommands::RegenerateToken { service_id, yes } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Regenerate token for service {service_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Token regeneration cancelled")?;
                    return Ok(());
                }
                cliclack::note(
                    "Service token",
                    ServicesService::regenerate_token(service_id).await?.token,
                )?;
            }
            ServicesCommands::SyncNetwork { service_id, yes } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!(
                        "Synchronize networking for all environments of service {service_id}?"
                    ))
                    .initial_value(true)
                    .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Network synchronization cancelled")?;
                    return Ok(());
                }
                let service = ServicesService::sync_network(service_id).await?;
                println!("{}", Self::get_services_table(vec![service])?);
            }
        }
        Ok(())
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
