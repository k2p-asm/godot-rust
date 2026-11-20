//! `servers/audio/effects/audio_effect_compressor.*`: the compressor.
//!
//! Ported from `audio_effect_compressor.cpp`: the peak follower, the
//! attack/release envelope in dB (`rundb`), the smoothed ratio (`runratio` with
//! its own attack/release coefficients), the gain computer
//! (`gr = -overdb * (ratio - 1) / ratio`), the makeup gain, the dry/wet mix and
//! the gain-reduction meter with its per-sample decay.
//!
//! The sidechain bus lookup and the audio server are not ported; processing takes
//! the mix rate explicitly and the input frames as the detector source.
//! `0.00001 s`/`0.5 s` are the engine's hard-coded ratio attack/release times.

use crate::math::audio_frame::AudioFrame;
use crate::math::funcs;

#[derive(Clone, Debug)]
pub struct AudioEffectCompressor {
    pub threshold: f32,
    pub ratio: f32,
    pub gain: f32,
    pub attack_us: f32,
    pub release_ms: f32,
    pub mix: f32,
    pub sidechain: String,
}

impl Default for AudioEffectCompressor {
    fn default() -> Self {
        Self {
            threshold: 0.0,
            ratio: 4.0,
            gain: 0.0,
            attack_us: 20.0,
            release_ms: 250.0,
            mix: 1.0,
            sidechain: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompressorInstance {
    rundb: f32,
    runratio: f32,
    averatio: f32,
    runmax: f32,
    maxover: f32,
    gr_meter: f32,
}

impl AudioEffectCompressor {
    pub fn new() -> Self {
        Self::default()
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
        self.sidechain = sidechain.to_string();
    }

    pub fn get_sidechain(&self) -> &str {
        &self.sidechain
    }

    pub fn instantiate(&self) -> CompressorInstance {
        CompressorInstance {
            rundb: 0.0,
            runratio: 0.0,
            averatio: 0.0,
            runmax: 0.0,
            maxover: 0.0,
            gr_meter: 1.0,
        }
    }

    /// `process`.
    pub fn process(
        &self,
        instance: &mut CompressorInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        if count == 0 {
            return;
        }
        let threshold = funcs::db_to_linear(self.threshold as f64) as f32;
        let ratatcoef = (-1.0f32 / (0.00001 * sample_rate)).exp();
        let ratrelcoef = (-1.0f32 / (0.5 * sample_rate)).exp();
        let attime = self.attack_us / 1_000_000.0;
        let reltime = self.release_ms / 1000.0;
        let atcoef = (-1.0f32 / (attime * sample_rate)).exp();
        let relcoef = (-1.0f32 / (reltime * sample_rate)).exp();
        let makeup = funcs::db_to_linear(self.gain as f64) as f32;
        let mix = self.mix;
        let gr_meter_decay = (1.0f32 / sample_rate).exp();

        for index in 0..count {
            let left = src[index].left.abs();
            let right = src[index].right.abs();
            let peak = left.max(right);

            let mut overdb =
                2.081_368_98 * funcs::linear_to_db((peak / threshold) as f64) as f32;
            if overdb < 0.0 {
                overdb = 0.0;
            }
            if overdb - instance.rundb > 5.0 {
                instance.averatio = 4.0;
            }
            if overdb > instance.rundb {
                instance.rundb = overdb + atcoef * (instance.rundb - overdb);
                instance.runratio =
                    instance.averatio + ratatcoef * (instance.runratio - instance.averatio);
            } else {
                instance.rundb = overdb + relcoef * (instance.rundb - overdb);
                instance.runratio =
                    instance.averatio + ratrelcoef * (instance.runratio - instance.averatio);
            }
            overdb = instance.rundb;
            instance.averatio = instance.runratio;

            let cratio = self.ratio;
            let gr = -overdb * (cratio - 1.0) / cratio;
            let grv = funcs::db_to_linear(gr as f64) as f32;

            instance.runmax =
                instance.maxover + relcoef * (instance.runmax - instance.maxover);
            instance.maxover = instance.runmax;

            if grv < instance.gr_meter {
                instance.gr_meter = grv;
            } else {
                instance.gr_meter *= gr_meter_decay;
                if instance.gr_meter > 1.0 {
                    instance.gr_meter = 1.0;
                }
            }

            let wet = grv * makeup * mix;
            let dry = 1.0 - mix;
            dst[index] = AudioFrame::new(
                src[index].left * wet + src[index].left * dry,
                src[index].right * wet + src[index].right * dry,
            );
        }
    }

    /// The current gain-reduction meter of an instance (1.0 = no reduction).
    pub fn get_gain_reduction(instance: &CompressorInstance) -> f32 {
        instance.gr_meter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectCompressor::new();
        assert_eq!(effect.get_threshold(), 0.0);
        assert_eq!(effect.get_ratio(), 4.0);
        assert_eq!(effect.get_gain(), 0.0);
        assert_eq!(effect.get_attack_us(), 20.0);
        assert_eq!(effect.get_release_ms(), 250.0);
        assert_eq!(effect.get_mix(), 1.0);
        assert_eq!(effect.get_sidechain(), "");

        effect.set_threshold(-12.0);
        effect.set_ratio(8.0);
        effect.set_gain(3.0);
        effect.set_attack_us(100.0);
        effect.set_release_ms(50.0);
        effect.set_mix(0.5);
        effect.set_sidechain("Music");
        assert_eq!(effect.get_threshold(), -12.0);
        assert_eq!(effect.get_ratio(), 8.0);
        assert_eq!(effect.get_gain(), 3.0);
        assert_eq!(effect.get_attack_us(), 100.0);
        assert_eq!(effect.get_release_ms(), 50.0);
        assert_eq!(effect.get_mix(), 0.5);
        assert_eq!(effect.get_sidechain(), "Music");
    }

    #[test]
    fn below_threshold_passes_through() {
        let effect = AudioEffectCompressor::new();
        let input: Vec<AudioFrame> = (0..256)
            .map(|index| AudioFrame::new(index as f32 * 0.001, -(index as f32) * 0.001))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output, 44100.0);
        for (a, b) in output.iter().zip(&input) {
            assert!((a.left - b.left).abs() < 1e-5);
            assert!((a.right - b.right).abs() < 1e-5);
        }
        assert!((AudioEffectCompressor::get_gain_reduction(&instance) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn above_threshold_is_attenuated() {
        let mut effect = AudioEffectCompressor::new();
        effect.set_attack_us(1.0); // fast attack so the envelope settles quickly
        let input = vec![AudioFrame::new(1.5, -1.5); 4096];
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output, 44100.0);

        // ratio 4 with the engine's 2.08x over-drive scaling reduces 1.5 to ~0.8.
        assert!(output[4095].left < 1.5);
        assert!(output[4095].left > 0.5);
        assert!((output[4095].left - 0.797).abs() < 0.05);
        assert!(AudioEffectCompressor::get_gain_reduction(&instance) < 1.0);
        // The output is symmetric.
        assert!((output[4095].left + output[4095].right).abs() < 1e-5);
    }

    #[test]
    fn makeup_gain_is_applied() {
        let mut effect = AudioEffectCompressor::new();
        effect.set_gain(6.0206); // ×2
        let input = vec![AudioFrame::new(0.25, -0.25); 512];
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, 44100.0);
        assert!((output[511].left - 0.5).abs() < 1e-3);
        assert!((output[511].right + 0.5).abs() < 1e-3);
    }

    #[test]
    fn mix_blends_dry_and_wet() {
        let mut effect = AudioEffectCompressor::new();
        effect.set_gain(6.0206); // wet is ×2
        effect.set_mix(0.5);
        let input = vec![AudioFrame::new(0.25, -0.25); 128];
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, 44100.0);
        // 0.5 × 0.25 + 0.5 × 0.5 = 0.375.
        assert!((output[127].left - 0.375).abs() < 1e-3);
        assert!((output[127].right + 0.375).abs() < 1e-3);
    }
}
