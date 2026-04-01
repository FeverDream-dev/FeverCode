use crate::api::TelegramApi;
use crate::error::TelegramError;
use serde_json::Value;

type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;

/// Abstraction over Telegram API calls allowing mock injection in tests.
pub trait TelegramClient: Send + Sync {
    fn send_message(&self, chat_id: String, text: String) -> BoxFuture<Result<(), TelegramError>>;
    fn get_updates(&self, offset: Option<i64>) -> BoxFuture<Result<Vec<Value>, TelegramError>>;
}

/// Production client that delegates to the real TelegramApi.
pub struct RealTelegramClient {
    api: TelegramApi,
}

impl RealTelegramClient {
    pub fn new(token: String) -> Self {
        Self {
            api: TelegramApi::new(token),
        }
    }
}

impl TelegramClient for RealTelegramClient {
    fn send_message(&self, chat_id: String, text: String) -> BoxFuture<Result<(), TelegramError>> {
        let api = self.api.clone_facade();
        Box::pin(async move { api.send_message(&chat_id, &text).await })
    }

    fn get_updates(&self, offset: Option<i64>) -> BoxFuture<Result<Vec<Value>, TelegramError>> {
        let api = self.api.clone_facade();
        Box::pin(async move { api.get_updates(offset).await })
    }
}

#[cfg(test)]
pub struct MockTelegramClient {
    pub updates: std::sync::Mutex<Vec<Value>>,
    pub sent: std::sync::Mutex<Vec<(String, String)>>,
}

#[cfg(test)]
impl MockTelegramClient {
    pub fn new(updates: Vec<Value>) -> Self {
        Self {
            updates: std::sync::Mutex::new(updates),
            sent: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
impl TelegramClient for MockTelegramClient {
    fn send_message(&self, chat_id: String, text: String) -> BoxFuture<Result<(), TelegramError>> {
        self.sent.lock().unwrap().push((chat_id, text));
        Box::pin(async { Ok(()) })
    }

    fn get_updates(&self, _offset: Option<i64>) -> BoxFuture<Result<Vec<Value>, TelegramError>> {
        let updates = self.updates.lock().unwrap().clone();
        Box::pin(async move { Ok(updates) })
    }
}
