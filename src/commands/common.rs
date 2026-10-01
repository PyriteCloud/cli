use std::collections::HashSet;
use std::error::Error;
use std::future::Future;

use cliclack::{Select, input};
use console::{Term, style};
use pyrite_client_rs::pyrite::v1::{
    common::v1::CursorPagination,
    projects::v1::Projects,
    services::v1::common::v1::{ServiceEnvironments, Services},
    teams::v1::Teams,
};

use crate::services::{
    ListQuery, MiscService, ProjectsService, ServiceEnvironmentsService, ServicesService,
    TeamsService, UtilsService,
};

const SELECT_PAGE_SIZE: i32 = 5;
const SELECT_MAX_ROWS: usize = 10;
const SUBMITTED_PROMPT_ROWS: usize = 3;

#[derive(Clone, Eq, PartialEq)]
enum Selection {
    All,
    Item(String),
    Search,
    ClearSearch,
    LoadMore,
}

struct SelectionItem {
    value: String,
    label: String,
    hint: String,
}

struct SelectionPage {
    items: Vec<SelectionItem>,
    has_next_page: bool,
    next_cursor: Option<String>,
}

struct SelectionMessages<'a> {
    prompt: &'a str,
    empty: &'a str,
    all: Option<SelectionAll<'a>>,
    search_prompt: &'a str,
    search_hint: &'a str,
    searching: &'a str,
    search_failed: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) struct SelectionAll<'a> {
    label: &'a str,
    hint: &'a str,
}

pub(crate) const SELECT_ALL_TEAMS: SelectionAll<'static> = SelectionAll {
    label: "All teams",
    hint: "Continue without filtering by team",
};

pub(crate) const SELECT_ALL_PROJECTS: SelectionAll<'static> = SelectionAll {
    label: "All projects",
    hint: "Continue without filtering by project",
};

pub(crate) const SELECT_NONE: SelectionAll<'static> = SelectionAll {
    label: "None",
    hint: "Do not assign an environment",
};

pub(crate) fn select_value(
    prompt: &str,
    empty: &str,
    items: Vec<(String, String, String)>,
) -> Result<String, Box<dyn Error>> {
    if items.is_empty() {
        return Err(empty.into());
    }

    Select::new(prompt)
        .items(&items)
        .max_rows(SELECT_MAX_ROWS)
        .interact()
        .map_err(Into::into)
}

pub(crate) fn redact_secret(secret: &str) -> String {
    let character_count = secret.chars().count();
    if character_count <= 4 {
        return "••••".to_owned();
    }

    format!(
        "••••{}",
        secret
            .chars()
            .skip(character_count.saturating_sub(4))
            .collect::<String>()
    )
}

pub(crate) async fn select_role(initial: Option<String>) -> Result<String, Box<dyn Error>> {
    let roles = MiscService::list_roles().await?;
    if roles.items.is_empty() {
        return Err("No team roles found".into());
    }

    let options = roles
        .items
        .into_iter()
        .map(|role| (role.clone(), role, String::new()))
        .collect::<Vec<_>>();
    let mut select = Select::new("Select a role").items(&options);
    if let Some(initial) = initial {
        select = select.initial_value(initial);
    }
    select.interact().map_err(Into::into)
}

enum SelectionMode {
    Browse,
    Search(Vec<SelectionItem>),
}

#[derive(Clone, Copy)]
enum ProjectSelectionScope {
    Team,
    Global,
}

#[derive(Clone, Copy)]
enum ServiceSelectionScope {
    Project,
    Team,
    Global,
}

#[derive(Clone, Copy)]
enum EnvironmentSelectionScope {
    Service,
    Global,
}

fn scope_hint(scope: &str, value: &str) -> String {
    if value.trim().is_empty() {
        String::new()
    } else {
        format!("{} {value}", style(format!("{scope}:")).cyan())
    }
}

fn join_hints(hints: impl IntoIterator<Item = String>) -> String {
    hints
        .into_iter()
        .filter(|hint| !hint.is_empty())
        .collect::<Vec<_>>()
        .join(" • ")
}

fn pagination(cursor: Option<String>) -> CursorPagination {
    CursorPagination {
        page_size: Some(SELECT_PAGE_SIZE),
        cursor,
        sort_by: None,
        sort_dir: None,
    }
}

fn merge_page(
    page: SelectionPage,
    items: &mut Vec<SelectionItem>,
    seen_values: &mut HashSet<String>,
    seen_cursors: &mut HashSet<String>,
) -> Result<Option<String>, Box<dyn Error>> {
    items.extend(
        page.items
            .into_iter()
            .filter(|item| seen_values.insert(item.value.clone())),
    );

    if !page.has_next_page {
        return Ok(None);
    }

    let next_cursor = page
        .next_cursor
        .ok_or("The API returned another page without a cursor")?;

    if !seen_cursors.insert(next_cursor.clone()) {
        return Err("The API returned a repeated pagination cursor".into());
    }

    Ok(Some(next_cursor))
}

