//! `servers/audio/effects/audio_effect_chorus.*`: parameter state.
//!
//! Four voices; the constructor enables two and overrides their delays, rates,
//! depths, cutoffs and pans (`AudioEffectChorus::AudioEffectChorus`). Voices 2
//! and 3 keep the `Voice` initialiser values. `set_voice_count` rejects values
//! outside `1..=MAX_VOICES` and keeps the old count; per-voice setters reject an
//! out-of-range index. No setter clamps the value itself.
//!
//! The DSP is not ported.

pub const MAX_DELAY_MS: i32 = 50;
pub const MAX_DEPTH_MS: i32 = 20;
pub const MAX_WIDTH_MS: i32 = 50;
pub const MAX_VOICES: usize = 4;
pub const MAX_CHANNELS: i32 = 4;
pub const MS_CUTOFF_MAX: f32 = 16000.0;

#[derive(Clone, Copy, Debug)]
pub struct ChorusVoice {
    pub delay: f32,
    pub rate: f32,
    pub depth: f32,
    pub level: f32,
    pub cutoff: f32,
    pub pan: f32,
}

impl Default for ChorusVoice {
    fn default() -> Self {
        Self {
            delay: 12.0,
            rate: 1.0,
            depth: 0.0,
            level: 0.0,
            cutoff: MS_CUTOFF_MAX,
            pan: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectChorus {
    voices: [ChorusVoice; MAX_VOICES],
    voice_count: i32,
    wet: f32,
    dry: f32,
}

impl Default for AudioEffectChorus {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectChorus {
    pub fn new() -> Self {
        let mut voices = [ChorusVoice::default(); MAX_VOICES];

        voices[0].delay = 15.0;
        voices[1].delay = 20.0;
        voices[0].rate = 0.8;
        voices[1].rate = 1.2;
        voices[0].depth = 2.0;
        voices[1].depth = 3.0;
        voices[0].cutoff = 8000.0;
        voices[1].cutoff = 8000.0;
        voices[0].pan = -0.5;
        voices[1].pan = 0.5;

        Self {
            voices,
            voice_count: 2,
            wet: 0.5,
            dry: 1.0,
        }
    }

    /// Rejects counts outside `1..=MAX_VOICES`, keeping the previous value.
    pub fn set_voice_count(&mut self, count: i32) -> bool {
        if count < 1 || count as usize > MAX_VOICES {
            return false;
        }
        self.voice_count = count;
        true
    }

    pub fn get_voice_count(&self) -> i32 {
        self.voice_count
    }

    pub fn get_voice(&self, voice: usize) -> Option<ChorusVoice> {
        self.voices.get(voice).copied()
    }

    pub fn set_voice_delay_ms(&mut self, voice: usize, delay_ms: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.delay = delay_ms;
                true
            }
            None => false,
        }
    }

    pub fn set_voice_rate_hz(&mut self, voice: usize, rate_hz: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.rate = rate_hz;
                true
            }
            None => false,
        }
    }

    pub fn set_voice_depth_ms(&mut self, voice: usize, depth_ms: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.depth = depth_ms;
                true
            }
            None => false,
        }
    }

    pub fn set_voice_level_db(&mut self, voice: usize, level_db: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.level = level_db;
                true
            }
            None => false,
        }
    }

    pub fn set_voice_cutoff_hz(&mut self, voice: usize, cutoff_hz: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.cutoff = cutoff_hz;
                true
            }
            None => false,
        }
    }

    pub fn set_voice_pan(&mut self, voice: usize, pan: f32) -> bool {
        match self.voices.get_mut(voice) {
            Some(v) => {
                v.pan = pan;
                true
            }
            None => false,
        }
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet;
    }

    pub fn get_wet(&self) -> f32 {
        self.wet
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_constructor() {
        let c = AudioEffectChorus::new();
        assert_eq!(c.get_voice_count(), 2);
        assert_eq!(c.get_wet(), 0.5);
        assert_eq!(c.get_dry(), 1.0);

        let v0 = c.get_voice(0).unwrap();
        assert_eq!(v0.delay, 15.0);
        assert_eq!(v0.rate, 0.8);
        assert_eq!(v0.depth, 2.0);
        assert_eq!(v0.cutoff, 8000.0);
        assert_eq!(v0.pan, -0.5);

        let v1 = c.get_voice(1).unwrap();
        assert_eq!(v1.delay, 20.0);
        assert_eq!(v1.rate, 1.2);
        assert_eq!(v1.depth, 3.0);
        assert_eq!(v1.pan, 0.5);

        // Voices 2 and 3 keep the Voice initialiser.
        let v2 = c.get_voice(2).unwrap();
        assert_eq!(v2.delay, 12.0);
        assert_eq!(v2.rate, 1.0);
        assert_eq!(v2.depth, 0.0);
        assert_eq!(v2.cutoff, MS_CUTOFF_MAX);
        assert_eq!(v2.pan, 0.0);
    }

    #[test]
    fn voice_count_range_is_enforced() {
        let mut c = AudioEffectChorus::new();
        assert!(!c.set_voice_count(0));
        assert!(!c.set_voice_count(5));
        assert_eq!(c.get_voice_count(), 2);
        assert!(c.set_voice_count(4));
        assert_eq!(c.get_voice_count(), 4);
        assert!(c.set_voice_count(1));
    }

    #[test]
    fn voice_setters_reject_bad_indices_and_do_not_clamp() {
        let mut c = AudioEffectChorus::new();
        assert!(!c.set_voice_delay_ms(4, 10.0));
        assert!(!c.set_voice_pan(9, 0.0));

        assert!(c.set_voice_delay_ms(0, 99.0));
        assert_eq!(c.get_voice(0).unwrap().delay, 99.0);
        assert!(c.set_voice_depth_ms(1, -2.0));
        assert_eq!(c.get_voice(1).unwrap().depth, -2.0);
        assert!(c.set_voice_level_db(2, 3.0));
        assert_eq!(c.get_voice(2).unwrap().level, 3.0);
        assert!(c.set_voice_cutoff_hz(3, 1.0));
        assert_eq!(c.get_voice(3).unwrap().cutoff, 1.0);

        c.set_wet(-1.0);
        c.set_dry(5.0);
        assert_eq!(c.get_wet(), -1.0);
        assert_eq!(c.get_dry(), 5.0);
    }

    #[test]
    fn constants_match_the_header() {
        assert_eq!(MAX_DELAY_MS, 50);
        assert_eq!(MAX_DEPTH_MS, 20);
        assert_eq!(MAX_WIDTH_MS, 50);
        assert_eq!(MAX_VOICES, 4);
        assert_eq!(MAX_CHANNELS, 4);
        assert_eq!(MS_CUTOFF_MAX, 16000.0);
    }
}
