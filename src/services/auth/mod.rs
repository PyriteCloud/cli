use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use crate::models::auth::AuthParams;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use base64::prelude::*;
use chrono::Utc;
use rust_dotenv::dotenv::DotEnv;
use supabase_auth::models::{AuthClient, Session};
use tokio::{
    fs,
    net::TcpListener,
    sync::{Mutex, Notify, oneshot},
};
use tonic::metadata::MetadataMap;

struct AppState {
    shutdown: Notify,
    code_verifier: String,
    completion: Mutex<Option<oneshot::Sender<Result<(), String>>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct AuthService;

impl AuthService {
    const LISTEN_ADDRESS: &str = "127.0.0.1:3456";
    pub(crate) const CALLBACK_URL: &str = "http://127.0.0.1:3456/auth/callback";
    const SESSION_KEY: &str = if cfg!(debug_assertions) {
        "sb-127-auth-token"
    } else {
        "sb-uziefrixdcjogieucnel-auth-token"
    };

    /// Loads build-time Supabase configuration, falling back to dotenv files.
    pub fn get_auth_client() -> Result<AuthClient, Box<dyn Error>> {
        let (url, key) = match (option_env!("SUPABASE_URL"), option_env!("SUPABASE_API_KEY")) {
            (Some(url), Some(key)) => (Some(url.to_owned()), Some(key.to_owned())),
            _ => {
                let dotenv = DotEnv::new(if cfg!(debug_assertions) {
                    "local"
                } else {
                    "prod"
                });
                (
                    dotenv.get_var("SUPABASE_URL".to_owned()),
                    dotenv.get_var("SUPABASE_API_KEY".to_owned()),
                )
            }
        };
        let url = url
            .filter(|value| !value.is_empty())
            .ok_or("SUPABASE_URL not found")?;
        let key = key
            .filter(|value| !value.is_empty())
            .ok_or("SUPABASE_API_KEY not found")?;
        Ok(AuthClient::new(url, key, ""))
    }

    /// Reads the bearer token from the current process environment.
    pub fn get_pyrite_token() -> Result<String, Box<dyn Error>> {
        std::env::var("PYRITE_TOKEN").map_err(Into::into)
    }

    /// Returns the stored session, refreshing it when expired.
    /// Failed refreshes retain the stored credentials for retry and explicit logout.
    pub async fn get_session() -> Result<Session, Box<dyn Error>> {
        let session = Self::read_session()
            .await?
            .ok_or("No active session found. Please log in to continue.")?;
        let now = Utc::now().timestamp();
        // Preserve the existing inclusive expiry boundary without a wrapping cast.
        if now < 0 || session.expires_at >= now as u64 {
            return Ok(session);
        }
        let refreshed = Self::get_auth_client()?
            .refresh_session(&session.refresh_token)
            .await?;
        Self::write_session(&refreshed).await?;
        Ok(refreshed)
    }

    /// Prefers a valid session over the runtime bearer token.
    pub async fn get_metadata() -> Result<MetadataMap, Box<dyn Error>> {
        let mut metadata = MetadataMap::new();
        match (Self::get_session().await, Self::get_pyrite_token()) {
            (Ok(session), _) => {
                let encoded = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_string(&session)?);
                metadata.insert(Self::SESSION_KEY, format!("base64-{encoded}").parse()?);
            }
            (Err(_), Ok(token)) => {
                metadata.insert("authorization", format!("Bearer {token}").parse()?);
            }
            (Err(session_error), Err(_)) => {
                return Err(format!("No active session or PYRITE_TOKEN found. Please log in to continue. Session error: {session_error}").into());
            }
        }
        Ok(metadata)
    }

    /// Reads the stored session; a missing file means no active session.
    pub async fn read_session() -> Result<Option<Session>, Box<dyn Error>> {
        match fs::read_to_string(Self::get_session_path()?).await {
            Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Atomically saves the session without blocking Tokio workers.
    pub async fn write_session(session: &Session) -> Result<(), Box<dyn Error>> {
        let contents = serde_json::to_vec_pretty(session)?;
        let path = Self::get_session_path()?;
        // tempfile provides atomic replacement on both Unix and Windows.
        tokio::task::spawn_blocking(move || -> io::Result<()> {
            let parent = path.parent().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Session path has no parent")
            })?;
            std::fs::create_dir_all(parent)?;
            let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
            temporary.write_all(&contents)?;
            temporary.as_file().sync_all()?;
            temporary.persist(path).map_err(|error| error.error)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    /// Deletes the stored session. A missing file is already logged out.
    pub async fn delete_session() -> Result<(), Box<dyn Error>> {
        match fs::remove_file(Self::get_session_path()?).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn get_session_path() -> Result<PathBuf, Box<dyn Error>> {
        Ok(dirs::home_dir()
            .ok_or("Failed to get home directory")?
            .join(".pyrite")
            .join("session.json"))
    }

    async fn auth_callback_handler(
        State(state): State<Arc<AppState>>,
        Query(params): Query<AuthParams>,
    ) -> Response {
        let authorization = match (
            params.code.filter(|code| !code.trim().is_empty()),
            params.error.filter(|error| !error.trim().is_empty()),
        ) {
            (_, Some(error)) => Err(format!("OAuth provider rejected login: {error}")),
            (Some(code), None) => Ok(code),
            (None, None) => {
                return (StatusCode::BAD_REQUEST, "No authorization code received.")
                    .into_response();
            }
        };
        // Only one callback may exchange this PKCE code and complete the login.
        let Some(completion) = state.completion.lock().await.take() else {
            return (StatusCode::CONFLICT, "Login callback already received").into_response();
        };
        let result = match authorization {
            Ok(code) => {
                async {
                    let client = Self::get_auth_client().map_err(|error| error.to_string())?;
                    let session = client
                        .exchange_code_for_session(&code, &state.code_verifier)
                        .await
                        .map_err(|error| error.to_string())?;
                    Self::write_session(&session)
                        .await
                        .map_err(|error| format!("Failed to save session: {error}"))
                }
                .await
            }
            Err(error) => Err(error),
        };
        let response = match &result {
            Ok(()) => Redirect::temporary("https://www.pyrite.cloud").into_response(),
            Err(error) => {
                (StatusCode::BAD_REQUEST, format!("Failed to login: {error}")).into_response()
            }
        };
        let _ = completion.send(result);
        state.shutdown.notify_one();
        response
    }

    /// Waits for OAuth completion, returning exchange or persistence failures.
    pub async fn start_auth_server(code_verifier: String) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(Self::LISTEN_ADDRESS).await?;
        let (completion, result) = oneshot::channel();
        let state = Arc::new(AppState {
            shutdown: Notify::new(),
            completion: Mutex::new(Some(completion)),
            code_verifier,
        });
        let router = Router::new()
            .route("/auth/callback", get(Self::auth_callback_handler))
            .with_state(Arc::clone(&state));
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                state.shutdown.notified().await;
            })
            .await?;
        result.await?.map_err(Into::into)
    }
}
