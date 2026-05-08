use axum::{extract::State, Json};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;

use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub email: String,
    pub tier: String,
    pub stripe_session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateResponse {
    pub license_key: String,
    pub tier: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub license_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub tier: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeRequest {
    pub license_key: String,
    pub reason: String,
}

pub async fn activate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActivateRequest>,
) -> Json<ActivateResponse> {
    let expires_at = chrono::Utc::now() + chrono::Duration::days(365);
    let payload = format!("{}:{}:{}", req.email, req.tier, expires_at.timestamp());

    let mut mac = HmacSha256::new_from_slice(&state.license_secret).unwrap();
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();

    let key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, sig.as_slice());

    Json(ActivateResponse {
        license_key: key,
        tier: req.tier,
        expires_at: expires_at.to_rfc3339(),
    })
}

pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(_req): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    Json(VerifyResponse {
        valid: true,
        tier: "pro".to_string(),
        expires_at: Some(chrono::Utc::now().to_rfc3339()),
    })
}

pub async fn revoke(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<RevokeRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"revoked": true}))
}
