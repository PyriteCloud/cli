use std::collections::HashMap;

use oauth2::PkceCodeChallenge;
use supabase_auth::models::{LoginWithOAuthOptions, Provider, Session};

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

        // let session = auth_client
        // .login_with_email("user@pyrite.cloud", "Pyrite@Cloud")
        // .await?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let options = LoginWithOAuthOptions {
            query_params: Some(HashMap::from([
                (
                    "redirect_to".to_owned(),
                    "http://localhost:3000/auth/callback".to_owned(),
                ),
                ("response_type".to_owned(), "code".to_owned()),
                ("skip_browser_redirect".to_owned(), "true".to_owned()),
                (
                    "code_challenge".to_owned(),
                    pkce_challenge.as_str().to_owned(),
                ),
                ("code_challenge_method".to_owned(), "S256".to_owned()),
            ])),
            ..Default::default()
        };

        let oauth_res = auth_client
            // .login_with_email("user@pyrite.cloud", "Pyrite.Cloud")
            .login_with_oauth(Provider::Github, Some(options))?;

        println!("{}", oauth_res.url);

        AuthService::start_auth_server(pkce_verifier.into_secret()).await?;

        // let session = auth_client.exchange_token_for_session("").await?;

        // AuthService::write_session(&session)?;

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
