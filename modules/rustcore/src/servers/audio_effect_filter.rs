//! `servers/audio/effects/audio_effect_filter.*` and `audio_effect_limiter.*`.
//!
//! Parameter state only. Filters share one class with a mode; the constructor
//! sets `cutoff = 2000`, `resonance = 0.5`, `gain = 1.0`, `db = FILTER_6DB`.
//! The limiter defaults come from its constructor: threshold 0, ceiling -0.1,
//! soft clip 2 dB and ratio 10. No setter clamps.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FilterDb {
    Db6,
    Db12,
    Db18,
    Db24,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
    BandLimit,
    Notch,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectFilter {
    mode: FilterMode,
    cutoff: f32,
    resonance: f32,
    gain: f32,
    db: FilterDb,
}

impl AudioEffectFilter {
    pub fn new(mode: FilterMode) -> Self {
        Self {
            mode,
            cutoff: 2000.0,
            resonance: 0.5,
            gain: 1.0,
            db: FilterDb::Db6,
        }
    }

    pub fn low_pass() -> Self {
        Self::new(FilterMode::LowPass)
    }

    pub fn high_pass() -> Self {
        Self::new(FilterMode::HighPass)
    }

    pub fn band_pass() -> Self {
        Self::new(FilterMode::BandPass)
    }

    pub fn band_limit() -> Self {
        Self::new(FilterMode::BandLimit)
    }

    pub fn notch() -> Self {
        Self::new(FilterMode::Notch)
    }

    pub fn get_mode(&self) -> FilterMode {
        self.mode
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn get_cutoff(&self) -> f32 {
        self.cutoff
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }

    pub fn get_resonance(&self) -> f32 {
        self.resonance
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    pub fn set_db(&mut self, db: FilterDb) {
        self.db = db;
    }

    pub fn get_db(&self) -> FilterDb {
        self.db
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectLimiter {
    threshold: f32,
    ceiling: f32,
    soft_clip: f32,
    soft_clip_ratio: f32,
}

impl Default for AudioEffectLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectLimiter {
    pub fn new() -> Self {
        Self {
            threshold: 0.0,
            ceiling: -0.1,
            soft_clip: 2.0,
            soft_clip_ratio: 10.0,
        }
    }

    pub fn set_threshold_db(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn get_threshold_db(&self) -> f32 {
        self.threshold
    }

    pub fn set_ceiling_db(&mut self, ceiling: f32) {
        self.ceiling = ceiling;
    }

    pub fn get_ceiling_db(&self) -> f32 {
        self.ceiling
    }

    pub fn set_soft_clip_db(&mut self, soft_clip: f32) {
        self.soft_clip = soft_clip;
    }

    pub fn get_soft_clip_db(&self) -> f32 {
        self.soft_clip
    }

    pub fn set_soft_clip_ratio(&mut self, ratio: f32) {
        self.soft_clip_ratio = ratio;
    }

    pub fn get_soft_clip_ratio(&self) -> f32 {
        self.soft_clip_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_defaults_match_the_constructor() {
        for filter in [
            AudioEffectFilter::low_pass(),
            AudioEffectFilter::high_pass(),
            AudioEffectFilter::band_pass(),
            AudioEffectFilter::band_limit(),
            AudioEffectFilter::notch(),
        ] {
            assert_eq!(filter.get_cutoff(), 2000.0);
            assert_eq!(filter.get_resonance(), 0.5);
            assert_eq!(filter.get_gain(), 1.0);
            assert_eq!(filter.get_db(), FilterDb::Db6);
        }
        assert_eq!(AudioEffectFilter::low_pass().get_mode(), FilterMode::LowPass);
        assert_eq!(AudioEffectFilter::high_pass().get_mode(), FilterMode::HighPass);
        assert_eq!(AudioEffectFilter::band_pass().get_mode(), FilterMode::BandPass);
        assert_eq!(AudioEffectFilter::band_limit().get_mode(), FilterMode::BandLimit);
        assert_eq!(AudioEffectFilter::notch().get_mode(), FilterMode::Notch);
    }

    #[test]
    fn filter_setters_do_not_clamp() {
        let mut f = AudioEffectFilter::low_pass();
        f.set_cutoff(1.0);
        f.set_resonance(-2.0);
        f.set_gain(4.0);
        f.set_db(FilterDb::Db24);
        assert_eq!(f.get_cutoff(), 1.0);
        assert_eq!(f.get_resonance(), -2.0);
        assert_eq!(f.get_gain(), 4.0);
        assert_eq!(f.get_db(), FilterDb::Db24);
    }

    #[test]
    fn filter_db_values_follow_declaration_order() {
        assert_eq!(FilterDb::Db6 as i32, 0);
        assert_eq!(FilterDb::Db12 as i32, 1);
        assert_eq!(FilterDb::Db18 as i32, 2);
        assert_eq!(FilterDb::Db24 as i32, 3);
    }

    #[test]
    fn limiter_defaults_and_setters() {
        let mut l = AudioEffectLimiter::new();
        assert_eq!(l.get_threshold_db(), 0.0);
        assert_eq!(l.get_ceiling_db(), -0.1);
        assert_eq!(l.get_soft_clip_db(), 2.0);
        assert_eq!(l.get_soft_clip_ratio(), 10.0);

        l.set_threshold_db(-12.0);
        l.set_ceiling_db(-3.0);
        l.set_soft_clip_db(5.0);
        l.set_soft_clip_ratio(20.0);
        // No clamping, like the setters in the C++.
        assert_eq!(l.get_threshold_db(), -12.0);
        assert_eq!(l.get_ceiling_db(), -3.0);
        assert_eq!(l.get_soft_clip_db(), 5.0);
        assert_eq!(l.get_soft_clip_ratio(), 20.0);
    }
}
