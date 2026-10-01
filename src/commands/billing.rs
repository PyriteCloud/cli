use clap::Subcommand;
use cliclack::{Confirm, Input};
use comfy_table::{Cell, Table, modifiers, presets};
use pyrite_client_rs::pyrite::v1::billing::v1::{
    BillingAccount, Invoices, Transactions, UpsertBillingAccountDto, Wallet,
};

use crate::commands::common::select_value;
use crate::services::BillingService;

#[derive(Subcommand, Debug, Clone)]
#[command(about = "Manage billing")]
pub(crate) enum BillingCommands {
    #[command(about = "Get the billing account")]
    Account,
    #[command(about = "Create or update the billing account")]
    UpdateAccount {
        #[arg(long, help = "Billing email")]
        email: Option<String>,
        #[arg(long, help = "First name")]
        first_name: Option<String>,
        #[arg(long, help = "Last name")]
        last_name: Option<String>,
        #[arg(long, help = "Phone number")]
        phone: Option<String>,
        #[arg(long, help = "Billing address")]
        address: Option<String>,
        #[arg(long, help = "Billing currency")]
        currency: Option<String>,
    },
    #[command(about = "List invoices", visible_alias = "invoices-list")]
    Invoices {
        #[arg(long, help = "Invoice page", default_value_t = 1)]
        page: i32,
    },
    #[command(about = "Get the wallet")]
    Wallet,
    #[command(about = "Top up the wallet")]
    TopUp {
        #[arg(long, help = "Top-up amount in cents")]
        amount_cents: Option<f32>,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List wallet transactions")]
    Transactions {
        #[arg(long, help = "Transaction page", default_value_t = 1)]
        page: i32,
    },
    #[command(about = "Generate a wallet transaction payment link")]
    TransactionPaymentLink {
        #[arg(long, help = "Wallet transaction id")]
        transaction_id: Option<String>,
        #[arg(
            long,
            help = "Transaction page used for selection",
            default_value_t = 1
        )]
        page: i32,
    },
    #[command(about = "Generate an invoice download link")]
    InvoiceDownloadLink {
        #[arg(long, help = "Invoice id")]
        invoice_id: Option<String>,
        #[arg(long, help = "Invoice page used for selection", default_value_t = 1)]
        page: i32,
    },
    #[command(about = "Generate an invoice payment link")]
    InvoicePaymentLink {
        #[arg(long, help = "Invoice id")]
        invoice_id: Option<String>,
        #[arg(long, help = "Invoice page used for selection", default_value_t = 1)]
        page: i32,
    },
}

impl BillingCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Account => print_account(BillingService::get_account().await?),
            Self::UpdateAccount {
                email,
                first_name,
                last_name,
                phone,
                address,
                currency,
            } => {
                let current = if email.is_none()
                    || first_name.is_none()
                    || last_name.is_none()
                    || phone.is_none()
                    || address.is_none()
                    || currency.is_none()
                {
                    Some(BillingService::get_account().await?)
                } else {
                    None
                };
                let email = match email {
                    Some(email) => email,
                    None => Input::new("Billing email")
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .email,
                        )
                        .interact()?,
                };
                let first_name = match first_name {
                    Some(first_name) => first_name,
                    None => Input::new("First name")
                        .required(false)
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .first_name,
                        )
                        .interact()?,
                };
                let last_name = match last_name {
                    Some(last_name) => last_name,
                    None => Input::new("Last name")
                        .required(false)
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .last_name,
                        )
                        .interact()?,
                };
                let phone = match phone {
                    Some(phone) => phone,
                    None => Input::new("Phone number")
                        .required(false)
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .phone,
                        )
                        .interact()?,
                };
                let address = match address {
                    Some(address) => address,
                    None => Input::new("Billing address")
                        .required(false)
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .address,
                        )
                        .interact()?,
                };
                let currency = match currency {
                    Some(currency) => currency,
                    None => Input::new("Billing currency")
                        .required(false)
                        .default_input(
                            &current
                                .as_ref()
                                .ok_or("Current billing account is unavailable")?
                                .currency,
                        )
                        .interact()?,
                };
                print_account(
                    BillingService::upsert_account(UpsertBillingAccountDto {
                        email,
                        first_name: Some(first_name),
                        last_name: Some(last_name),
                        phone: Some(phone),
                        address: Some(address),
                        currency: Some(currency),
                    })
                    .await?,
                );
            }
            Self::Invoices { page } => {
                print_invoices(BillingService::list_invoices(page).await?)?;
            }
            Self::Wallet => print_wallet(BillingService::get_wallet().await?),
            Self::TopUp { amount_cents, yes } => {
                let amount_cents = match amount_cents {
                    Some(amount_cents) => amount_cents,
                    None => Input::new("Top-up amount in cents")
                        .validate(|amount: &String| {
                            if amount.parse::<f32>().is_ok_and(|amount| amount > 0.0) {
                                Ok(())
                            } else {
                                Err("Amount must be greater than zero")
                            }
                        })
                        .interact()?,
                };
                if amount_cents <= 0.0 {
                    return Err("Amount must be greater than zero".into());
                }
                let confirmed = yes
                    || Confirm::new(format!("Top up the wallet by {amount_cents:.2} cents?"))
                        .initial_value(false)
                        .interact()?;
                if !confirmed {
                    cliclack::outro_cancel("Wallet top-up cancelled")?;
                    return Ok(());
                }
                print_wallet(BillingService::top_up(amount_cents).await?);
            }
            Self::Transactions { page } => {
                print_transactions(BillingService::list_transactions(page).await?)?;
            }
            Self::TransactionPaymentLink {
                transaction_id,
                page,
            } => {
                let transaction_id = match transaction_id {
                    Some(transaction_id) => transaction_id,
                    None => select_transaction(page).await?,
                };
                cliclack::note(
                    "Payment link",
                    BillingService::transaction_payment_link(transaction_id)
                        .await?
                        .link,
                )?;
            }
            Self::InvoiceDownloadLink { invoice_id, page } => {
                let invoice_id = match invoice_id {
                    Some(invoice_id) => invoice_id,
                    None => select_invoice(page).await?,
                };
                cliclack::note(
                    "Download link",
                    BillingService::invoice_download_link(invoice_id)
                        .await?
                        .link,
                )?;
            }
            Self::InvoicePaymentLink { invoice_id, page } => {
                let invoice_id = match invoice_id {
                    Some(invoice_id) => invoice_id,
                    None => select_invoice(page).await?,
                };
                cliclack::note(
                    "Payment link",
                    BillingService::invoice_payment_link(invoice_id).await?.link,
                )?;
            }
        }
        Ok(())
    }
}

