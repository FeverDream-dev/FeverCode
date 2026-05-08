use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushRequest {
    pub session_id: String,
    pub machine_id: String,
    pub encrypted_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPullRequest {
    pub session_id: String,
}

pub async fn push(Json(_req): Json<SyncPushRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"stored": true}))
}

pub async fn pull(Json(_req): Json<SyncPullRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"data": null, "message": "no sync data"}))
}
