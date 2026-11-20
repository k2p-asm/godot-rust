//! `servers/audio/effects/audio_effect_reverb.*`: the parameter state.
//!
//! Defaults are taken from `AudioEffectReverb::AudioEffectReverb()` and the only
//! setter that clamps is `predelay_feedback` (`0..0.98`); the rest assign
//! directly, like the engine. The DSP (`Reverb::process`) is not ported.

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectReverb {
    predelay: f32,
    predelay_fb: f32,
    hpf: f32,
    room_size: f32,
    damping: f32,
    spread: f32,
    dry: f32,
    wet: f32,
}

impl Default for AudioEffectReverb {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectReverb {
    pub fn new() -> Self {
        Self {
            predelay: 150.0,
            predelay_fb: 0.4,
            hpf: 0.0,
            room_size: 0.8,
            damping: 0.5,
            spread: 1.0,
            dry: 1.0,
            wet: 0.5,
        }
    }

    pub fn set_predelay_msec(&mut self, msec: f32) {
        self.predelay = msec;
    }

    pub fn get_predelay_msec(&self) -> f32 {
        self.predelay
    }

    /// Clamped to `0..0.98`, as in the engine.
    pub fn set_predelay_feedback(&mut self, feedback: f32) {
        self.predelay_fb = feedback.clamp(0.0, 0.98);
    }

    pub fn get_predelay_feedback(&self) -> f32 {
        self.predelay_fb
    }

    pub fn set_hpf(&mut self, hpf: f32) {
        self.hpf = hpf;
    }

    pub fn get_hpf(&self) -> f32 {
        self.hpf
    }

    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size;
    }

    pub fn get_room_size(&self) -> f32 {
        self.room_size
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn get_damping(&self) -> f32 {
        self.damping
    }

    pub fn set_spread(&mut self, spread: f32) {
        self.spread = spread;
    }

    pub fn get_spread(&self) -> f32 {
        self.spread
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet;
    }

    pub fn get_wet(&self) -> f32 {
        self.wet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_constructor() {
        let r = AudioEffectReverb::new();
        assert_eq!(r.get_predelay_msec(), 150.0);
        assert_eq!(r.get_predelay_feedback(), 0.4);
        assert_eq!(r.get_hpf(), 0.0);
        assert_eq!(r.get_room_size(), 0.8);
        assert_eq!(r.get_damping(), 0.5);
        assert_eq!(r.get_spread(), 1.0);
        assert_eq!(r.get_dry(), 1.0);
        assert_eq!(r.get_wet(), 0.5);
    }

    #[test]
    fn predelay_feedback_is_clamped() {
        let mut r = AudioEffectReverb::new();
        r.set_predelay_feedback(-1.0);
        assert_eq!(r.get_predelay_feedback(), 0.0);
        r.set_predelay_feedback(2.0);
        assert_eq!(r.get_predelay_feedback(), 0.98);
        r.set_predelay_feedback(0.25);
        assert_eq!(r.get_predelay_feedback(), 0.25);
    }

    #[test]
    fn other_setters_assign_directly() {
        let mut r = AudioEffectReverb::new();
        r.set_room_size(1.5);
        r.set_damping(-2.0);
        r.set_spread(0.3);
        r.set_dry(0.9);
        r.set_wet(0.1);
        r.set_hpf(120.0);
        r.set_predelay_msec(40.0);

        // No clamping on these, exactly like the engine setters.
        assert_eq!(r.get_room_size(), 1.5);
        assert_eq!(r.get_damping(), -2.0);
        assert_eq!(r.get_spread(), 0.3);
        assert_eq!(r.get_dry(), 0.9);
        assert_eq!(r.get_wet(), 0.1);
        assert_eq!(r.get_hpf(), 120.0);
        assert_eq!(r.get_predelay_msec(), 40.0);
    }
}
