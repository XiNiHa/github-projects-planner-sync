use axum::http::HeaderValue;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use project_query::ProjectQueryOrganizationProjectNextItemsNodesContent as ProjectItemNodeContent;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::CONFIG;

pub static GITHUB_API_URL: &str = "https://api.github.com/graphql";

lazy_static::lazy_static! {
    static ref GITHUB_API_CLIENT: Option<Client> = {
        let header_value = HeaderValue::from_str(&format!("Bearer {}", CONFIG.github.api_key)).ok()?;
        Client::builder()
            .user_agent("reqwest/0.11.8")
            .default_headers(std::iter::once((reqwest::header::AUTHORIZATION, header_value)).collect())
            .build()
            .ok()
    };
}

#[derive(Serialize, Deserialize)]
pub struct GitHubConfig {
    pub api_key: String,
    pub organization_name: String,
    pub project_no: i64,
    pub bucket_param_name: String,
}

#[derive(Debug)]
pub enum GitHubApiError {
    ClientInitError,
    ReqwestError(reqwest::Error),
    GraphQLErrors(Vec<graphql_client::Error>),
    NoDataReceived,
    OrgNotFound,
    ProjectNotFound,
    ProjectItemNotFound,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/github-api-schema.graphql",
    query_path = "src/graphql/query.graphql"
)]
struct ProjectQuery;

#[derive(Deserialize)]
pub struct URI(String);

#[derive(Deserialize)]
struct ProjectFieldSettings {
    pub options: Vec<ProjectFieldOption>,
}

#[derive(Deserialize)]
struct ProjectFieldOption {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct ProjectState {
    pub items: Vec<ProjectItem>,
}

#[derive(Serialize)]
pub struct ProjectItem {
    pub _type: ProjectItemType,
    pub number: i64,
    pub title: String,
    pub url: String,
    pub status: String,
}

#[derive(Serialize)]
pub enum ProjectItemType {
    Issue,
    PullRequest,
}

pub async fn get_project_state() -> Result<ProjectState, GitHubApiError> {
    let client = GITHUB_API_CLIENT
        .as_ref()
        .ok_or(GitHubApiError::ClientInitError)?;
    let variables = project_query::Variables {
        login: CONFIG.github.organization_name.clone(),
        project_no: CONFIG.github.project_no,
    };
    let response_body = post_graphql::<ProjectQuery, _>(client, GITHUB_API_URL, variables)
        .await
        .map_err(GitHubApiError::ReqwestError)?;
    let response_data = match response_body.errors {
        None => response_body.data.ok_or(GitHubApiError::NoDataReceived),
        Some(errors) => Err(GitHubApiError::GraphQLErrors(errors)),
    }?;

    // TODO: should fetch all items using pagination, currently just fetching first 100 items
    let items = response_data
        .organization
        .ok_or(GitHubApiError::OrgNotFound)?
        .project_next
        .ok_or(GitHubApiError::ProjectNotFound)?
        .items
        .nodes
        .ok_or(GitHubApiError::ProjectItemNotFound)?
        .into_iter()
        .filter_map(|x| {
            x.and_then(|item| {
                let (_type, number, title, URI(url)) = match item.content? {
                    ProjectItemNodeContent::Issue(issue) => {
                        (ProjectItemType::Issue, issue.number, issue.title, issue.url)
                    }
                    ProjectItemNodeContent::PullRequest(pr) => {
                        (ProjectItemType::PullRequest, pr.number, pr.title, pr.url)
                    }
                };
                let status = item.field_values.nodes?.into_iter().find_map(|field| {
                    let field = field?;
                    if field.project_field.name == CONFIG.github.bucket_param_name {
                        let settings: ProjectFieldSettings =
                            serde_json::from_str(&field.project_field.settings?).ok()?;
                        let field_value = field.value?;
                        settings
                            .options
                            .into_iter()
                            .find(|opt| opt.id == field_value)
                            .map(|opt| opt.name)
                    } else {
                        None
                    }
                })?;

                Some(ProjectItem {
                    _type,
                    number,
                    title,
                    url,
                    status,
                })
            })
        });

    Ok(ProjectState {
        items: items.collect(),
    })
}