fn team_selection_page(teams: Teams) -> SelectionPage {
    let items = teams
        .teams
        .into_iter()
        .map(|team| {
            let owner = team
                .meta
                .as_ref()
                .map(|meta| meta.owner_email.as_str())
                .filter(|owner| !owner.trim().is_empty())
                .unwrap_or(&team.owner);
            let hint = scope_hint("Owner", owner);

            SelectionItem {
                value: team.id,
                label: team.name,
                hint,
            }
        })
        .collect();

    SelectionPage {
        items,
        has_next_page: teams.has_next_page,
        next_cursor: teams.next_cursor,
    }
}

fn project_selection_page(projects: Projects, scope: ProjectSelectionScope) -> SelectionPage {
    let items = projects
        .projects
        .into_iter()
        .map(|project| {
            let services = project
                .meta
                .map(|meta| format!("{} services", meta.services_count))
                .unwrap_or_default();
            let hint = match scope {
                ProjectSelectionScope::Team => services,
                ProjectSelectionScope::Global => {
                    join_hints([scope_hint("Team", &project.team_id), services])
                }
            };

            SelectionItem {
                value: project.id,
                label: project.name,
                hint,
            }
        })
        .collect();

    SelectionPage {
        items,
        has_next_page: projects.has_next_page,
        next_cursor: projects.next_cursor,
    }
}

