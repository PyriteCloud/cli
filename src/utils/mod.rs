use cliclack::{Theme, ThemeState};
use console::Style;

pub(crate) const BASE_URL: &str = "https://pyritecloud.github.io/workflows";
pub(crate) const ERR_MSG: &str = "There was an error, please try again";

pub(crate) struct PyriteTheme;

impl Theme for PyriteTheme {
    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        Style::new().color256(123)
    }
}
