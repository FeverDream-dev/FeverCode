use std::time::Duration;

pub struct Transition {
    pub active: bool,
    pub progress: f32,
    pub duration: Duration,
}

impl Transition {
    pub fn new() -> Self {
        Self {
            active: false,
            progress: 1.0,
            duration: Duration::from_millis(150),
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.progress = 0.0;
    }

    pub fn tick(&mut self, delta: Duration) {
        if !self.active {
            return;
        }
        self.progress += delta.as_secs_f32() / self.duration.as_secs_f32();
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.active = false;
        }
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self::new()
    }
}
