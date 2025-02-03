use cliclack::{Theme, ThemeState};
use console::Style;
pub(crate) mod handlebars;

pub(crate) const PYRITE_API_BASE_URL: &str = "http://localhost:50051";
pub(crate) const WORKFLOWS_BASE_URL: &str = "https://pyritecloud.github.io/workflows";
pub(crate) const ERR_MSG: &str = "There was an error, please try again";
pub(crate) const DOCKER_FILE: &str = "Dockerfile";

pub(crate) struct PyriteTheme;

impl Theme for PyriteTheme {
    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        Style::new().color256(123)
    }
}
