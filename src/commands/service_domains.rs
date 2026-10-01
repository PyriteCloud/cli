use chrono::{DateTime, Local};
use clap::Subcommand;
use cliclack::{Confirm, Input};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::services::v1::{
    CreateServiceDomainDto, ServiceDomain, UpdateServiceDomainDto,
};

use crate::commands::common::{
    SELECT_NONE, select_service, select_service_environment, select_value,
};
use crate::services::ServiceDomainsService;
use crate::utils::TABLE_DATE_FORMAT;

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ServiceDomainsCommands {
    #[command(about = "List service domains", visible_alias = "ls")]
    List {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
    },
    #[command(about = "Get a service domain", visible_alias = "g")]
    Get {
        #[arg(long, help = "Domain id")]
        domain_id: Option<String>,
        #[arg(long, help = "Service id used to select a domain")]
        service_id: Option<String>,
    },
    #[command(about = "Create a service domain", visible_alias = "c")]
    Create {
        #[arg(long, help = "Service id")]
        service_id: Option<String>,
        #[arg(long, help = "Domain name")]
        domain: Option<String>,
        #[arg(long, help = "Service environment id", visible_alias = "env-id")]
        service_environment_id: Option<String>,
    },
    #[command(about = "Update a service domain", visible_alias = "u")]
    Update {
        #[arg(long, help = "Domain id")]
        domain_id: Option<String>,
        #[arg(long, help = "Service id used to select a domain")]
        service_id: Option<String>,
        #[arg(long, help = "Service environment id", visible_alias = "env-id")]
        service_environment_id: Option<String>,
    },
    #[command(about = "Delete a service domain", visible_alias = "d")]
    Delete {
        #[arg(long, help = "Domain id")]
        domain_id: Option<String>,
        #[arg(long, help = "Service id used to select a domain")]
        service_id: Option<String>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
}

impl ServiceDomainsCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::List { service_id } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                print_domains(
                    ServiceDomainsService::list(service_id)
                        .await?
                        .service_domains,
                )?;
            }
            Self::Get {
                domain_id,
                service_id,
            } => {
                let domain_id = match domain_id {
                    Some(domain_id) => domain_id,
                    None => select_domain(service_id).await?,
                };
                print_domains(vec![ServiceDomainsService::get(domain_id).await?])?;
            }
            Self::Create {
                service_id,
                domain,
                service_environment_id,
            } => {
                let service_id = match service_id {
                    Some(service_id) => service_id,
                    None => select_service(None, None, None)
                        .await?
                        .ok_or("A service must be selected")?,
                };
                let domain = match domain {
                    Some(domain) => domain,
                    None => Input::new("Domain name").interact()?,
                };
                let service_environment_id = match service_environment_id {
                    Some(service_environment_id) => Some(service_environment_id),
                    None => {
                        select_service_environment(Some(service_id.clone()), Some(SELECT_NONE))
                            .await?
                    }
                };
                print_domains(vec![
                    ServiceDomainsService::create(CreateServiceDomainDto {
                        domain,
                        service_id,
                        service_environment_id,
                    })
                    .await?,
                ])?;
            }
            Self::Update {
                domain_id,
                service_id,
                service_environment_id,
            } => {
                let domain_id = match domain_id {
                    Some(domain_id) => domain_id,
                    None => select_domain(service_id).await?,
                };
                let service_environment_id = match service_environment_id {
                    Some(service_environment_id) => Some(service_environment_id),
                    None => {
                        let current = ServiceDomainsService::get(domain_id.clone()).await?;
                        select_service_environment(Some(current.service_id), Some(SELECT_NONE))
                            .await?
                    }
                };
                print_domains(vec![
                    ServiceDomainsService::update(UpdateServiceDomainDto {
                        id: domain_id,
                        service_environment_id,
                    })
                    .await?,
                ])?;
            }
            Self::Delete {
                domain_id,
                service_id,
                yes,
            } => {
                let domain_id = match domain_id {
                    Some(domain_id) => domain_id,
                    None => select_domain(service_id).await?,
                };
                let confirmed = yes
                    || Confirm::new(format!("Delete domain {domain_id}?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Domain deletion cancelled")?;
                    return Ok(());
                }
                print_domains(vec![ServiceDomainsService::delete(domain_id).await?])?;
            }
        }
        Ok(())
    }
}

async fn select_domain(service_id: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let service_id = match service_id {
        Some(service_id) => service_id,
        None => select_service(None, None, None)
            .await?
            .ok_or("A service must be selected")?,
    };
    let options = ServiceDomainsService::list(service_id)
        .await?
        .service_domains
        .into_iter()
        .map(|domain| {
            (
                domain.id,
                domain.domain,
                domain.service_environment_id.unwrap_or_default(),
            )
        })
        .collect();
    select_value("Select a domain", "No service domains found", options)
}

fn print_domains(domains: Vec<ServiceDomain>) -> Result<(), Box<dyn std::error::Error>> {
    if domains.is_empty() {
        cliclack::outro("No service domains found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Domain Id",
            "Domain",
            "Status",
            "Service Id",
            "Environment Id",
            "DNS Records",
            "Created At",
            "Updated At",
        ]);
    for domain in domains {
        let dns_records = domain
            .meta
            .map(|meta| {
                meta.dns_records
                    .into_iter()
                    .map(|record| format!("{} {} {}", record.r#type, record.name, record.value))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let created_at = DateTime::parse_from_rfc3339(&domain.created_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        let updated_at = DateTime::parse_from_rfc3339(&domain.updated_at)?
            .with_timezone(&Local)
            .format(TABLE_DATE_FORMAT);
        table.add_row(vec![
            Cell::new(domain.id),
            Cell::new(domain.domain),
            Cell::new(domain.status),
            Cell::new(domain.service_id),
            Cell::new(domain.service_environment_id.unwrap_or_default()),
            Cell::new(dns_records),
            Cell::new(created_at),
            Cell::new(updated_at),
        ]);
    }
    println!("{table}");
    Ok(())
}
