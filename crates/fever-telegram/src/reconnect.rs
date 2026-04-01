use std::future::Future;
use std::time::Duration;

/// Retry an async operation with exponential backoff: 1s, 2s, 4s, 8s, 16s.
pub async fn reconnect_with_backoff<F, Fut, T, E>(mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>> + Send,
{
    let delays = [1u64, 2, 4, 8, 16];
    for (idx, d) in delays.iter().enumerate() {
        match operation().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if idx + 1 == delays.len() {
                    return Err(e);
                }
                tokio::time::sleep(Duration::from_secs(*d)).await;
            }
        }
    }
    unreachable!()
}
