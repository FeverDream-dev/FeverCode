use std::time::{Duration, Instant};

pub struct LogoReveal {
    started_at: Option<Instant>,
    duration: Duration,
}

impl LogoReveal {
    pub fn new() -> Self {
        Self {
            started_at: None,
            duration: Duration::from_millis(1200),
        }
    }

    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }

    pub fn advance(&mut self, _delta: Duration) {}

    pub fn is_complete(&self) -> bool {
        match self.started_at {
            Some(start) => start.elapsed() >= self.duration,
            None => false,
        }
    }

    pub fn progress(&self) -> f32 {
        match self.started_at {
            Some(start) => {
                let elapsed = start.elapsed().as_millis() as f32;
                let total = self.duration.as_millis() as f32;
                (elapsed / total).min(1.0)
            }
            None => 0.0,
        }
    }

    pub fn phase(&self) -> LogoPhase {
        let p = self.progress();
        if p < 0.17 {
            LogoPhase::Clear
        } else if p < 0.42 {
            LogoPhase::Glyph
        } else if p < 0.67 {
            LogoPhase::Title
        } else if p < 1.0 {
            LogoPhase::Subtitle
        } else {
            LogoPhase::Complete
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoPhase {
    Clear,
    Glyph,
    Title,
    Subtitle,
    Complete,
}

impl Default for LogoReveal {
    fn default() -> Self {
        Self::new()
    }
}
