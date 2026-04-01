use std::time::{Duration, Instant};

/// Simple rate limiter for outgoing Telegram messages.
///
/// Ensures messages are sent no more frequently than `min_interval` and
/// queues excess messages for later flushing.
pub struct RateLimiter {
    min_interval: Duration,
    last_sent: Option<Instant>,
    queue: Vec<String>,
}

impl RateLimiter {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last_sent: None,
            queue: Vec::new(),
        }
    }

    // Attempt to send immediately. Returns true if sent now, false if queued for later.
    pub fn try_send(&mut self, msg: String) -> bool {
        let now = Instant::now();
        match self.last_sent {
            None => {
                self.last_sent = Some(now);
                true
            }
            Some(last) => {
                if now.duration_since(last) >= self.min_interval {
                    self.last_sent = Some(now);
                    true
                } else {
                    self.queue.push(msg);
                    false
                }
            }
        }
    }

    // Retrieve and clear pending messages queued due to rate limiting.
    pub fn flush_pending(&mut self) -> Vec<String> {
        self.queue.drain(..).collect()
    }

    // Force immediate send, bypass rate limiter.
    pub fn force_send(&mut self, _msg: String) -> bool {
        self.last_sent = Some(Instant::now());
        true
    }
}