fn service_selection_page(services: Services, scope: ServiceSelectionScope) -> SelectionPage {
    let items = services
        .services
        .into_iter()
        .map(|service| {
            let r#type = scope_hint("Type", &service.r#type.to_uppercase());
            let hint = match scope {
                ServiceSelectionScope::Project => r#type,
                ServiceSelectionScope::Team => {
                    let project = service
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.project.as_ref())
                        .map(|project| scope_hint("Project", &project.name))
                        .unwrap_or_else(|| scope_hint("Project", &service.project_id));
                    join_hints([project, r#type])
                }
                ServiceSelectionScope::Global => {
                    let team = service
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.team.as_ref())
                        .map(|team| scope_hint("Team", &team.name))
                        .unwrap_or_default();
                    let project = service
                        .meta
                        .as_ref()
                        .and_then(|meta| meta.project.as_ref())
                        .map(|project| scope_hint("Project", &project.name))
                        .unwrap_or_else(|| scope_hint("Project", &service.project_id));
                    join_hints([team, project, r#type])
                }
            };

            SelectionItem {
                value: service.id,
                label: service.name,
                hint,
            }
        })
        .collect();

    SelectionPage {
        items,
        has_next_page: services.has_next_page,
        next_cursor: services.next_cursor,
    }
}

fn service_environment_selection_page(
    service_environments: ServiceEnvironments,
    scope: EnvironmentSelectionScope,
) -> SelectionPage {
    let items = service_environments
        .service_environments
        .into_iter()
        .map(|environment| {
            let hint = match scope {
                EnvironmentSelectionScope::Service => {
                    scope_hint("Namespace", &environment.namespace)
                }
                EnvironmentSelectionScope::Global => {
                    let meta = environment.meta.as_ref();
                    join_hints([
                        meta.and_then(|meta| meta.team.as_ref())
                            .map(|team| scope_hint("Team", &team.name))
                            .unwrap_or_default(),
                        meta.and_then(|meta| meta.project.as_ref())
                            .map(|project| scope_hint("Project", &project.name))
                            .unwrap_or_default(),
                        meta.and_then(|meta| meta.service.as_ref())
                            .map(|service| scope_hint("Service", &service.name))
                            .unwrap_or_else(|| scope_hint("Service", &environment.service_id)),
                    ])
                }
            };

            SelectionItem {
                value: environment.id,
                label: environment.name,
                hint,
            }
        })
        .collect();

    SelectionPage {
        items,
        has_next_page: service_environments.has_next_page,
        next_cursor: service_environments.next_cursor,
    }
}

fn with_items(mut select: Select<Selection>, items: &[SelectionItem]) -> Select<Selection> {
    for item in items {
        select = select.item(Selection::Item(item.value.clone()), &item.label, &item.hint);
    }

    select
}

fn browse_select(
    messages: &SelectionMessages<'_>,
    items: &[SelectionItem],
    has_next_page: bool,
) -> Select<Selection> {
    let mut select = Select::new(messages.prompt);

    if let Some(all) = &messages.all {
        select = select.item(Selection::All, style(all.label).cyan().bold(), all.hint);
    }

    let select = select.item(
        Selection::Search,
        style("Search").cyan().bold(),
        messages.search_hint,
    );
    let mut select = with_items(select, items);

    if has_next_page {
        select = select.item(
            Selection::LoadMore,
            style("Load more").cyan().bold(),
            format!("Fetch the next {SELECT_PAGE_SIZE} options"),
        );
    }

    select.max_rows(SELECT_MAX_ROWS)
}

fn search_select(messages: &SelectionMessages<'_>, items: &[SelectionItem]) -> Select<Selection> {
    let select = Select::new(messages.prompt).item(
        Selection::Search,
        style("Search again").cyan().bold(),
        messages.search_hint,
    );
    with_items(select, items)
        .item(
            Selection::ClearSearch,
            style("Clear search").cyan().bold(),
            "Return to all options",
        )
        .max_rows(SELECT_MAX_ROWS)
}

fn clear_submitted_prompt() -> Result<(), Box<dyn Error>> {
    Term::stderr()
        .clear_last_lines(SUBMITTED_PROMPT_ROWS)
        .map_err(Into::into)
}

async fn select_with_search<Browse, BrowseFuture, Search, SearchFuture>(
    messages: SelectionMessages<'_>,
    mut load_browse_page: Browse,
    mut search: Search,
) -> Result<Option<String>, Box<dyn Error>>
where
    Browse: FnMut(Option<String>) -> BrowseFuture,
    BrowseFuture: Future<Output = Result<SelectionPage, Box<dyn Error>>>,
    Search: FnMut(String) -> SearchFuture,
    SearchFuture: Future<Output = Result<SelectionPage, Box<dyn Error>>>,
{
    let mut seen_cursors = HashSet::new();
    let mut seen_values = HashSet::new();
    let mut browse_items = Vec::new();
    let mut mode = SelectionMode::Browse;

    let first_page = load_browse_page(None).await?;
    let mut next_cursor = merge_page(
        first_page,
        &mut browse_items,
        &mut seen_values,
        &mut seen_cursors,
    )?;

    while browse_items.is_empty() {
        let Some(cursor) = next_cursor.take() else {
            if messages.all.is_some() {
                break;
            }
            return Err(messages.empty.into());
        };

        let page = UtilsService::with_transient_progress(
            || load_browse_page(Some(cursor)),
            "Loading more options",
            "Failed to load more options",
        )
        .await?;
        next_cursor = merge_page(page, &mut browse_items, &mut seen_values, &mut seen_cursors)?;
    }

    loop {
        let selection = match &mode {
            SelectionMode::Browse => {
                browse_select(&messages, &browse_items, next_cursor.is_some()).interact()?
            }
            SelectionMode::Search(items) => search_select(&messages, items).interact()?,
        };

        match selection {
            Selection::All => return Ok(None),
            Selection::Item(id) => return Ok(Some(id)),
            Selection::Search => {
                clear_submitted_prompt()?;
                let query: String = input(messages.search_prompt).interact()?;
                clear_submitted_prompt()?;
                let query = query.trim().to_owned();

                if query.is_empty() {
                    mode = SelectionMode::Browse;
                    continue;
                }

                let page = UtilsService::with_transient_progress(
                    || search(query),
                    messages.searching,
                    messages.search_failed,
                )
                .await?;
                mode = SelectionMode::Search(page.items);
            }
            Selection::ClearSearch => {
                clear_submitted_prompt()?;
                mode = SelectionMode::Browse;
            }
            Selection::LoadMore => {
                clear_submitted_prompt()?;

                let cursor = next_cursor
                    .take()
                    .ok_or("No additional selection page is available")?;
                let page = UtilsService::with_transient_progress(
                    || load_browse_page(Some(cursor)),
                    "Loading more options",
                    "Failed to load more options",
                )
                .await?;
                next_cursor =
                    merge_page(page, &mut browse_items, &mut seen_values, &mut seen_cursors)?;
            }
        }
    }
}

pub(crate) async fn select_team(
    all: Option<SelectionAll<'_>>,
) -> Result<Option<String>, Box<dyn Error>> {
    select_with_search(
        SelectionMessages {
            prompt: "Select a team",
            empty: "No teams found",
            all,
            search_prompt: "Search teams",
            search_hint: "Search all teams",
            searching: "Searching teams",
            search_failed: "Failed to search teams",
        },
        |cursor| async move {
            TeamsService::list_teams(ListQuery::Browse(Some(pagination(cursor))))
                .await
                .map(team_selection_page)
        },
        |query| async move {
            TeamsService::list_teams(ListQuery::Search(query))
                .await
                .map(team_selection_page)
        },
    )
    .await
}

pub(crate) async fn select_project(
    team_id: Option<String>,
    all: Option<SelectionAll<'_>>,
) -> Result<Option<String>, Box<dyn Error>> {
    let team_id = match team_id {
        Some(team_id) => Some(team_id),
        None if all.is_none() => Some(select_team(None).await?.ok_or("A team must be selected")?),
        None => None,
    };
    let scope = if team_id.is_some() {
        ProjectSelectionScope::Team
    } else {
        ProjectSelectionScope::Global
    };
    let browse_team_id = &team_id;
    let search_team_id = &team_id;

    select_with_search(
        SelectionMessages {
            prompt: "Select a project",
            empty: "No projects found",
            all,
            search_prompt: "Search projects",
            search_hint: match scope {
                ProjectSelectionScope::Team => "Search projects in this team",
                ProjectSelectionScope::Global => "Search all projects",
            },
            searching: "Searching projects",
            search_failed: "Failed to search projects",
        },
        move |cursor| {
            let team_id = browse_team_id.clone();
            async move {
                ProjectsService::list_projects(team_id, ListQuery::Browse(Some(pagination(cursor))))
                    .await
                    .map(|projects| project_selection_page(projects, scope))
            }
        },
        move |query| {
            let team_id = search_team_id.clone();
            async move {
                ProjectsService::list_projects(team_id, ListQuery::Search(query))
                    .await
                    .map(|projects| project_selection_page(projects, scope))
            }
        },
    )
    .await
}

pub(crate) async fn select_service(
    team_id: Option<String>,
    project_id: Option<String>,
    all: Option<SelectionAll<'_>>,
) -> Result<Option<String>, Box<dyn Error>> {
    let (team_id, project_id) = match project_id {
        Some(project_id) => (team_id, Some(project_id)),
        None if all.is_none() => {
            let team_id = match team_id {
                Some(team_id) => team_id,
                None => select_team(None).await?.ok_or("A team must be selected")?,
            };
            let project_id = select_project(Some(team_id.clone()), None)
                .await?
                .ok_or("A project must be selected")?;
            (Some(team_id), Some(project_id))
        }
        None => (team_id, None),
    };
    let scope = if project_id.is_some() {
        ServiceSelectionScope::Project
    } else if team_id.is_some() {
        ServiceSelectionScope::Team
    } else {
        ServiceSelectionScope::Global
    };
    let browse_team_id = &team_id;
    let search_team_id = &team_id;
    let browse_project_id = &project_id;
    let search_project_id = &project_id;

    select_with_search(
        SelectionMessages {
            prompt: "Select a service",
            empty: "No services found",
            all,
            search_prompt: "Search services",
            search_hint: match scope {
                ServiceSelectionScope::Project => "Search services in this project",
                ServiceSelectionScope::Team => "Search services in this team",
                ServiceSelectionScope::Global => "Search all services",
            },
            searching: "Searching services",
            search_failed: "Failed to search services",
        },
        move |cursor| {
            let team_id = browse_team_id.clone();
            let project_id = browse_project_id.clone();
            async move {
                ServicesService::list_services(
                    team_id,
                    project_id,
                    ListQuery::Browse(Some(pagination(cursor))),
                )
                .await
                .map(|services| service_selection_page(services, scope))
            }
        },
        move |query| {
            let team_id = search_team_id.clone();
            let project_id = search_project_id.clone();
            async move {
                ServicesService::list_services(team_id, project_id, ListQuery::Search(query))
                    .await
                    .map(|services| service_selection_page(services, scope))
            }
        },
    )
    .await
}

pub(crate) async fn select_service_environment(
    service_id: Option<String>,
    all: Option<SelectionAll<'_>>,
) -> Result<Option<String>, Box<dyn Error>> {
    let service_id = match service_id {
        Some(service_id) => Some(service_id),
        None if all.is_none() => Some(
            select_service(None, None, None)
                .await?
                .ok_or("A service must be selected")?,
        ),
        None => None,
    };
    let scope = if service_id.is_some() {
        EnvironmentSelectionScope::Service
    } else {
        EnvironmentSelectionScope::Global
    };
    let browse_service_id = &service_id;
    let search_service_id = &service_id;

    select_with_search(
        SelectionMessages {
            prompt: "Select an environment",
            empty: "No service environments found",
            all,
            search_prompt: "Search environments",
            search_hint: match scope {
                EnvironmentSelectionScope::Service => "Search environments in this service",
                EnvironmentSelectionScope::Global => "Search all environments",
            },
            searching: "Searching environments",
            search_failed: "Failed to search environments",
        },
        move |cursor| {
            let service_id = browse_service_id.clone();
            async move {
                ServiceEnvironmentsService::list_service_environments(
                    service_id,
                    ListQuery::Browse(Some(pagination(cursor))),
                )
                .await
                .map(|environments| service_environment_selection_page(environments, scope))
            }
        },
        move |query| {
            let service_id = search_service_id.clone();
            async move {
                ServiceEnvironmentsService::list_service_environments(
                    service_id,
                    ListQuery::Search(query),
                )
                .await
                .map(|environments| service_environment_selection_page(environments, scope))
            }
        },
    )
    .await
}
