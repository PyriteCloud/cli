use cliclack::{Theme, ThemeState};
use console::Style;
pub(crate) mod handlebars;

pub(crate) const PYRITE_API_BASE_URL: &str = "grpc://api-grpc.pyrite.cloud";
pub(crate) const WORKFLOWS_BASE_URL: &str = "https://pyritecloud.github.io/workflows";
pub(crate) const ERR_MSG: &str = "There was an error, please try again";
pub(crate) const DOCKER_FILE: &str = "Dockerfile";

pub(crate) const TABLE_DATE_FORMAT: &str = "%d-%m-%Y %I:%M:%S %p %:z";

pub(crate) struct PyriteTheme;

impl Theme for PyriteTheme {
    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        Style::new().color256(123)
    }
}
