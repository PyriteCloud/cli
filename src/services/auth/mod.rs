use std::{collections::HashMap, env, error::Error, fs, path::PathBuf, sync::Arc};

use axum::{
    Router,
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
};
use base64::prelude::*;
use chrono::Utc;
use rust_dotenv::dotenv::DotEnv;
use supabase_auth::models::{AuthClient, Session};
use tokio::net::TcpListener;
use tonic::metadata::MetadataMap;

use crate::models::auth::AuthParams;
use tokio::sync::Notify;

#[derive(Clone)]
struct AppState {
    shutdown_notify: Arc<Notify>,
    code_verifier: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AuthService;

impl AuthService {
    pub fn get_auth_client() -> AuthClient {
        let env = if cfg!(debug_assertions) {
            "local"
        } else {
            "prod"
        };
        let dot_env = DotEnv::new(env);

        let project_url = dot_env.get_var("SUPABASE_URL".to_owned()).unwrap();
        let api_key = dot_env.get_var("SUPABASE_KEY".to_owned()).unwrap();

        AuthClient::new(project_url, api_key, "")
    }

    pub async fn get_session() -> Result<Session, Box<dyn Error>> {
        let auth_client = Self::get_auth_client();
        let session = Self::read_session().await?.ok_or("No session found")?;

        // Return the session if it's not expired
        if session.expires_at >= Utc::now().timestamp() as u64 {
            return Ok(session);
        }

        // Refresh the session if it's expired
        let new_session = auth_client
            .refresh_session(&session.refresh_token)
            .await
            .expect("Failed to refresh session");

        // Write the new session to file
        Self::write_session(&new_session)?;

        Ok(new_session)
    }

    pub async fn get_metadata() -> Result<MetadataMap, Box<dyn Error>> {
        let session = Self::get_session().await?;
        let cookie_key = if cfg!(debug_assertions) {
            "sb-127-auth-token"
        } else {
            "sb-uziefrixdcjogieucnel-auth-token"
        };
        let headers = HashMap::from([(
            cookie_key.to_owned(),
            format!(
                "base64-{}",
                BASE64_STANDARD_NO_PAD.encode(serde_json::to_string(&session)?)
            ),
        )]);
        let metadata = MetadataMap::from_headers(TryFrom::try_from(&headers).unwrap());
        Ok(metadata)
    }

    async fn auth_callback_handler(
        State(state): State<Arc<AppState>>,
        Query(params): Query<AuthParams>,
    ) -> impl IntoResponse {
        if let Some(code) = params.code {
            let auth_client = Self::get_auth_client();
            let session = auth_client
                .exchange_code_for_session(&code, &state.code_verifier)
                .await;

            // Trigger the graceful shutdown
            state.shutdown_notify.notify_one();

            match session {
                Ok(session) => {
                    let _ = Self::write_session(&session);
                    // return "Login successful you can close this window now".into_response();
                    return Redirect::temporary("https://www.pyrite.cloud").into_response();
                }
                Err(err) => {
                    return format!("Failed to login: {}", err).into_response();
                }
            }
        }

        "Something went wrong".into_response()
    }

    pub async fn start_auth_server(code_verifier: String) -> Result<(), Box<dyn Error>> {
        let shutdown_notify = Arc::new(Notify::new());

        let state: Arc<AppState> = Arc::new(AppState {
            shutdown_notify: shutdown_notify.clone(),
            code_verifier,
        });

        let router = Router::<Arc<AppState>>::new()
            .route("/auth/callback", get(AuthService::auth_callback_handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:3456").await.unwrap();

        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                shutdown_notify.notified().await;
            })
            .await?;

        Ok(())
    }

    pub async fn read_session() -> Result<Option<Session>, Box<dyn Error>> {
        let path_buf = Self::get_session_path();

        if path_buf.exists() {
            let session = fs::read_to_string(path_buf)?;
            Ok(Some(serde_json::from_str(&session)?))
        } else {
            Ok(None)
        }
    }

    pub fn write_session(session: &Session) -> Result<(), Box<dyn Error>> {
        let path_buf = Self::get_session_path();
        fs::create_dir_all(path_buf.parent().unwrap())?;
        fs::write(&path_buf, serde_json::to_string_pretty(session)?)?;
        Ok(())
    }

    pub fn delete_session() -> Result<(), Box<dyn Error>> {
        let path_buf = Self::get_session_path();
        fs::remove_file(path_buf)?;
        Ok(())
    }

    pub fn get_session_path() -> PathBuf {
        let home = env::var("HOME").unwrap();
        PathBuf::from(home).join(".pyrite").join("session.json")
    }
}
