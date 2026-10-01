use chrono::{DateTime, Local};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::services::v1::{
    Pods, ServiceDetails, ServiceLogs, ServiceMetrics,
    common::v1::{ServiceEnvironment, service_environment},
    deployments::v1::{Deployments, deployments::DeploymentsList},
};

use crate::services::UtilsService;
use crate::utils::TABLE_DATE_FORMAT;

pub(crate) fn service_environments_table(
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
        let created_at = DateTime::parse_from_rfc3339(&service_environment.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let updated_at = DateTime::parse_from_rfc3339(&service_environment.updated_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let deployment_status =
            service_environment
                .active_deployment
                .map(|deployment| match deployment {
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
                    .map(|service| service.name.as_str())
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

pub(crate) fn print_deployments(
    deployments: Deployments,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Deployment Id",
            "Environment Id",
            "Type",
            "Plan",
            "Status",
            "Source",
            "Created At",
        ]);

    match deployments.deployments_list {
        Some(DeploymentsList::DockerDeployments(deployments)) => {
            for deployment in deployments.deployments {
                let created_at = DateTime::parse_from_rfc3339(&deployment.created_at)?
                    .with_timezone(&Local)
                    .format(TABLE_DATE_FORMAT);
                table.add_row(vec![
                    Cell::new(deployment.id),
                    Cell::new(deployment.service_environment_id),
                    Cell::new("Docker"),
                    Cell::new(deployment.plan),
                    Cell::new(UtilsService::get_deployment_status_label(deployment.status))
                        .fg(UtilsService::get_deployment_status_color(deployment.status)),
                    Cell::new(deployment.source_type),
                    Cell::new(created_at),
                ]);
            }
        }
        Some(DeploymentsList::PostgresDeployments(deployments)) => {
            for deployment in deployments.deployments {
                let created_at = DateTime::parse_from_rfc3339(&deployment.created_at)?
                    .with_timezone(&Local)
                    .format(TABLE_DATE_FORMAT);
                table.add_row(vec![
                    Cell::new(deployment.id),
                    Cell::new(deployment.service_environment_id),
                    Cell::new("Postgres"),
                    Cell::new(deployment.plan),
                    Cell::new(UtilsService::get_deployment_status_label(deployment.status))
                        .fg(UtilsService::get_deployment_status_color(deployment.status)),
                    Cell::new(deployment.version),
                    Cell::new(created_at),
                ]);
            }
        }
        None => {}
    }

    if table.row_count() == 0 {
        cliclack::outro("No deployments found")?;
    } else {
        println!("{table}");
    }
    Ok(())
}

pub(crate) fn print_pods(pods: Pods) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["Region", "Pod", "Status", "Restarts", "Deployment Id"]);
    for region in pods.regions {
        for pod in region.pods {
            table.add_row(vec![
                Cell::new(&region.name),
                Cell::new(pod.name),
                Cell::new(pod.status),
                Cell::new(pod.restart_count),
                Cell::new(pod.deployment_id),
            ]);
        }
    }
    if table.row_count() == 0 {
        cliclack::outro("No pods found")?;
    } else {
        println!("{table}");
    }
    Ok(())
}

pub(crate) fn print_details(details: ServiceDetails) -> Result<(), Box<dyn std::error::Error>> {
    if details.details.is_empty() {
        cliclack::outro("No service details found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["Name", "Value", "Tags"]);
    for detail in details.details {
        table.add_row(vec![
            Cell::new(detail.name),
            Cell::new(detail.value),
            Cell::new(detail.tags.join(", ")),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub(crate) fn print_logs(logs: ServiceLogs) -> Result<(), Box<dyn std::error::Error>> {
    if logs.service_logs.is_empty() {
        cliclack::outro("No service logs found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["Timestamp", "Log"]);
    for log in logs.service_logs {
        table.add_row(vec![Cell::new(log.timestamp), Cell::new(log.log)]);
    }
    println!("{table}");
    if let Some(next_cursor) = logs
        .pagination
        .and_then(|pagination| pagination.next_cursor)
    {
        cliclack::note("Next cursor", next_cursor)?;
    }
    Ok(())
}

pub(crate) fn print_metrics(metrics: ServiceMetrics) -> Result<(), Box<dyn std::error::Error>> {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["Metric", "Unit", "Scope", "Series", "Time", "Value"]);
    for metric in metrics.metrics {
        for series in metric.series {
            for value in series.values {
                table.add_row(vec![
                    Cell::new(&metric.metric),
                    Cell::new(&metric.unit),
                    Cell::new(metric.scope.as_deref().unwrap_or_default()),
                    Cell::new(&series.series),
                    Cell::new(value.time),
                    Cell::new(value.value),
                ]);
            }
        }
    }
    if table.row_count() == 0 {
        cliclack::outro("No service metrics found")?;
    } else {
        println!("{table}");
    }
    Ok(())
}
