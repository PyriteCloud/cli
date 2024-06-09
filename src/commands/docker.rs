use std::collections::BTreeMap;
use std::error::Error;
use std::future::Future;

use clap::Subcommand;
use cliclack::{spinner, Input, Select};
use handlebars::Handlebars;

use crate::models::options::Meta;
use crate::models::vars::{After, QuestionType, TemplateVars};
use crate::utils::{BASE_URL, ERR_MSG};

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum DockerCommands {
    Init,
}

impl DockerCommands {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DockerCommands::Init => {
                // If the template is not set, ask the user
                let meta = Self::with_progress(
                    Self::fetch_metadata,
                    "Fetching metadata",
                    "Fetched metadata",
                    "Failed to fetch metadata",
                )
                .await?;

                let options = meta.options;

                let ans = Select::new("Which template would you like to use?")
                    .items(
                        options
                            .iter()
                            .map(|x| (&x.goto, &x.name, ""))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .interact();

                if let Ok(choice) = ans {
                    let mut after = Some(After {
                        goto: choice.to_owned(),
                    });
                    let mut vars = BTreeMap::new();

                    while after.is_some() {
                        after =
                            Self::ask_questions(&after.clone().unwrap().goto, &mut vars).await?;
                    }

                    println!("{:?}", vars)
                } else {
                    return Err(ERR_MSG)?;
                }
            }
        }
        Ok(())
    }

    async fn fetch_metadata() -> Result<Meta, Box<dyn std::error::Error>> {
        let meta = reqwest::get(format!("{}{}", BASE_URL, "/templates/options.json"))
            .await?
            .json::<Meta>()
            .await?;

        Ok(meta)
    }
    async fn fetch_questions(path: &str) -> Result<TemplateVars, Box<dyn std::error::Error>> {
        let path = format!("{}{}{}", BASE_URL, path, "/vars.json");
        let t_vars = reqwest::get(path).await?.json::<TemplateVars>().await?;
        Ok(t_vars)
    }

    async fn ask_questions(
        path: &str,
        answers: &mut BTreeMap<String, String>,
    ) -> Result<Option<After>, Box<dyn std::error::Error>> {
        let t_vars = Self::with_progress(
            || Self::fetch_questions(path),
            "Fetching questions",
            "Fetched questions",
            "Failed to fetch questions",
        )
        .await?;

        for question in t_vars.questions {
            match question._type {
                QuestionType::Input => {
                    let ans = Input::new(&question.message)
                        .default_input(&question.default.clone().unwrap_or_default())
                        .placeholder(&question.default.clone().unwrap_or_default())
                        .interact();

                    if let Ok(choice) = ans {
                        answers.insert(question.var_name, choice);
                    } else if let Some(choice) = question.default {
                        answers.insert(question.var_name, choice);
                    } else {
                        return Err(ERR_MSG)?;
                    }
                }
                QuestionType::Select => {
                    let options = question
                        .options
                        .unwrap()
                        .iter()
                        .map(|x| (x.to_owned(), x.to_owned(), "".to_owned()))
                        .collect::<Vec<(String, String, String)>>();

                    let ans = Select::new(&question.message)
                        .items(options.as_slice())
                        .interact();

                    if let Ok(choice) = ans {
                        answers.insert(question.var_name, choice);
                    } else {
                        return Err(ERR_MSG)?;
                    }
                }
            }
        }

        if t_vars.after.is_some() {
            let handlebars = Handlebars::new();
            let new_goto = handlebars.render_template(&t_vars.after.unwrap().goto, &answers)?;

            return Ok(Some(After { goto: new_goto }));
        }

        Ok(t_vars.after)
    }

    async fn with_progress<T, R>(
        fun: impl FnOnce() -> T,
        msg: &str,
        success: &str,
        failed: &str,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        T: Future<Output = Result<R, Box<dyn Error>>>,
    {
        let progress = spinner();
        progress.start(msg);

        let result = fun().await;

        match result {
            Ok(x) => {
                progress.stop(success);
                Ok(x)
            }
            Err(err) => {
                progress.error(failed);
                Err(err)
            }
        }
    }
}
