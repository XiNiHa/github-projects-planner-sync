use config::{Config, File, FileFormat};
use response::ApiResponse;
use serde::{Deserialize, Serialize};
use webhook::Trackable;

pub mod webhook;
pub mod github_api;
pub mod response;

lazy_static::lazy_static! {
    static ref CONFIG: AppConfig = {
        let mut config = Config::default();
        config.merge(File::new("sync-config.yaml", FileFormat::Yaml)).expect("Failed to load config");
        config.try_into().expect("Config structure is invalid")
    };
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub github: github_api::GitHubConfig,

    pub tracked_actions: Vec<String>,
    pub tracked_labels: Vec<String>,
}

pub async fn webhook_handler(payload: webhook::WebhookPayload) -> ApiResponse {
    let is_tracked = match payload {
        webhook::WebhookPayload::Issues(payload) => {
            let has_tracking_action = CONFIG.tracked_actions.contains(&payload.action);
            let has_tracking_label = payload.issue.has_tracking_label();
            has_tracking_action && has_tracking_label
        }
        webhook::WebhookPayload::PullRequest(payload) => {
            let has_tracking_action = CONFIG.tracked_actions.contains(&payload.action);
            let has_tracking_label = payload.pull_request.has_tracking_label();
            has_tracking_action && has_tracking_label
        }
    };
    if !is_tracked {
        return ApiResponse::NotTargeted;
    }

    let github_project_state = match github_api::get_project_state().await {
        Ok(state) => state,
        Err(err) => return ApiResponse::GitHubApiError(err)
    };

    // TODO: remove this
    println!("{}", serde_json::to_string_pretty(&github_project_state).unwrap());

    todo!()
}
