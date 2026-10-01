use chrono::{Duration, Utc};
use clap::Subcommand;
use cliclack::{Confirm, Input};
use pyrite_client_rs::pyrite::v1::services::v1::{
    CreateServiceEnvironmentDto, LogsCursorPagination, ServiceLogsFilter, ServiceLogsRequest,
    ServiceMetricSelector, ServiceMetricsFilter, ServiceMetricsRequest,
    UpdateServiceEnvironmentDto,
};

use crate::services::ListQuery;
use crate::services::service_environments::ServiceEnvironmentsService;
use crate::services::{ServiceLogsService, ServiceMetricsService};

use super::common::{select_service, select_service_environment};
use super::service_runtime::{
    print_deployments, print_details, print_logs, print_metrics, print_pods,
    service_environments_table,
};

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
    #[command(about = "Create service environment", visible_alias = "c")]
    Create {
        #[arg(short, long, help = "Service id")]
        service_id: Option<String>,
        #[arg(short, long, help = "Environment name")]
        name: Option<String>,
    },
    #[command(about = "Update service environment", visible_alias = "u")]
    Update {
        #[arg(short, long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(short, long, help = "Environment name")]
        name: Option<String>,
    },
    #[command(about = "Delete service environment", visible_alias = "d")]
    Delete {
        #[arg(short, long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List environment deployments")]
    Deployments {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
    },
    #[command(about = "Redeploy a service environment")]
    Redeploy {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "Deployment id to redeploy")]
        deployment_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Pause a service environment")]
    Pause {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Resume a service environment")]
    Resume {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List environment pods")]
    Pods {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
    },
    #[command(about = "Get environment runtime details")]
    Details {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
    },
    #[command(about = "Synchronize environment networking")]
    SyncNetwork {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List environment logs")]
    Logs {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "RFC 3339 start time")]
        from: Option<String>,
        #[arg(long, help = "RFC 3339 end time")]
        to: Option<String>,
        #[arg(long, help = "Filter by deployment id")]
        deployment_id: Option<String>,
        #[arg(long, help = "Filter by region")]
        region: Option<String>,
        #[arg(long, help = "Search log content")]
        search: Option<String>,
        #[arg(long, help = "Pagination cursor", default_value = "")]
        cursor: String,
        #[arg(long, help = "Page size", default_value_t = 100)]
        page_size: u32,
    },
    #[command(about = "List environment metrics")]
    Metrics {
        #[arg(long, help = "Environment id", visible_alias = "env-id")]
        environment_id: Option<String>,
        #[arg(long, help = "RFC 3339 start time")]
        from: Option<String>,
        #[arg(long, help = "RFC 3339 end time")]
        to: Option<String>,
        #[arg(long, help = "Filter by deployment id")]
        deployment_id: Option<String>,
        #[arg(long, help = "Filter by region")]
        region: Option<String>,
        #[arg(long, help = "Metric name")]
        metric: Vec<String>,
        #[arg(long, help = "Metric scope")]
        scope: Option<String>,
        #[arg(long, help = "Metric series")]
        series: Vec<String>,
        #[arg(long, help = "Averaging window in seconds", default_value_t = 60)]
        averaging_window_seconds: u32,
        #[arg(long, help = "Step in seconds", default_value_t = 60)]
        step_seconds: u32,
    },
}

