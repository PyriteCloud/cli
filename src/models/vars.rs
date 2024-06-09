use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct TemplateVars {
    pub name: Option<String>,
    pub questions: Vec<Question>,
    pub after: Option<After>,
}

#[derive(Deserialize)]
pub(crate) struct Question {
    #[serde(rename = "type")]
    pub _type: QuestionType,
    pub message: String,
    pub var_name: String,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) enum QuestionType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "select")]
    Select,
}

#[derive(Deserialize, Clone)]
pub(crate) struct After {
    pub goto: String,
}
