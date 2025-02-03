use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use base64::prelude::*;
use rust_dotenv::dotenv::DotEnv;
use supabase_auth::models::{AuthClient, Session};
use tonic::metadata::MetadataMap;

#[derive(Debug, Clone)]
pub(crate) struct AuthService;

impl AuthService {
    pub fn get_auth_client() -> AuthClient {
        let dot_env = DotEnv::new("local");

        let project_url = dot_env.get_var("SUPABASE_URL".to_owned()).unwrap();
        let api_key = dot_env.get_var("SUPABASE_KEY".to_owned()).unwrap();
        let jwt_secret = dot_env.get_var("SUPABASE_SERVICE_KEY".to_owned()).unwrap();

        AuthClient::new(project_url, api_key, jwt_secret)
    }

    pub async fn refresh_session() -> Result<Session, Box<dyn Error>> {
        let auth_client = Self::get_auth_client();
        let session = Self::read_session().await?;

        if let Some(session) = session {
            let new_session = auth_client.refresh_session(&session.refresh_token).await?;
            return Ok(new_session);
        }

        Err("No session found".into())
    }

    pub async fn get_metadata() -> Result<MetadataMap, Box<dyn Error>> {
        let session = Self::read_session().await?;
        if let Some(session) = session {
            let headers = HashMap::from([(
                "sb-127-auth-token".to_owned(),
                format!(
                    "base64-{}",
                    BASE64_STANDARD_NO_PAD.encode(serde_json::to_string(&session)?)
                ),
            )]);
            let metadata = MetadataMap::from_headers(TryFrom::try_from(&headers).unwrap());
            Ok(metadata)
        } else {
            Err("No session found".into())
        }
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