impl EnvironmentsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            EnvironmentsCommands::List { service_id } => {
                let service_environments_res =
                    ServiceEnvironmentsService::list_service_environments(
                        Some(service_id),
                        ListQuery::Browse(None),
                    )
                    .await?;
                let service_environments = service_environments_res.service_environments;
                if service_environments.is_empty() {
                    cliclack::outro("No service environments found")?;
                } else {
                    let table = service_environments_table(service_environments)?;
                    println!("{table}");
                }
            }
            EnvironmentsCommands::Get { environment_id } => {
                let service_environment =
                    ServiceEnvironmentsService::get_service_environment(environment_id).await?;
                let table = service_environments_table(vec![service_environment])?;
                println!("{table}");
            }
            EnvironmentsCommands::Create { service_id, name } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let name = match name {
                    Some(name) => name,
                    None => Input::new("Environment name")
                        .validate(|input: &String| {
                            if input.trim().is_empty() {
                                Err("Environment name is required")
                            } else {
                                Ok(())
                            }
                        })
                        .interact()?,
                };

                let service_environment = ServiceEnvironmentsService::create_service_environment(
                    CreateServiceEnvironmentDto { name, service_id },
                )
                .await?;
                let table = service_environments_table(vec![service_environment])?;
                println!("{table}");
            }
            EnvironmentsCommands::Update {
                environment_id,
                name,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let current_environment = if name.is_none() {
                    Some(
                        ServiceEnvironmentsService::get_service_environment(environment_id.clone())
                            .await?,
                    )
                } else {
                    None
                };
                let name = match name {
                    Some(name) => name,
                    None => {
                        let current_environment = current_environment
                            .as_ref()
                            .ok_or("Current service environment data is unavailable")?;
                        Input::new("Environment name")
                            .default_input(&current_environment.name)
                            .validate(|input: &String| {
                                if input.trim().is_empty() {
                                    Err("Environment name is required")
                                } else {
                                    Ok(())
                                }
                            })
                            .interact()?
                    }
                };

                let service_environment = ServiceEnvironmentsService::update_service_environment(
                    UpdateServiceEnvironmentDto {
                        id: environment_id,
                        name,
                    },
                )
                .await?;
                let table = service_environments_table(vec![service_environment])?;
                println!("{table}");
            }
            EnvironmentsCommands::Delete {
                environment_id,
                yes,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };

                let confirmed = yes
                    || Confirm::new(format!("Delete service environment {environment_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Service environment deletion cancelled")?;
                    return Ok(());
                }

                let service_environment =
                    ServiceEnvironmentsService::delete_service_environment(environment_id).await?;
                let table = service_environments_table(vec![service_environment])?;
                println!("{table}");
            }
            EnvironmentsCommands::Deployments { environment_id } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                print_deployments(
                    ServiceEnvironmentsService::list_deployments(environment_id).await?,
                )?;
            }
            EnvironmentsCommands::Redeploy {
                environment_id,
                deployment_id,
                yes,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Redeploy environment {environment_id}?"))
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Redeployment cancelled")?;
                    return Ok(());
                }
                let environment =
                    ServiceEnvironmentsService::redeploy(environment_id, deployment_id).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            EnvironmentsCommands::Pause {
                environment_id,
                yes,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Pause environment {environment_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Pause cancelled")?;
                    return Ok(());
                }
                let environment = ServiceEnvironmentsService::pause(environment_id).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            EnvironmentsCommands::Resume {
                environment_id,
                yes,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Resume environment {environment_id}?"))
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Resume cancelled")?;
                    return Ok(());
                }
                let environment = ServiceEnvironmentsService::resume(environment_id).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            EnvironmentsCommands::Pods { environment_id } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                print_pods(ServiceEnvironmentsService::list_pods(environment_id).await?)?;
            }
            EnvironmentsCommands::Details { environment_id } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                print_details(ServiceEnvironmentsService::get_details(environment_id).await?)?;
            }
            EnvironmentsCommands::SyncNetwork {
                environment_id,
                yes,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Synchronize network for {environment_id}?"))
                        .initial_value(true)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Network synchronization cancelled")?;
                    return Ok(());
                }
                let environment = ServiceEnvironmentsService::sync_network(environment_id).await?;
                println!("{}", service_environments_table(vec![environment])?);
            }
            EnvironmentsCommands::Logs {
                environment_id,
                from,
                to,
                deployment_id,
                region,
                search,
                cursor,
                page_size,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let from = match from {
                    Some(from) => from,
                    None => {
                        let default_from = (Utc::now() - Duration::hours(1)).to_rfc3339();
                        Input::new("Logs from")
                            .default_input(&default_from)
                            .interact()?
                    }
                };
                let to = match to {
                    Some(to) => to,
                    None => {
                        let default_to = Utc::now().to_rfc3339();
                        Input::new("Logs to")
                            .default_input(&default_to)
                            .interact()?
                    }
                };
                print_logs(
                    ServiceLogsService::list(ServiceLogsRequest {
                        from,
                        to,
                        filter: Some(ServiceLogsFilter {
                            service_environment_id: environment_id,
                            deployment_id,
                            region,
                            search,
                        }),
                        pagination: Some(LogsCursorPagination { cursor, page_size }),
                    })
                    .await?,
                )?;
            }
            EnvironmentsCommands::Metrics {
                environment_id,
                from,
                to,
                deployment_id,
                region,
                metric,
                scope,
                series,
                averaging_window_seconds,
                step_seconds,
            } => {
                let environment_id = match environment_id {
                    Some(environment_id) => environment_id,
                    None => select_service_environment(None, None)
                        .await?
                        .ok_or("A service environment must be selected")?,
                };
                let from = match from {
                    Some(from) => from,
                    None => {
                        let default_from = (Utc::now() - Duration::hours(1)).to_rfc3339();
                        Input::new("Metrics from")
                            .default_input(&default_from)
                            .interact()?
                    }
                };
                let to = match to {
                    Some(to) => to,
                    None => {
                        let default_to = Utc::now().to_rfc3339();
                        Input::new("Metrics to")
                            .default_input(&default_to)
                            .interact()?
                    }
                };
                let metric = if metric.is_empty() {
                    vec![Input::new("Metric name").interact()?]
                } else {
                    metric
                };
                let metrics = metric
                    .into_iter()
                    .map(|metric| ServiceMetricSelector {
                        metric,
                        scope: scope.clone(),
                        series: series.clone(),
                    })
                    .collect();
                print_metrics(
                    ServiceMetricsService::list(ServiceMetricsRequest {
                        from,
                        to,
                        averaging_window_seconds,
                        step_seconds,
                        filter: Some(ServiceMetricsFilter {
                            service_environment_id: environment_id,
                            deployment_id,
                            region,
                        }),
                        metrics,
                    })
                    .await?,
                )?;
            }
        }
        Ok(())
    }
}
