use chrono::{DateTime, Local};
use clap::Subcommand;
use comfy_table::Cell;
use comfy_table::Table;
use comfy_table::modifiers;
use comfy_table::presets;
use pyrite_client_rs::pyrite::v1::services::v1::common::v1::ServiceEnvironment;
use pyrite_client_rs::pyrite::v1::services::v1::common::v1::service_environment;

use crate::services::UtilsService;
use crate::services::service_environments::ServiceEnvironmentsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
#[command(
    about = "Manage service environments",
    visible_aliases = ["envs", "e"],
    arg_required_else_help = false
)]
pub(crate) enum EnvironmentsCommands {
    #[command(about = "List all service environments", visible_alias = "ls")]
    List {
        #[arg(short, long, help = "List service environments by service id")]
        service_id: String,
    },
    #[command(about = "Get service environment", visible_alias = "g")]
    Get {
        #[arg(
            short,
            long,
            help = "Get service environment by environment id",
            visible_alias = "env-id"
        )]
        environment_id: String,
    },
}

impl EnvironmentsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            EnvironmentsCommands::List { service_id } => {
                let service_environments_res =
                    ServiceEnvironmentsService::list_service_environments(service_id).await?;
                let service_environments = service_environments_res.service_environments;
                if service_environments.is_empty() {
                    cliclack::outro("No service environments found")?;
                } else {
                    let table = Self::get_service_environments_table(service_environments)?;
                    println!("{table}");
                }
            }
            EnvironmentsCommands::Get { environment_id } => {
                let service_environment =
                    ServiceEnvironmentsService::get_service_environment(environment_id).await?;
                let table = Self::get_service_environments_table(vec![service_environment])?;
                println!("{table}");
            }
        }
        Ok(())
    }

    fn get_service_environments_table(
        service_environments: Vec<ServiceEnvironment>,
    ) -> Result<Table, Box<dyn std::error::Error>> {
        let mut table = Table::new();

        table
            .load_preset(presets::UTF8_FULL)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Environment Id",
                "Environment Name",
                "Service Name",
                "Type",
                "Status",
                "Deployment Status",
                "Created At",
                "Updated At",
            ]);

        for service_environment in service_environments {
            let created_at = DateTime::parse_from_rfc3339(service_environment.created_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            let updated_at = DateTime::parse_from_rfc3339(service_environment.updated_at.as_str())?
                .with_timezone(&Local)
                .format(TABLE_DATE_FORMAT)
                .to_string();

            let deployment_status =
                service_environment
                    .active_deployment
                    .map(|active_deployment| match active_deployment {
                        service_environment::ActiveDeployment::DockerDeployment(deployment) => {
                            deployment.status
                        }
                        service_environment::ActiveDeployment::PostgresDeployment(deployment) => {
                            deployment.status
                        }
                    });

            table.add_row(vec![
                Cell::new(service_environment.id),
                Cell::new(service_environment.name).fg(comfy_table::Color::White),
                Cell::new(
                    service_environment
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.service.as_ref())
                        .map(|service| service.name.to_owned())
                        .unwrap_or_default(),
                )
                .fg(comfy_table::Color::White),
                Cell::new(
                    service_environment
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.service.as_ref())
                        .map(|service| service.r#type.to_uppercase())
                        .unwrap_or_default(),
                ),
                Cell::new(UtilsService::get_service_status_label(
                    service_environment.status,
                ))
                .fg(UtilsService::get_service_status_color(
                    service_environment.status,
                )),
                deployment_status.map_or(Cell::new(""), |deployment_status| {
                    Cell::new(UtilsService::get_deployment_status_label(deployment_status))
                        .fg(UtilsService::get_deployment_status_color(deployment_status))
                }),
                Cell::new(created_at),
                Cell::new(updated_at),
            ]);
        }

        Ok(table)
    }
}
