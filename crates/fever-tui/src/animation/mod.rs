pub mod pulse;
pub mod reveal;
pub mod transition;

use std::time::{Duration, Instant};

pub struct AnimationState {
    pub logo_reveal: reveal::LogoReveal,
    pub pulse_phase: f32,
    pub last_tick: Instant,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            logo_reveal: reveal::LogoReveal::new(),
            pulse_phase: 0.0,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.pulse_phase =
            (self.pulse_phase + delta.as_secs_f32() * 2.0) % (std::f32::consts::PI * 2.0);
        self.logo_reveal.advance(delta);
        self.last_tick = Instant::now();
    }

    pub fn pulse_intensity(&self) -> f32 {
        self.pulse_phase.sin() * 0.5 + 0.5
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new()
    }
}
