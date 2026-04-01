use fever_telegram::reconnect::reconnect_with_backoff;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn test_reconnect_with_backoff_succeeds_after_retries() {
    // Simulate an operation that fails twice and then succeeds
    let counter = std::sync::Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    let result = reconnect_with_backoff(move || {
        let c = counter_clone.clone();
        async move {
            let n = c.fetch_add(1, Ordering::SeqCst);
            if n < 2 { Err(()) } else { Ok("ok") }
        }
    })
    .await;
    assert_eq!(result, Ok("ok"));
}
