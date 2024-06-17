use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::future::Future;

use clap::Subcommand;
use cliclack::{spinner, Confirm, Input, Select};
use handlebars::Handlebars;
use serde_json::Value;

use crate::models::options::Meta;
use crate::models::vars::{After, QuestionType, TemplateVars};
use crate::utils::handlebars::setup_handlebars;
use crate::utils::{BASE_URL, DOCKER_FILE, ERR_MSG};

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

                let options = meta
                    .options
                    .iter()
                    .map(|x| (&x.goto, &x.name, ""))
                    .collect::<Vec<_>>();

                let ans = Select::new("Which template would you like to use?")
                    .items(options.as_slice())
                    .interact();

                if let Ok(choice) = ans {
                    Self::process_choice(choice.to_owned()).await?;
                } else {
                    return Err(ERR_MSG)?;
                }
            }
        }
        Ok(())
    }

    async fn fetch_metadata() -> Result<Meta, Box<dyn std::error::Error>> {
        let meta = reqwest::get(format!("{BASE_URL}{}", "/templates/options.json"))
            .await?
            .json::<Meta>()
            .await?;

        Ok(meta)
    }

    async fn fetch_questions(path: &str) -> Result<TemplateVars, Box<dyn std::error::Error>> {
        let path = format!("{BASE_URL}{path}{}", "/vars.json");
        let t_vars = reqwest::get(path).await?.json::<TemplateVars>().await?;
        Ok(t_vars)
    }

    async fn fetch_template(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tmpl = reqwest::get(path).await?.text().await?;
        Ok(tmpl)
    }

    async fn process_choice(choice: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut after = Some(After::new(choice));
        let mut path = None;
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();

        while after.is_some() {
            (after, path) = Self::ask_questions(&after.unwrap().goto, &mut vars).await?;
        }

        let path = path.unwrap();
        let tmpl = Self::with_progress(
            || Self::fetch_template(&path),
            "Generating template",
            "Template generated",
            "Failed to generate template",
        )
        .await?;

        let mut handlebars = Handlebars::new();
        setup_handlebars(&mut handlebars);

        let out = handlebars.render_template(&tmpl, &vars)?;

        fs::write(DOCKER_FILE, out)?;

        Ok(())
    }

    async fn ask_questions(
        path: &str,
        answers: &mut BTreeMap<String, Value>,
    ) -> Result<(Option<After>, Option<String>), Box<dyn std::error::Error>> {
        let t_vars = Self::with_progress(
            || Self::fetch_questions(path),
            "Fetching questions",
            "Fetched questions",
            "Failed to fetch questions",
        )
        .await?;

        for question in t_vars.questions {
            match question.q_type {
                QuestionType::Input => {
                    let mut ans_input = Input::new(&question.message);
                    if let Some(default) = &question.default {
                        let default = default.to_owned();
                        ans_input = ans_input.default_input(&default).placeholder(&default);
                    }
                    let ans = ans_input.required(true).interact::<String>();

                    if let Ok(choice) = ans {
                        answers.insert(question.var_name, choice.into());
                    } else if let Some(choice) = question.default {
                        answers.insert(question.var_name, choice.into());
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
                        answers.insert(question.var_name, choice.into());
                    } else {
                        return Err(ERR_MSG)?;
                    }
                }
                QuestionType::Confirm => {
                    let ans = Confirm::new(&question.message).interact();
                    if let Ok(choice) = ans {
                        answers.insert(question.var_name, choice.into());
                    } else {
                        return Err(ERR_MSG)?;
                    }
                }
            }
        }

        if t_vars.after.is_some() {
            let handlebars = Handlebars::new();
            let goto = t_vars.after.unwrap().goto;
            let new_goto = handlebars.render_template(&goto, &answers)?;
            return Ok((Some(After::new(new_goto)), None));
        }

        let file_path = format!(
            "{BASE_URL}{path}/{}",
            t_vars.name.unwrap_or(DOCKER_FILE.to_owned())
        );

        Ok((None, Some(file_path)))
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
