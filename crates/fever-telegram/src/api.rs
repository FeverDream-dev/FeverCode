use crate::error::TelegramError;
use serde_json::Value;

/// Minimal Telegram API client facade used by FeverCode tests.
///
/// This struct is designed for unit tests where actual HTTP requests are
/// avoided. It provides the surface necessary for internal logic without
/// performing network calls in tests.
#[derive(Clone)]
pub struct TelegramApi {
    token: String,
    client: reqwest::Client,
}

impl TelegramApi {
    pub fn clone_facade(&self) -> Self {
        Self {
            token: self.token.clone(),
            client: self.client.clone(),
        }
    }
}

impl TelegramApi {
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .build()
            .expect("Failed to build HTTP client");
        Self { token, client }
    }

    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), TelegramError> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let payload = serde_json::json!({"chat_id": chat_id, "text": text});
        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| TelegramError::Network(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status().as_u16() as i32;
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            Err(TelegramError::Api {
                code: status,
                message: body,
            })
        }
    }

    pub async fn get_updates(&self, offset: Option<i64>) -> Result<Vec<Value>, TelegramError> {
        let url = format!("https://api.telegram.org/bot{}/getUpdates", self.token);
        let mut req = self.client.get(&url).query(&[("timeout", "30")]);
        if let Some(off) = offset {
            req = req.query(&[("offset", &off.to_string())]);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| TelegramError::Network(e.to_string()))?;
        if resp.status().is_success() {
            let json: Value = resp.json().await.map_err(|e| TelegramError::Api {
                code: -1,
                message: e.to_string(),
            })?;
            let updates = json
                .get("result")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            Ok(updates)
        } else {
            let status = resp.status().as_u16() as i32;
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            Err(TelegramError::Api {
                code: status,
                message: body,
            })
        }
    }
}
