use axum::{http::StatusCode, routing::post, Json, Router};
use github_projects_planner_sync::{
    response::SerializableResponse, webhook::WebhookPayload, webhook_handler,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/webhook", post(handler));

    axum::Server::bind(&"[::]:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler<'a>(
    Json(payload): Json<WebhookPayload>,
) -> (StatusCode, Json<SerializableResponse<'a>>) {
    let response = webhook_handler(payload).await.build();
    (response.status.try_into().unwrap(), Json(response))
}
