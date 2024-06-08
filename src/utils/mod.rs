use cliclack::{Theme, ThemeState};
use console::Style;
pub(crate) struct PyriteTheme;

impl Theme for PyriteTheme {
    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        Style::new().color256(123)
    }
}
