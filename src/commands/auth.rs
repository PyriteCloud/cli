use supabase_auth::models::Session;

use crate::services::{AuthService, UtilsService};

#[derive(Debug, Clone)]
pub(crate) struct AuthCommands;

impl AuthCommands {
    pub async fn login() -> Result<(), Box<dyn std::error::Error>> {
        let session = UtilsService::with_progress(
            AuthCommands::process_login,
            "Logging in...",
            "Login successful",
            "Failed to login",
        )
        .await?; // Session

        cliclack::outro(format!("Logged in as {}", session.user.email))?;

        Ok(())
    }

    async fn process_login() -> Result<Session, Box<dyn std::error::Error>> {
        let auth_client = AuthService::get_auth_client();

        let session = auth_client
            .login_with_email("user@pyrite.cloud", "Pyrite@Cloud")
            .await?;

        AuthService::write_session(&session)?;

        let session = AuthService::refresh_session().await?;
        Ok(session)
    }

    pub async fn logout() -> Result<(), Box<dyn std::error::Error>> {
        let session = UtilsService::with_progress(
            AuthService::read_session,
            "Reading session",
            "Session found",
            "Failed to read session",
        )
        .await?;

        if session.is_some() {
            AuthService::delete_session()?;
            cliclack::outro("Logged out")?;
        } else {
            cliclack::outro("No session found")?;
        }

        Ok(())
    }
}
