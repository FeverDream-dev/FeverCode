use axum::{routing::{get, post}, Router, Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod license;
mod sync;
mod stripe;

#[derive(Clone)]
struct AppState {
    license_secret: Vec<u8>,
    stripe_key: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("fever_cloud_api=info").init();

    let state = AppState {
        license_secret: std::env::var("LICENSE_SECRET")
            .unwrap_or_else(|_| "fevercode-license-secret-v1".to_string())
            .into_bytes(),
        stripe_key: std::env::var("STRIPE_SECRET_KEY")
            .unwrap_or_else(|_| "sk_test_placeholder".to_string()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/license/activate", post(license::activate))
        .route("/api/v1/license/verify", post(license::verify))
        .route("/api/v1/license/revoke", post(license::revoke))
        .route("/api/v1/sync/push", post(sync::push))
        .route("/api/v1/sync/pull", post(sync::pull))
        .route("/api/v1/billing/checkout", post(stripe::create_checkout))
        .route("/api/v1/billing/webhook", post(stripe::webhook))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Fever Cloud API listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}
