use pyrite_client_rs::{
    helpers::request::ReqWithMetadata,
    pyrite::v1::{
        billing::v1::{
            BillingAccount, InvoiceById, Invoices, InvoicesPagination, Link, TopupWalletDto,
            Transactions, UpsertBillingAccountDto, Wallet, WalletTransactionById,
            billing_service_client::BillingServiceClient,
        },
        common::v1::Empty,
    },
};
use tonic::{Request, transport::Channel};

use crate::utils::PYRITE_API_BASE_URL;

use super::AuthService;

pub(crate) struct BillingService;

impl BillingService {
    async fn client() -> Result<BillingServiceClient<Channel>, Box<dyn std::error::Error>> {
        Ok(BillingServiceClient::connect(PYRITE_API_BASE_URL).await?)
    }

    pub async fn get_account() -> Result<BillingAccount, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(Empty {}, AuthService::get_metadata().await?);
        client
            .find_billing_account(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn upsert_account(
        dto: UpsertBillingAccountDto,
    ) -> Result<BillingAccount, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(dto, AuthService::get_metadata().await?);
        client
            .upsert_billing_account(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn list_invoices(page: i32) -> Result<Invoices, Box<dyn std::error::Error>> {
        if page < 1 {
            return Err("Invoice page must be greater than zero".into());
        }
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            InvoicesPagination { page },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_invoices(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn get_wallet() -> Result<Wallet, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(Empty {}, AuthService::get_metadata().await?);
        client
            .find_wallet(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn top_up(amount_cents: f32) -> Result<Wallet, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            TopupWalletDto { amount_cents },
            AuthService::get_metadata().await?,
        );
        client
            .top_up_wallet(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn list_transactions(page: i32) -> Result<Transactions, Box<dyn std::error::Error>> {
        if page < 1 {
            return Err("Transaction page must be greater than zero".into());
        }
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            InvoicesPagination { page },
            AuthService::get_metadata().await?,
        );
        client
            .find_all_wallet_transactions(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn transaction_payment_link(id: String) -> Result<Link, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> = ReqWithMetadata::with_metadata(
            WalletTransactionById { id },
            AuthService::get_metadata().await?,
        );
        client
            .generate_wallet_transaction_payment_link(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn invoice_download_link(id: String) -> Result<Link, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(InvoiceById { id }, AuthService::get_metadata().await?);
        client
            .generate_invoice_download_link(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }

    pub async fn invoice_payment_link(id: String) -> Result<Link, Box<dyn std::error::Error>> {
        let mut client = Self::client().await?;
        let request: Request<_> =
            ReqWithMetadata::with_metadata(InvoiceById { id }, AuthService::get_metadata().await?);
        client
            .generate_invoice_payment_link(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| error.message().into())
    }
}
