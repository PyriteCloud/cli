use std::collections::BTreeMap;

use clap::{arg, Subcommand};
use cliclack::Select;
use handlebars::Handlebars;

use crate::templates::{dart::DART_TEMPLATE_NAME, setup_templates};

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum DockerCommands {
    Init {
        #[arg(short, long)]
        template: Option<String>,
    },
}

impl DockerCommands {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut handlebars = Handlebars::new();
        setup_templates(&mut handlebars)?;

        match self {
            DockerCommands::Init { mut template } => {
                // If the template is not set, ask the user
                if template.is_none() {
                    let options = vec![(DART_TEMPLATE_NAME, "Dart", "")];

                    let ans = Select::new("Which template would you like to use?")
                        .items(options.as_slice())
                        .interact();

                    match ans {
                        Ok(choice) => template = Some(choice.to_string()),
                        Err(_) => println!("There was an error, please try again"),
                    }
                }

                let mut vars = BTreeMap::new();
                vars.insert("NAME".to_owned(), "app".to_owned());

                // Render the template
                let output = handlebars
                    .render(template.unwrap().as_str(), &vars)
                    .unwrap();

                println!("{}", output);
            }
        }
        Ok(())
    }
}
