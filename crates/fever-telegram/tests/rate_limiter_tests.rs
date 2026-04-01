use fever_telegram::RateLimiter;
use std::time::Duration;

#[tokio::test]
async fn test_rate_limiter_basic() {
    let mut rl = RateLimiter::new(Duration::from_millis(50));
    // first message should be sent immediately
    assert!(rl.try_send("a".to_string()));
    // second message immediately should be queued
    assert!(!rl.try_send("b".to_string()));
    // wait for interval
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(rl.try_send("c".to_string()));
}
