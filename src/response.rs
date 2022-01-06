use std::borrow::Cow;

use serde::Serialize;

use crate::github_api::GitHubApiError;

#[derive(Serialize)]
pub struct SerializableResponse<'a> {
    #[serde(skip)]
    pub status: u16,
    pub code: Cow<'a, str>,
    pub message: Cow<'a, str>,
}

impl lambda_http::IntoResponse for SerializableResponse<'_> {
    fn into_response(self) -> lambda_http::Response<lambda_http::Body> {
        lambda_http::Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(lambda_http::Body::Text(
                serde_json::to_string(&self).unwrap(),
            ))
            .unwrap()
    }
}

pub enum ApiResponse {
    NoBodyFound,
    UnknownInput,
    NotTargeted,
    GitHubApiError(GitHubApiError),
}

impl<'a> ApiResponse {
    pub fn build(self) -> SerializableResponse<'a> {
        let (status, code, message) = match self {
            ApiResponse::NoBodyFound => (
                400,
                Cow::Borrowed("no_body_found"),
                Cow::Borrowed("No body found in request"),
            ),
            ApiResponse::UnknownInput => (
                400,
                Cow::Borrowed("unknown_input"),
                Cow::Borrowed("Unknown type of input. Check if any invalid webhook events are activated"),
            ),
            ApiResponse::NotTargeted => (
                200,
                Cow::Borrowed("not_targeted"),
                Cow::Borrowed("Not targeted to be tracked, skipping"),
            ),
            ApiResponse::GitHubApiError(err) => (
                500,
                Cow::Borrowed("github_api_error"),
                Cow::Owned(format!("Error occured while communicating with GitHub API: {:?}", err)),
            ),
        };

        SerializableResponse {
            status,
            code,
            message,
        }
    }
}
