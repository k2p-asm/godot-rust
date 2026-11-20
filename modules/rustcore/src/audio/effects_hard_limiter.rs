//! `servers/audio/effects/audio_effect_hard_limiter.*`: the look-ahead limiter.
//!
//! Ported from `audio_effect_hard_limiter.cpp`: the pre-gain, ceiling, release
//! and fixed 2 ms attack/sustain windows, the delayed sample ring (the latency
//! introduces the look-ahead), the release/attack gain interpolation, and the
//! per-bucket minimum gain across the stored window (the engine takes the minimum
//! over every bucket each sample).
//!
//! The audio server is not ported; processing takes the mix rate explicitly.

use crate::math::audio_frame::AudioFrame;
use crate::math::funcs;

#[derive(Clone, Debug)]
pub struct AudioEffectHardLimiter {
    pub pre_gain: f32,
    pub ceiling: f32,
    pub sustain: f32,
    pub release: f32,
    /// `const float attack = 0.002`.
    pub attack: f32,
}

impl Default for AudioEffectHardLimiter {
    fn default() -> Self {
        Self {
            pre_gain: 0.0,
            ceiling: -0.3,
            sustain: 0.02,
            release: 0.1,
            attack: 0.002,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HardLimiterInstance {
    sample_cursor: usize,
    release_factor: f32,
    attack_factor: f32,
    gain: f32,
    gain_target: f32,
    sample_buffer_left: Vec<f32>,
    sample_buffer_right: Vec<f32>,
    gain_samples_to_store: usize,
    gain_bucket_cursor: usize,
    gain_bucket_size: usize,
    gain_buckets: Vec<f32>,
}

impl AudioEffectHardLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_ceiling_db(&mut self, ceiling: f32) {
        self.ceiling = ceiling;
    }

    pub fn get_ceiling_db(&self) -> f32 {
        self.ceiling
    }

    pub fn set_release(&mut self, release: f32) {
        self.release = release;
    }

    pub fn get_release(&self) -> f32 {
        self.release
    }

    pub fn set_pre_gain_db(&mut self, pre_gain: f32) {
        self.pre_gain = pre_gain;
    }

    pub fn get_pre_gain_db(&self) -> f32 {
        self.pre_gain
    }

    /// `instantiate`: buffers sized from the mix rate.
    pub fn instantiate(&self, mix_rate: f32) -> HardLimiterInstance {
        let sample_len = (mix_rate * self.attack).ceil() as usize + 1;
        let samples_to_store = (mix_rate * (self.attack + self.sustain) + 1.0).ceil() as usize;
        let bucket_size = (mix_rate * self.attack) as usize;
        let mut gain_buckets = Vec::new();
        let mut index = 0usize;
        while index < samples_to_store {
            gain_buckets.push(1.0);
            index += bucket_size.max(1);
        }
        HardLimiterInstance {
            sample_cursor: 0,
            release_factor: 0.0,
            attack_factor: 0.0,
            gain: 1.0,
            gain_target: 1.0,
            sample_buffer_left: vec![0.0; sample_len],
            sample_buffer_right: vec![0.0; sample_len],
            gain_samples_to_store: samples_to_store,
            gain_bucket_cursor: 0,
            gain_bucket_size: bucket_size,
            gain_buckets,
        }
    }

    /// `process`.
    pub fn process(
        &self,
        instance: &mut HardLimiterInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let ceiling = funcs::db_to_linear(self.ceiling as f64) as f32;
        let release = self.release;
        let attack = self.attack;
        let pre_gain = funcs::db_to_linear(self.pre_gain as f64) as f32;
        let bucket_size = instance.gain_bucket_size.max(1);
        let samples_to_store = instance.gain_samples_to_store.max(1);

        for index in 0..count {
            let mut sample_left = src[index].left * pre_gain;
            let mut sample_right = src[index].right * pre_gain;

            let largest_sample = sample_left.abs().max(sample_right.abs());

            instance.release_factor = (instance.release_factor - 1.0 / sample_rate).max(0.0);
            instance.release_factor = instance.release_factor.min(release);

            if instance.release_factor > 0.0 {
                instance.gain = lerp(
                    instance.gain_target,
                    1.0,
                    1.0 - instance.release_factor / release,
                );
            }

            if largest_sample * instance.gain > ceiling {
                instance.gain_target = ceiling / largest_sample;
                instance.release_factor = release;
                instance.attack_factor = attack;
            }

            instance.attack_factor = (instance.attack_factor - 1.0 / sample_rate).max(0.0);
            if instance.attack_factor > 0.0 {
                instance.gain = lerp(
                    instance.gain_target,
                    instance.gain,
                    1.0 - instance.attack_factor / attack,
                );
            }

            let bucket_id = instance.gain_bucket_cursor / bucket_size;
            if instance.gain_bucket_cursor % bucket_size == 0 {
                instance.gain_buckets[bucket_id] = 1.0;
            }
            instance.gain_buckets[bucket_id] =
                instance.gain_buckets[bucket_id].min(instance.gain);
            instance.gain_bucket_cursor = (instance.gain_bucket_cursor + 1) % samples_to_store;

            for bucket in &instance.gain_buckets {
                instance.gain = instance.gain.min(*bucket);
            }

            let dst_left = instance.sample_buffer_left[instance.sample_cursor];
            let dst_right = instance.sample_buffer_right[instance.sample_cursor];
            instance.sample_buffer_left[instance.sample_cursor] = sample_left;
            instance.sample_buffer_right[instance.sample_cursor] = sample_right;
            instance.sample_cursor =
                (instance.sample_cursor + 1) % instance.sample_buffer_left.len();

            // The locals are re-read for clarity; the engine writes them back.
            sample_left = dst_left;
            sample_right = dst_right;
            dst[index] = AudioFrame::new(
                sample_left * instance.gain,
                sample_right * instance.gain,
            );
        }
    }

    pub fn get_latency_frames(&self, mix_rate: f32) -> usize {
        (mix_rate * self.attack).ceil() as usize + 1
    }
}

fn lerp(from: f32, to: f32, weight: f32) -> f32 {
    from + (to - from) * weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectHardLimiter::new();
        assert_eq!(effect.get_ceiling_db(), -0.3);
        assert_eq!(effect.get_release(), 0.1);
        assert_eq!(effect.get_pre_gain_db(), 0.0);
        assert_eq!(effect.sustain, 0.02);
        assert_eq!(effect.attack, 0.002);

        effect.set_ceiling_db(-6.0);
        effect.set_release(0.25);
        effect.set_pre_gain_db(3.0);
        assert_eq!(effect.get_ceiling_db(), -6.0);
        assert_eq!(effect.get_release(), 0.25);
        assert_eq!(effect.get_pre_gain_db(), 3.0);
    }

    #[test]
    fn instance_sizes() {
        let effect = AudioEffectHardLimiter::new();
        let instance = effect.instantiate(44100.0);
        assert_eq!(instance.sample_buffer_left.len(), 90);
        assert_eq!(instance.sample_buffer_right.len(), 90);
        // ceil(44100 * 0.022 + 1) = 972; buckets every 88 samples.
        assert_eq!(instance.gain_samples_to_store, 972);
        assert_eq!(instance.gain_bucket_size, 88);
        assert_eq!(instance.gain_buckets.len(), 12);
        assert_eq!(effect.get_latency_frames(44100.0), 90);
    }

    #[test]
    fn silence_stays_silent() {
        let effect = AudioEffectHardLimiter::new();
        let input = vec![AudioFrame::default(); 1024];
        let mut output = vec![AudioFrame::default(); 1024];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        assert!(output.iter().all(|frame| frame.left == 0.0 && frame.right == 0.0));
    }

    #[test]
    fn over_ceiling_is_limited_after_the_latency() {
        let mut effect = AudioEffectHardLimiter::new();
        effect.set_ceiling_db(0.0);
        let rate = 44100.0;
        let input = vec![AudioFrame::new(2.0, -2.0); 4096];
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        // The first samples are silent: they are the look-ahead delay.
        assert_eq!(output[0], AudioFrame::new(0.0, 0.0));
        // After the attack window the output settles at the ceiling.
        let late = output[4095];
        assert!(late.left.abs() <= 1.0 + 1e-4, "limited {late:?}");
        assert!(late.left > 0.2, "signal survives {late:?}");
        assert!((late.left + late.right).abs() < 1e-4);
    }

    #[test]
    fn pre_gain_scales_into_the_limiter() {
        let mut quiet = AudioEffectHardLimiter::default();
        quiet.set_ceiling_db(0.0);
        let rate = 44100.0;
        let input = vec![AudioFrame::new(0.05, -0.05); 4096];
        let mut plain = vec![AudioFrame::default(); input.len()];
        quiet.process(&mut quiet.instantiate(rate), &input, &mut plain, rate);

        let mut boosted = AudioEffectHardLimiter::default();
        boosted.set_ceiling_db(0.0);
        boosted.set_pre_gain_db(20.0); // ×10
        let mut loud = vec![AudioFrame::default(); input.len()];
        boosted.process(&mut boosted.instantiate(rate), &input, &mut loud, rate);

        assert!(loud[4095].left.abs() > plain[4095].left.abs());
        assert!(loud[4095].left.abs() <= 1.0 + 1e-4);
    }
}
