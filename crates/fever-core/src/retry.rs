use crate::error::{Error, Result};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub backoff_type: BackoffType,
}

#[derive(Debug, Clone, Copy)]
pub enum BackoffType {
    Exponential,
    Linear,
    Fixed,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            backoff_type: BackoffType::Exponential,
        }
    }
}

impl RetryPolicy {
    pub fn exponential(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            backoff_type: BackoffType::Exponential,
        }
    }

    pub fn linear(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            multiplier: 1.0,
            backoff_type: BackoffType::Linear,
        }
    }

    pub fn fixed(max_attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay: delay,
            max_delay: delay,
            multiplier: 1.0,
            backoff_type: BackoffType::Fixed,
        }
    }

    pub fn compute_delay(&self, attempt: u32) -> Duration {
        let delay_ms = match self.backoff_type {
            BackoffType::Exponential => {
                self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32 - 1)
            }
            BackoffType::Linear => {
                self.initial_delay.as_millis() as f64 * (attempt as f64)
            }
            BackoffType::Fixed => self.initial_delay.as_millis() as f64,
        };

        let delay_ms = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(delay_ms as u64)
    }
}

pub async fn retry_with_policy<F, Fut, T, E>(
    policy: &RetryPolicy,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut last_error = None;

    for attempt in 1..=policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(Error::Internal(format!("Attempt {} failed: {}", attempt, e)));

                if attempt < policy.max_attempts {
                    let delay = policy.compute_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::RetryExhausted("Operation failed".to_string())))
}
