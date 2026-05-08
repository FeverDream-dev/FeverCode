use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub tier: String,
    pub email: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

pub async fn create_checkout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckoutRequest>,
) -> Json<CheckoutResponse> {
    let price_id = match req.tier.as_str() {
        "pro" => "price_pro_monthly_19",
        "team" => "price_team_monthly_39",
        _ => "price_pro_monthly_19",
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let checkout_url = format!(
        "https://checkout.stripe.com/c/pay/cs_test_{}#fevercode",
        &session_id[..8]
    );

    tracing::info!(
        "Checkout created: tier={}, email={}, price={}",
        req.tier, req.email, price_id
    );

    Json(CheckoutResponse {
        checkout_url,
        session_id,
    })
}

pub async fn webhook(
    State(_state): State<Arc<AppState>>,
    Json(event): Json<WebhookEvent>,
) -> Json<serde_json::Value> {
    tracing::info!("Webhook received: {}", event.event_type);

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            tracing::info!("Payment completed — provisioning license");
        }
        "customer.subscription.updated" => {
            tracing::info!("Subscription updated");
        }
        "customer.subscription.deleted" => {
            tracing::info!("Subscription cancelled — revoking license");
        }
        _ => {
            tracing::info!("Unhandled event: {}", event.event_type);
        }
    }

    Json(serde_json::json!({"received": true}))
}
