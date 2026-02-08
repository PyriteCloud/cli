use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthParams {
    pub code: Option<String>,
}
