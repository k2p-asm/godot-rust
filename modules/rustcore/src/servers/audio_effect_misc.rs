//! `servers/audio/effects/`: parameter state for five more effects.
//!
//! Every default here is copied from the class's constructor or from the header's
//! in-class initialiser, and the validation mirrors the C++ setters exactly:
//!
//! - `AudioEffectAmplify`: `volume_db = 0`; `volume_linear` converts with
//!   `linear_to_db`/`db_to_linear`.
//! - `AudioEffectPanner`: `pan = 0`.
//! - `AudioEffectStereoEnhance`: `0 dB`, `pan_pullout 1`, `time_pullout 0`,
//!   `surround 0`. The engine exposes no setter for `volume_db`.
//! - `AudioEffectCompressor`: `0 / 4 / 0 / 20 us / 250 ms / 1`.
//! - `AudioEffectPitchShift`: in-class initialisers, `set_pitch_scale` rejects
//!   non-positive values, `set_oversampling` rejects `< 4`, `set_fft_size`
//!   rejects an out-of-range enum.
//!
//! No DSP is ported.

use crate::math::funcs::{db_to_linear, linear_to_db};
use crate::string::string_name::StringName;

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectAmplify {
    volume_db: f32,
}

impl Default for AudioEffectAmplify {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectAmplify {
    pub fn new() -> Self {
        Self { volume_db: 0.0 }
    }

    pub fn set_volume_db(&mut self, volume_db: f32) {
        self.volume_db = volume_db;
    }

    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }

    /// `set_volume_linear` goes through `linear_to_db`.
    pub fn set_volume_linear(&mut self, volume: f32) {
        self.volume_db = linear_to_db(volume);
    }

