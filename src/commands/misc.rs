use clap::Subcommand;
use comfy_table::{Cell, Table, modifiers, presets};

use crate::services::MiscService;

#[derive(Subcommand, Debug, Clone)]
#[command(about = "View platform options")]
pub(crate) enum MiscCommands {
    #[command(about = "List service types")]
    ServiceTypes,
    #[command(about = "List supported protocols")]
    Protocols,
    #[command(about = "List team roles")]
    Roles,
    #[command(about = "List runtimes")]
    Runtimes,
    #[command(about = "List builders")]
    Builders,
    #[command(about = "List regions")]
    Regions,
    #[command(about = "List plans")]
    Plans {
        #[arg(long, help = "Filter by service type")]
        service_type: Option<String>,
    },
    #[command(about = "List team subscriptions")]
    TeamSubscriptions,
    #[command(about = "List fixed prices")]
    FixedPrices,
}

impl MiscCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::ServiceTypes => print_items(
                "Service Type",
                MiscService::list_service_types().await?.items,
            )?,
            Self::Protocols => print_items("Protocol", MiscService::list_protocols().await?.items)?,
            Self::Roles => print_items("Role", MiscService::list_roles().await?.items)?,
            Self::Runtimes => print_items("Runtime", MiscService::list_runtimes().await?.items)?,
            Self::Builders => print_items("Builder", MiscService::list_builders().await?.items)?,
            Self::Regions => {
                let regions = MiscService::list_regions().await?.items;
                if regions.is_empty() {
                    cliclack::outro("No regions found")?;
                    return Ok(());
                }
                let mut table = table_with_header(vec!["Region", "Owner"]);
                for region in regions {
                    table.add_row(vec![Cell::new(region.name), Cell::new(region.owner)]);
                }
                println!("{table}");
            }
            Self::Plans { service_type } => {
                let plans = MiscService::list_plans(service_type).await?.items;
                if plans.is_empty() {
                    cliclack::outro("No plans found")?;
                    return Ok(());
                }
                let mut table = table_with_header(vec![
                    "Plan",
                    "CPU",
                    "Memory",
                    "Disk",
                    "Class",
                    "Price",
                    "Total Price",
                ]);
                for plan in plans {
                    table.add_row(vec![
                        Cell::new(plan.name),
                        Cell::new(plan.cpu),
                        Cell::new(plan.memory),
                        Cell::new(plan.disk),
                        Cell::new(plan.class),
                        Cell::new(format!("{:.2}", plan.price)),
                        Cell::new(plan.meta.map(|meta| meta.total_price).unwrap_or_default()),
                    ]);
                }
                println!("{table}");
            }
            Self::TeamSubscriptions => {
                let subscriptions = MiscService::list_team_subscriptions().await?.items;
                if subscriptions.is_empty() {
                    cliclack::outro("No team subscriptions found")?;
                    return Ok(());
                }
                let mut table = table_with_header(vec![
                    "Subscription",
                    "Members",
                    "Projects",
                    "Services",
                    "Deployments",
                    "Price",
                ]);
                for subscription in subscriptions {
                    table.add_row(vec![
                        Cell::new(subscription.name),
                        Cell::new(subscription.max_members),
                        Cell::new(subscription.max_projects),
                        Cell::new(subscription.max_services),
                        Cell::new(subscription.max_deployments),
                        Cell::new(format!("{:.2}", subscription.price)),
                    ]);
                }
                println!("{table}");
            }
            Self::FixedPrices => {
                let prices = MiscService::list_fixed_prices().await?.items;
                if prices.is_empty() {
                    cliclack::outro("No fixed prices found")?;
                    return Ok(());
                }
                let mut table = table_with_header(vec!["Name", "Price"]);
                for price in prices {
                    table.add_row(vec![
                        Cell::new(price.name),
                        Cell::new(format!("{:.2}", price.price)),
                    ]);
                }
                println!("{table}");
            }
        }
        Ok(())
    }
}

fn print_items(header: &str, items: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if items.is_empty() {
        cliclack::outro(format!("No {} found", header.to_lowercase()))?;
        return Ok(());
    }
    let mut table = table_with_header(vec![header]);
    for item in items {
        table.add_row(vec![Cell::new(item)]);
    }
    println!("{table}");
    Ok(())
}

fn table_with_header(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(headers);
    table
}
