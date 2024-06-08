pub mod dart;
use dart::{DART_TEMPLATE, DART_TEMPLATE_NAME};
use handlebars::Handlebars;

pub(crate) struct Template {
    name: String,
    tmpl: String,
}

pub(crate) enum Templates {
    Dart,
}

impl Templates {
    pub(crate) fn get_name(&self) -> String {
        match self {
            Templates::Dart => DART_TEMPLATE_NAME.to_owned(),
        }
    }

    pub(crate) fn get_template(&self) -> String {
        match self {
            Templates::Dart => DART_TEMPLATE.to_owned(),
        }
    }
}

pub(crate) fn setup_templates(
    handlebars: &mut Handlebars,
) -> Result<(), Box<dyn std::error::Error>> {
    let templates = [Template {
        name: Templates::Dart.get_name(),
        tmpl: Templates::Dart.get_template(),
    }];

    for template in templates {
        handlebars.register_template_string(&template.name, &template.tmpl)?;
    }

    Ok(())
}
