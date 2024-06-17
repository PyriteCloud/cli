use handlebars::handlebars_helper;
use serde_json::Value;

handlebars_helper!(is_equal_helper: |v1: Value,v2: Value| v1==v2);

pub(crate) fn setup_handlebars(handlebars: &mut handlebars::Handlebars) {
    handlebars.register_helper("isEqual", Box::new(is_equal_helper));
}
