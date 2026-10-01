use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthParams {
    pub code: Option<String>,
    pub error: Option<String>,
}
