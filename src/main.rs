use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod handlers;
mod image_ops;
mod s3;
mod utils;

use handlers::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let region_provider =
        Region::new(env::var("S3_REGION").unwrap_or_else(|_| "ap-northeast-1".to_string()));

    let mut config_loader =
        aws_config::defaults(BehaviorVersion::latest()).region(region_provider.clone());

    // Check for explicit credentials in env (S3_ACCESS_KEY_ID) to override
    // standard AWS_ACCESS_KEY_ID if needed, or just rely on standard AWS env vars.
    // The original project used S3_ACCESS_KEY_ID.
    if let (Ok(ak), Ok(sk)) = (
        env::var("S3_ACCESS_KEY_ID"),
        env::var("S3_SECRET_ACCESS_KEY"),
    ) {
        if !ak.is_empty() && !sk.is_empty() {
            config_loader =
                config_loader.credentials_provider(Credentials::new(ak, sk, None, None, "env"));
        }
    }

    let sdk_config = config_loader.load().await;

    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
        .region(region_provider)
        .force_path_style(true);

    if let Ok(endpoint) = env::var("S3_ENDPOINT") {
        if !endpoint.is_empty() {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }
    }

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

    let state = Arc::new(AppState { s3_client });

    let app = Router::new()
        .route("/wiki/thumb/*path", get(handlers::handle_resize))
        .route(
            "/webp/*path",
            get(handlers::handle_webp).delete(handlers::handle_purge),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
