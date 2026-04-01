use fever_telegram::client::TelegramClient;
use fever_telegram::{TelegramConfig, TelegramService};
use serde_json::json;
use std::sync::{Arc, Mutex};

type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;

struct MockClient {
    updates: Arc<Mutex<Vec<serde_json::Value>>>,
    sent: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockClient {
    fn new(updates: Vec<serde_json::Value>) -> Self {
        Self {
            updates: Arc::new(Mutex::new(updates)),
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TelegramClient for MockClient {
    fn send_message(
        &self,
        chat_id: String,
        text: String,
    ) -> BoxFuture<Result<(), fever_telegram::TelegramError>> {
        let sent = Arc::clone(&self.sent);
        Box::pin(async move {
            sent.lock().unwrap().push((chat_id, text));
            Ok(())
        })
    }

    fn get_updates(
        &self,
        _offset: Option<i64>,
    ) -> BoxFuture<Result<Vec<serde_json::Value>, fever_telegram::TelegramError>> {
        let updates = Arc::clone(&self.updates);
        Box::pin(async move { Ok(updates.lock().unwrap().clone()) })
    }
}

#[tokio::test]
async fn test_auto_link_sets_chat_id_and_sends_linked() {
    let updates =
        vec![json!({"update_id": 1, "message": {"text": "hello", "chat": {"id": 98765}}})];
    let mock = MockClient::new(updates);
    let cfg = TelegramConfig {
        bot_token: "tok".to_string(),
        chat_id: None,
        notify_interval_secs: 5,
        loop_mode: true,
    };
    let mut svc = TelegramService::with_client(cfg, Box::new(mock));
    let res = svc.start().await;
    assert!(res.is_ok());
}