async fn select_invoice(page: i32) -> Result<String, Box<dyn std::error::Error>> {
    let options = BillingService::list_invoices(page)
        .await?
        .invoices
        .into_iter()
        .map(|invoice| (invoice.id, invoice.invoice_number, invoice.amount))
        .collect();
    select_value("Select an invoice", "No invoices found", options)
}

async fn select_transaction(page: i32) -> Result<String, Box<dyn std::error::Error>> {
    let options = BillingService::list_transactions(page)
        .await?
        .transactions
        .into_iter()
        .map(|transaction| {
            (
                transaction.id,
                transaction.amount,
                transaction.transaction_status,
            )
        })
        .collect();
    select_value(
        "Select a transaction",
        "No wallet transactions found",
        options,
    )
}

fn print_account(account: BillingAccount) {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Email",
            "First Name",
            "Last Name",
            "Phone",
            "Address",
            "Currency",
            "Status",
        ])
        .add_row(vec![
            Cell::new(account.email),
            Cell::new(account.first_name),
            Cell::new(account.last_name),
            Cell::new(account.phone),
            Cell::new(account.address),
            Cell::new(account.currency),
            Cell::new(account.status),
        ]);
    println!("{table}");
}

fn print_wallet(wallet: Wallet) {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Wallet Id",
            "Balance",
            "Top Up",
            "Total Usage",
            "Projected Balance",
        ])
        .add_row(vec![
            Cell::new(wallet.wallet_id),
            Cell::new(wallet.balance),
            Cell::new(wallet.topup_amount.unwrap_or_default()),
            Cell::new(wallet.total_usage.unwrap_or_default()),
            Cell::new(wallet.projected_balance.unwrap_or_default()),
        ]);
    println!("{table}");
}

fn print_invoices(invoices: Invoices) -> Result<(), Box<dyn std::error::Error>> {
    if invoices.invoices.is_empty() {
        cliclack::outro("No invoices found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Invoice Id",
            "Number",
            "Amount",
            "Issued",
            "Status",
            "Payment Status",
            "Overdue",
        ]);
    for invoice in invoices.invoices {
        table.add_row(vec![
            Cell::new(invoice.id),
            Cell::new(invoice.invoice_number),
            Cell::new(invoice.amount),
            Cell::new(invoice.issuing_date),
            Cell::new(invoice.status),
            Cell::new(invoice.payment_status),
            Cell::new(invoice.payment_overdue),
        ]);
    }
    println!("{table}");
    if let Some(next_page) = invoices.next_page {
        cliclack::note("Next page", next_page)?;
    }
    Ok(())
}

fn print_transactions(transactions: Transactions) -> Result<(), Box<dyn std::error::Error>> {
    if transactions.transactions.is_empty() {
        cliclack::outro("No wallet transactions found")?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Transaction Id",
            "Amount",
            "Transaction Status",
            "Type",
            "Status",
            "Settled At",
        ]);
    for transaction in transactions.transactions {
        table.add_row(vec![
            Cell::new(transaction.id),
            Cell::new(transaction.amount),
            Cell::new(transaction.transaction_status),
            Cell::new(transaction.transaction_type),
            Cell::new(transaction.status),
            Cell::new(transaction.settled_at),
        ]);
    }
    println!("{table}");
    if let Some(next_page) = transactions.next_page {
        cliclack::note("Next page", next_page)?;
    }
    Ok(())
}
