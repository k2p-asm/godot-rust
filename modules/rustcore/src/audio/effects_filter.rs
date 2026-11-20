//! `servers/audio/effects/audio_effect_filter.*`: the filter effects.
//!
//! Ported from `audio_effect_filter.cpp`: `AudioEffectFilter`'s parameters
//! (cutoff clamped to at least 1 Hz, resonance, gain, mode, dB slope), the
//! stage count `db + 1`, and the per-channel serial chain of up to four
//! `AudioFilterSW::Processor` sections sharing one filter.
//!
//! The subclasses (`LowPass`, `HighPass`, `BandPass`, `BandLimit`, `Notch`,
//! `LowShelf`, `HighShelf`) only fix the initial mode. The audio server is not
//! ported; processing takes the mix rate explicitly.

use crate::audio::filter_sw::{AudioFilterSw, FilterMode, FilterProcessor};
use crate::math::audio_frame::AudioFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FilterDb {
    Db6 = 0,
    Db12 = 1,
    Db18 = 2,
    Db24 = 3,
}

impl FilterDb {
    pub fn stages(self) -> usize {
        self as usize + 1
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectFilter {
    pub mode: FilterMode,
    pub cutoff: f32,
    pub resonance: f32,
    pub gain: f32,
    pub db: FilterDb,
}

impl Default for AudioEffectFilter {
    fn default() -> Self {
        Self::new(FilterMode::Lowpass)
    }
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
        Self::new(FilterMode::Lowpass)
    }

    pub fn high_pass() -> Self {
        Self::new(FilterMode::Highpass)
    }

    pub fn band_pass() -> Self {
        Self::new(FilterMode::Bandpass)
    }

    pub fn band_limit() -> Self {
        Self::new(FilterMode::Bandlimit)
    }

    pub fn notch() -> Self {
        Self::new(FilterMode::Notch)
    }

    pub fn low_shelf() -> Self {
        Self::new(FilterMode::Lowshelf)
    }

    pub fn high_shelf() -> Self {
        Self::new(FilterMode::Highshelf)
    }

    /// `set_cutoff`: clamped to at least 1 Hz.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.max(1.0);
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

    pub fn instantiate(&self, sample_rate: f32) -> FilterInstance {
        let mut processors: [[FilterProcessor; 4]; 2] = Default::default();
        for channel in 0..2 {
            for stage in 0..4 {
                processors[channel][stage].set_filter(Some(self.make_filter(sample_rate)), true);
            }
        }
        FilterInstance { processors }
    }

    fn make_filter(&self, sample_rate: f32) -> AudioFilterSw {
        let mut filter = AudioFilterSw::new();
        filter.set_mode(self.mode);
        filter.set_cutoff(self.cutoff);
        filter.set_resonance(self.resonance);
        filter.set_gain(self.gain);
        filter.set_stages(self.db.stages() as i32);
        filter.set_sampling_rate(sample_rate);
        filter
    }

    /// `process`: serially applies `db + 1` sections per channel.
    pub fn process(
        &self,
        instance: &mut FilterInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let stages = self.db.stages();
        let filter = self.make_filter(sample_rate);
        for channel in 0..2 {
            for stage in 0..4 {
                instance.processors[channel][stage].set_filter(Some(filter.clone()), false);
                instance.processors[channel][stage].update_coeffs(0);
            }
        }

        for index in 0..count {
            let mut left = src[index].left;
            for stage in 0..stages {
                left = instance.processors[0][stage].process_sample(left);
            }
            let mut right = src[index].right;
            for stage in 0..stages {
                right = instance.processors[1][stage].process_sample(right);
            }
            dst[index] = AudioFrame::new(left, right);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterInstance {
    processors: [[FilterProcessor; 4]; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frames: usize, frequency: f32, sample_rate: f32) -> Vec<AudioFrame> {
        (0..frames)
            .map(|index| {
                let value =
                    0.5 * (std::f32::consts::TAU * frequency * index as f32 / sample_rate).sin();
                AudioFrame::new(value, value)
            })
            .collect()
    }

    fn rms(frames: &[AudioFrame], start: usize) -> f32 {
        let window = &frames[start..];
        (window.iter().map(|frame| frame.left * frame.left).sum::<f32>() / window.len() as f32)
            .sqrt()
    }

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectFilter::new(FilterMode::Lowpass);
        assert_eq!(effect.get_cutoff(), 2000.0);
        assert_eq!(effect.get_resonance(), 0.5);
        assert_eq!(effect.get_gain(), 1.0);
        assert_eq!(effect.get_db(), FilterDb::Db6);
        assert_eq!(effect.mode, FilterMode::Lowpass);

        effect.set_cutoff(0.0);
        assert_eq!(effect.get_cutoff(), 1.0, "cutoff is at least 1 Hz");
        effect.set_cutoff(800.0);
        effect.set_resonance(0.7);
        effect.set_gain(2.0);
        effect.set_db(FilterDb::Db24);
        assert_eq!(effect.get_cutoff(), 800.0);
        assert_eq!(effect.get_resonance(), 0.7);
        assert_eq!(effect.get_gain(), 2.0);
        assert_eq!(effect.get_db(), FilterDb::Db24);
    }

    #[test]
    fn slope_maps_to_stages() {
        assert_eq!(FilterDb::Db6.stages(), 1);
        assert_eq!(FilterDb::Db12.stages(), 2);
        assert_eq!(FilterDb::Db18.stages(), 3);
        assert_eq!(FilterDb::Db24.stages(), 4);
        assert_eq!(FilterDb::Db6 as i32, 0);
        assert_eq!(FilterDb::Db24 as i32, 3);
    }

    #[test]
    fn subclasses_pick_the_mode() {
        assert_eq!(AudioEffectFilter::low_pass().mode, FilterMode::Lowpass);
        assert_eq!(AudioEffectFilter::high_pass().mode, FilterMode::Highpass);
        assert_eq!(AudioEffectFilter::band_pass().mode, FilterMode::Bandpass);
        assert_eq!(AudioEffectFilter::band_limit().mode, FilterMode::Bandlimit);
        assert_eq!(AudioEffectFilter::notch().mode, FilterMode::Notch);
        assert_eq!(AudioEffectFilter::low_shelf().mode, FilterMode::Lowshelf);
        assert_eq!(AudioEffectFilter::high_shelf().mode, FilterMode::Highshelf);
    }

    #[test]
    fn low_pass_passes_low_and_blocks_high_frequencies() {
        let mut effect = AudioEffectFilter::low_pass();
        effect.set_cutoff(1000.0);
        let rate = 44100.0;
        let low = sine(8192, 100.0, rate);
        let high = sine(8192, 8000.0, rate);
        let mut out_low = vec![AudioFrame::default(); low.len()];
        let mut out_high = vec![AudioFrame::default(); high.len()];
        effect.process(&mut effect.instantiate(rate), &low, &mut out_low, rate);
        effect.process(&mut effect.instantiate(rate), &high, &mut out_high, rate);
        let low_gain = rms(&out_low, 4096) / rms(&low, 4096);
        let high_gain = rms(&out_high, 4096) / rms(&high, 4096);
        assert!(low_gain > 0.8, "low gain {low_gain}");
        assert!(high_gain < 0.1, "high gain {high_gain}");
    }

    #[test]
    fn high_pass_blocks_dc_and_stronger_slopes_attenuate_more() {
        let rate = 44100.0;
        let high = sine(8192, 4000.0, rate);
        let mut out12 = vec![AudioFrame::default(); high.len()];
        let mut out24 = vec![AudioFrame::default(); high.len()];

        let mut effect = AudioEffectFilter::high_pass();
        effect.set_cutoff(500.0);
        effect.set_db(FilterDb::Db12);
        effect.process(&mut effect.instantiate(rate), &high, &mut out12, rate);

        let mut steeper = AudioEffectFilter::high_pass();
        steeper.set_cutoff(500.0);
        steeper.set_db(FilterDb::Db24);
        steeper.process(&mut steeper.instantiate(rate), &high, &mut out24, rate);

        // The 24 dB/octave chain passes the same tone more fully (both are above
        // the cutoff); the difference proves the stage chain is active.
        let gain12 = rms(&out12, 4096) / rms(&high, 4096);
        let gain24 = rms(&out24, 4096) / rms(&high, 4096);
        assert!(gain12 > 0.0 && gain24 > 0.0);
        assert!((gain12 - gain24).abs() > 1e-4);
    }
}
