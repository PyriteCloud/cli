use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Meta {
    pub options: Vec<Option>,
}

#[derive(Deserialize)]
pub(crate) struct Option {
    pub name: String,
    pub goto: String,
}
