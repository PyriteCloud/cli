use serde::Deserialize;
use std::clone::Clone;

#[derive(Deserialize, Clone)]
pub(crate) struct TemplateVars {
    pub name: Option<String>,
    pub questions: Vec<Question>,
    pub after: Option<After>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Question {
    #[serde(rename = "type")]
    pub q_type: QuestionType,
    pub message: String,
    pub var_name: String,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub(crate) enum QuestionType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "select")]
    Select,
    #[serde(rename = "confirm")]
    Confirm,
}

#[derive(Deserialize, Clone)]
pub(crate) struct After {
    pub goto: String,
}

impl After {
    pub(crate) fn new(goto: String) -> Self {
        Self { goto }
    }
}