    pub fn get_volume_linear(&self) -> f32 {
        db_to_linear(self.volume_db)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioEffectPanner {
    pan: f32,
}

impl AudioEffectPanner {
    pub fn new() -> Self {
        Self { pan: 0.0 }
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan;
    }

    pub fn get_pan(&self) -> f32 {
        self.pan
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectStereoEnhance {
    volume_db: f32,
    pan_pullout: f32,
    time_pullout: f32,
    surround: f32,
}

impl Default for AudioEffectStereoEnhance {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectStereoEnhance {
    pub fn new() -> Self {
        Self {
            volume_db: 0.0,
            pan_pullout: 1.0,
            time_pullout: 0.0,
            surround: 0.0,
        }
    }

    /// Read-only: the class has no `set_volume_db`.
    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }

    pub fn set_pan_pullout(&mut self, amount: f32) {
        self.pan_pullout = amount;
    }

    pub fn get_pan_pullout(&self) -> f32 {
        self.pan_pullout
    }

    pub fn set_time_pullout(&mut self, amount: f32) {
        self.time_pullout = amount;
    }

    pub fn get_time_pullout(&self) -> f32 {
        self.time_pullout
    }

    pub fn set_surround(&mut self, amount: f32) {
        self.surround = amount;
    }

    pub fn get_surround(&self) -> f32 {
        self.surround
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectCompressor {
    threshold: f32,
    ratio: f32,
    gain: f32,
    attack_us: f32,
    release_ms: f32,
    mix: f32,
    sidechain: StringName,
}

impl Default for AudioEffectCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectCompressor {
    pub fn new() -> Self {
        Self {
            threshold: 0.0,
            ratio: 4.0,
            gain: 0.0,
            attack_us: 20.0,
            release_ms: 250.0,
            mix: 1.0,
            sidechain: StringName::new(""),
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
    }

    pub fn get_ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    pub fn set_attack_us(&mut self, attack_us: f32) {
        self.attack_us = attack_us;
    }

    pub fn get_attack_us(&self) -> f32 {
        self.attack_us
    }

    pub fn set_release_ms(&mut self, release_ms: f32) {
        self.release_ms = release_ms;
    }

    pub fn get_release_ms(&self) -> f32 {
        self.release_ms
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn set_sidechain(&mut self, sidechain: &str) {
        self.sidechain = StringName::new(sidechain);
    }

    pub fn get_sidechain(&self) -> &StringName {
        &self.sidechain
    }
}

/// `AudioEffectPitchShift::FFTSize`; `Max` is `FFT_SIZE_MAX` and is not a valid
/// value for `set_fft_size`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FftSize {
    Fft256,
    Fft512,
    Fft1024,
    Fft2048,
    Fft4096,
    Max,
}

impl FftSize {
    /// The sample counts the engine uses in `instantiate`.
    pub fn samples(self) -> Option<i32> {
        match self {
            FftSize::Fft256 => Some(256),
            FftSize::Fft512 => Some(512),
            FftSize::Fft1024 => Some(1024),
            FftSize::Fft2048 => Some(2048),
            FftSize::Fft4096 => Some(4096),
            FftSize::Max => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectPitchShift {
    pitch_scale: f32,
    oversampling: i32,
    fft_size: FftSize,
    wet: f32,
    dry: f32,
    filter: bool,
}

impl Default for AudioEffectPitchShift {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectPitchShift {
    pub fn new() -> Self {
        Self {
            pitch_scale: 1.0,
            oversampling: 4,
            fft_size: FftSize::Fft2048,
            wet: 0.0,
            dry: 0.0,
            filter: false,
        }
    }

    /// `ERR_FAIL_COND(!(p_pitch_scale > 0.0))`: rejects zero, negatives and NaN,
    /// keeping the previous value.
    pub fn set_pitch_scale(&mut self, pitch_scale: f32) -> bool {
        if !(pitch_scale > 0.0) {
            return false;
        }
        self.pitch_scale = pitch_scale;
        true
    }

    pub fn get_pitch_scale(&self) -> f32 {
        self.pitch_scale
    }

    /// `ERR_FAIL_COND(p_oversampling < 4)`.
    pub fn set_oversampling(&mut self, oversampling: i32) -> bool {
        if oversampling < 4 {
            return false;
        }
        self.oversampling = oversampling;
        true
    }

    pub fn get_oversampling(&self) -> i32 {
        self.oversampling
    }

    /// `ERR_FAIL_INDEX(p_fft_size, FFT_SIZE_MAX)`.
    pub fn set_fft_size(&mut self, fft_size: FftSize) -> bool {
        if fft_size == FftSize::Max {
            return false;
        }
        self.fft_size = fft_size;
        true
    }

    pub fn get_fft_size(&self) -> FftSize {
        self.fft_size
    }

    /// No setter exists in the engine for these two fields.
    pub fn get_wet(&self) -> f32 {
        self.wet
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }

    /// `filter` has no accessor in the engine either; exposed for inspection.
    pub fn is_filter_enabled(&self) -> bool {
        self.filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amplify_defaults_and_linear_conversion() {
        let mut a = AudioEffectAmplify::new();
        assert_eq!(a.get_volume_db(), 0.0);
        assert!((a.get_volume_linear() - 1.0).abs() < 1e-5);

        // +6.0206 dB is twice the amplitude.
        a.set_volume_db(6.0205999);
        assert!((a.get_volume_linear() - 2.0).abs() < 1e-4, "{}", a.get_volume_linear());

        // And the inverse path.
        a.set_volume_linear(0.5);
        assert!((a.get_volume_db() + 6.0205999).abs() < 1e-4, "{}", a.get_volume_db());
        assert!((a.get_volume_linear() - 0.5).abs() < 1e-5);

        // No clamping.
        a.set_volume_db(-200.0);
        assert_eq!(a.get_volume_db(), -200.0);
    }

    #[test]
    fn panner_defaults_to_centre() {
        let mut p = AudioEffectPanner::new();
        assert_eq!(p.get_pan(), 0.0);
        p.set_pan(-0.75);
        assert_eq!(p.get_pan(), -0.75);
        // Out-of-range pans are stored as given.
        p.set_pan(3.0);
        assert_eq!(p.get_pan(), 3.0);
    }

    #[test]
    fn stereo_enhance_defaults() {
        let mut s = AudioEffectStereoEnhance::new();
        assert_eq!(s.get_volume_db(), 0.0);
        assert_eq!(s.get_pan_pullout(), 1.0);
        assert_eq!(s.get_time_pullout(), 0.0);
        assert_eq!(s.get_surround(), 0.0);

        s.set_pan_pullout(2.0);
        s.set_time_pullout(-1.0);
        s.set_surround(5.0);
        assert_eq!(s.get_pan_pullout(), 2.0);
        assert_eq!(s.get_time_pullout(), -1.0);
        assert_eq!(s.get_surround(), 5.0);
    }

    #[test]
    fn compressor_defaults_and_setters() {
        let mut c = AudioEffectCompressor::new();
        assert_eq!(c.get_threshold(), 0.0);
        assert_eq!(c.get_ratio(), 4.0);
        assert_eq!(c.get_gain(), 0.0);
        assert_eq!(c.get_attack_us(), 20.0);
        assert_eq!(c.get_release_ms(), 250.0);
        assert_eq!(c.get_mix(), 1.0);
        assert_eq!(c.get_sidechain().as_string(), "");

        c.set_threshold(-18.0);
        c.set_ratio(2.5);
        c.set_gain(6.0);
        c.set_attack_us(100.0);
        c.set_release_ms(50.0);
        c.set_mix(0.0);
        c.set_sidechain("Voice");
        assert_eq!(c.get_threshold(), -18.0);
        assert_eq!(c.get_ratio(), 2.5);
        assert_eq!(c.get_gain(), 6.0);
        assert_eq!(c.get_attack_us(), 100.0);
        assert_eq!(c.get_release_ms(), 50.0);
        assert_eq!(c.get_mix(), 0.0);
        assert_eq!(c.get_sidechain().as_string(), "Voice");
    }

    #[test]
    fn pitch_shift_defaults() {
        let p = AudioEffectPitchShift::new();
        assert_eq!(p.get_pitch_scale(), 1.0);
        assert_eq!(p.get_oversampling(), 4);
        assert_eq!(p.get_fft_size(), FftSize::Fft2048);
        assert_eq!(p.get_wet(), 0.0);
        assert_eq!(p.get_dry(), 0.0);
        assert!(!p.is_filter_enabled());
        assert_eq!(p.get_fft_size().samples(), Some(2048));
    }

    #[test]
    fn pitch_scale_rejects_non_positive_values() {
        let mut p = AudioEffectPitchShift::new();
        assert!(!p.set_pitch_scale(0.0));
        assert!(!p.set_pitch_scale(-2.0));
        assert!(!p.set_pitch_scale(f32::NAN));
        assert_eq!(p.get_pitch_scale(), 1.0);

        assert!(p.set_pitch_scale(0.5));
        assert_eq!(p.get_pitch_scale(), 0.5);
        // No upper clamp.
        assert!(p.set_pitch_scale(64.0));
        assert_eq!(p.get_pitch_scale(), 64.0);
    }

    #[test]
    fn oversampling_and_fft_size_validation() {
        let mut p = AudioEffectPitchShift::new();
        assert!(!p.set_oversampling(3));
        assert_eq!(p.get_oversampling(), 4);
        assert!(p.set_oversampling(4));
        assert!(p.set_oversampling(32));
        assert_eq!(p.get_oversampling(), 32);

        assert!(!p.set_fft_size(FftSize::Max));
        assert_eq!(p.get_fft_size(), FftSize::Fft2048);
        assert!(p.set_fft_size(FftSize::Fft256));
        assert_eq!(p.get_fft_size(), FftSize::Fft256);
    }

    #[test]
    fn fft_enum_values_follow_declaration_order() {
        assert_eq!(FftSize::Fft256 as i32, 0);
        assert_eq!(FftSize::Fft512 as i32, 1);
        assert_eq!(FftSize::Fft1024 as i32, 2);
        assert_eq!(FftSize::Fft2048 as i32, 3);
        assert_eq!(FftSize::Fft4096 as i32, 4);
        assert_eq!(FftSize::Max as i32, 5);
        assert_eq!(FftSize::Max.samples(), None);
    }
}
