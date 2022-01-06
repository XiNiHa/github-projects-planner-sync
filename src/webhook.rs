use serde::{Serialize, Deserialize};

use crate::CONFIG;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookPayload {
    Issues(IssuesPayload),
    PullRequest(PullRequestPayload),
}

#[derive(Serialize, Deserialize)]
pub struct IssuesPayload {
    pub action: String,
    pub issue: Issue,
    pub repository: Repository,
}

#[derive(Serialize, Deserialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub repository: Repository,
}

#[derive(Serialize, Deserialize)]
pub struct Issue {
    labels: Vec<Label>,
}

#[derive(Serialize, Deserialize)]
pub struct PullRequest {
    labels: Vec<Label>,
}

#[derive(Serialize, Deserialize)]
pub struct Label {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Repository {
    full_name: String,
}

pub trait Trackable {
    fn labels(&self) -> &Vec<Label>;
    fn has_tracking_label(&self) -> bool {
        self.labels().iter().any(|label| CONFIG.tracked_labels.contains(&label.name))
    }
}

impl Trackable for Issue {
    fn labels(&self) -> &Vec<Label> {
        &self.labels
    }
}

impl Trackable for PullRequest {
    fn labels(&self) -> &Vec<Label> {
        &self.labels
    }
}
