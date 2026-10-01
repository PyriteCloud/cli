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
        .await?;

        cliclack::outro(format!("Logged in as {}", session.user.email))?;

        Ok(())
    }

    async fn process_login() -> Result<Session, Box<dyn std::error::Error>> {
        // Check if we have a valid session and return it
        {
            let session = AuthService::get_session().await;
            if let Ok(session) = session {
                return Ok(session);
            }
        }

        // If no valid session, proceed with OAuth
        {
            let auth_client = AuthService::get_auth_client()?;

            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

            let options = LoginWithOAuthOptions {
                query_params: Some(HashMap::from([
                    ("skip_browser_redirect".to_owned(), "true".to_owned()),
                    (
                        "redirect_to".to_owned(),
                        AuthService::CALLBACK_URL.to_owned(),
                    ),
                    ("response_type".to_owned(), "code".to_owned()),
                    (
                        "code_challenge".to_owned(),
                        pkce_challenge.as_str().to_owned(),
                    ),
                    ("code_challenge_method".to_owned(), "S256".to_owned()),
                ])),
                ..Default::default()
            };

            let oauth_res = auth_client.login_with_oauth(Provider::Github, Some(options))?;

            println!("{}", oauth_res.url);

            AuthService::start_auth_server(pkce_verifier.into_secret()).await?;

            let session = AuthService::get_session().await?;

            Ok(session)
        }
    }

    pub async fn logout() -> Result<(), Box<dyn std::error::Error>> {
        let session = UtilsService::with_progress(
            AuthService::read_session,
            "Reading session",
            "Reading session",
            "Failed to read session",
        )
        .await?;

        if session.is_some() {
            AuthService::delete_session().await?;
            cliclack::outro("Logged out")?;
        } else {
            cliclack::outro("No session found")?;
        }

        Ok(())
    }
}
