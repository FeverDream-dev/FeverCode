pub struct Pulse {}

impl Pulse {
    pub fn new() -> Self {
        Self {}
    }

    pub fn style_alpha(&self, phase: f32) -> f32 {
        phase.sin() * 0.3 + 0.7
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Self::new()
    }
}
