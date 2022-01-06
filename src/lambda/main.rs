use github_projects_planner_sync::{response::ApiResponse, webhook_handler};
use lambda_http::{
    handler,
    lambda_runtime::{self, Error},
    Context, IntoResponse, Request,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(request_handler)).await?;
    Ok(())
}

async fn request_handler(req: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let body = match req.into_body() {
        lambda_http::Body::Text(str) => Some(str),
        lambda_http::Body::Binary(bin) => String::from_utf8(bin).ok(),
        lambda_http::Body::Empty => None,
    };
    match body {
        Some(body) => Ok(match serde_json::from_str(&body) {
            Ok(payload) => webhook_handler(payload).await.build(),
            Err(_) => ApiResponse::UnknownInput.build(),
        }),
        None => Ok(ApiResponse::NoBodyFound.build()),
    }
}
