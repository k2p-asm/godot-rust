//! `servers/audio/effects/audio_effect_delay.*`: parameter state.
//!
//! Defaults come from the header's inline initialisers. All setters assign
//! directly (no clamping) in the engine. `MAX_DELAY_MS = 3000` is exposed as a
//! constant; the DSP buffer is not ported.

pub const MAX_DELAY_MS: f32 = 3000.0;
pub const MAX_TAPS: usize = 2;

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectDelay {
    dry: f32,
    tap_1_active: bool,
    tap_1_delay_ms: f32,
    tap_1_level: f32,
    tap_1_pan: f32,
    tap_2_active: bool,
    tap_2_delay_ms: f32,
    tap_2_level: f32,
    tap_2_pan: f32,
    feedback_active: bool,
    feedback_delay_ms: f32,
    feedback_level: f32,
    feedback_lowpass: f32,
}

impl Default for AudioEffectDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectDelay {
    pub fn new() -> Self {
        Self {
            dry: 1.0,
            tap_1_active: true,
            tap_1_delay_ms: 250.0,
            tap_1_level: -6.0,
            tap_1_pan: 0.2,
            tap_2_active: true,
            tap_2_delay_ms: 500.0,
            tap_2_level: -12.0,
            tap_2_pan: -0.4,
            feedback_active: false,
            feedback_delay_ms: 340.0,
            feedback_level: -6.0,
            feedback_lowpass: 16000.0,
        }
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }

    pub fn set_tap1_active(&mut self, active: bool) {
        self.tap_1_active = active;
    }

    pub fn is_tap1_active(&self) -> bool {
        self.tap_1_active
    }

    pub fn set_tap1_delay_ms(&mut self, delay_ms: f32) {
        self.tap_1_delay_ms = delay_ms;
    }

    pub fn get_tap1_delay_ms(&self) -> f32 {
        self.tap_1_delay_ms
    }

    pub fn set_tap1_level_db(&mut self, level_db: f32) {
        self.tap_1_level = level_db;
    }

    pub fn get_tap1_level_db(&self) -> f32 {
        self.tap_1_level
    }

    pub fn set_tap1_pan(&mut self, pan: f32) {
        self.tap_1_pan = pan;
    }

    pub fn get_tap1_pan(&self) -> f32 {
        self.tap_1_pan
    }

    pub fn set_tap2_active(&mut self, active: bool) {
        self.tap_2_active = active;
    }

    pub fn is_tap2_active(&self) -> bool {
        self.tap_2_active
    }

    pub fn set_tap2_delay_ms(&mut self, delay_ms: f32) {
        self.tap_2_delay_ms = delay_ms;
    }

    pub fn get_tap2_delay_ms(&self) -> f32 {
        self.tap_2_delay_ms
    }

    pub fn set_tap2_level_db(&mut self, level_db: f32) {
        self.tap_2_level = level_db;
    }

    pub fn get_tap2_level_db(&self) -> f32 {
        self.tap_2_level
    }

    pub fn set_tap2_pan(&mut self, pan: f32) {
        self.tap_2_pan = pan;
    }

    pub fn get_tap2_pan(&self) -> f32 {
        self.tap_2_pan
    }

    pub fn set_feedback_active(&mut self, active: bool) {
        self.feedback_active = active;
    }

    pub fn is_feedback_active(&self) -> bool {
        self.feedback_active
    }

    pub fn set_feedback_delay_ms(&mut self, delay_ms: f32) {
        self.feedback_delay_ms = delay_ms;
    }

    pub fn get_feedback_delay_ms(&self) -> f32 {
        self.feedback_delay_ms
    }

    pub fn set_feedback_level_db(&mut self, level_db: f32) {
        self.feedback_level = level_db;
    }

    pub fn get_feedback_level_db(&self) -> f32 {
        self.feedback_level
    }

    pub fn set_feedback_lowpass(&mut self, lowpass: f32) {
        self.feedback_lowpass = lowpass;
    }

    pub fn get_feedback_lowpass(&self) -> f32 {
        self.feedback_lowpass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_header() {
        let d = AudioEffectDelay::new();
        assert_eq!(d.get_dry(), 1.0);
        assert!(d.is_tap1_active());
        assert_eq!(d.get_tap1_delay_ms(), 250.0);
        assert_eq!(d.get_tap1_level_db(), -6.0);
        assert_eq!(d.get_tap1_pan(), 0.2);
        assert!(d.is_tap2_active());
        assert_eq!(d.get_tap2_delay_ms(), 500.0);
        assert_eq!(d.get_tap2_level_db(), -12.0);
        assert_eq!(d.get_tap2_pan(), -0.4);
        assert!(!d.is_feedback_active());
        assert_eq!(d.get_feedback_delay_ms(), 340.0);
        assert_eq!(d.get_feedback_level_db(), -6.0);
        assert_eq!(d.get_feedback_lowpass(), 16000.0);
    }

    #[test]
    fn setters_assign_without_clamping() {
        let mut d = AudioEffectDelay::new();
        d.set_tap1_delay_ms(9999.0);
        assert_eq!(d.get_tap1_delay_ms(), 9999.0);
        d.set_tap2_pan(-3.0);
        assert_eq!(d.get_tap2_pan(), -3.0);
        d.set_dry(-1.0);
        assert_eq!(d.get_dry(), -1.0);
        d.set_feedback_lowpass(0.0);
        assert_eq!(d.get_feedback_lowpass(), 0.0);

        d.set_tap1_active(false);
        d.set_tap2_active(false);
        d.set_feedback_active(true);
        assert!(!d.is_tap1_active());
        assert!(!d.is_tap2_active());
        assert!(d.is_feedback_active());
    }

    #[test]
    fn constants() {
        assert_eq!(MAX_DELAY_MS, 3000.0);
        assert_eq!(MAX_TAPS, 2);
    }
}
