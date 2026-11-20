//! `servers/audio/effects/audio_effect_stereo_enhance.*`: stereo widening.
//!
//! Ported from `audio_effect_stereo_enhance.cpp`: the pan pull-out (mixing
//! towards the centre), the optional surround mode (a delayed mid-signal added to
//! the left and subtracted from the right), and the non-surround mode (the right
//! channel is delayed while the left is untouched). The ring buffer is sized from
//! `MAX_DELAY_MS + 2` milliseconds, rounded up to a power of two exactly like the
//! engine.
//!
//! The effect resource binding and the audio server are not ported; processing
//! takes the mix rate explicitly.

use crate::math::audio_frame::AudioFrame;

pub const MAX_DELAY_MS: i32 = 50;

#[derive(Clone, Debug)]
pub struct AudioEffectStereoEnhance {
    pub pan_pullout: f32,
    pub time_pullout: f32,
    pub surround: f32,
}

impl Default for AudioEffectStereoEnhance {
    fn default() -> Self {
        Self {
            pan_pullout: 1.0,
            time_pullout: 0.0,
            surround: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StereoEnhanceInstance {
    delay_ringbuff: Vec<f32>,
    ringbuff_mask: usize,
    ringbuff_pos: usize,
}

impl AudioEffectStereoEnhance {
    pub fn new() -> Self {
        Self::default()
    }

    /// `instantiate`: the ring size rule is the engine's (`1 << bits` for the
    /// first power of two strictly greater than the requested length).
    pub fn instantiate(&self, sample_rate: f32) -> StereoEnhanceInstance {
        let mut ring_buffer_max_size = (MAX_DELAY_MS + 2) as f32;
        ring_buffer_max_size /= 1000.0;
        ring_buffer_max_size *= sample_rate;
        let mut ringbuff_size = ring_buffer_max_size as usize;
        let mut bits = 0u32;
        while ringbuff_size > 0 {
            bits += 1;
            ringbuff_size /= 2;
        }
        let ringbuff_size = 1usize << bits;
        StereoEnhanceInstance {
            delay_ringbuff: vec![0.0; ringbuff_size],
            ringbuff_mask: ringbuff_size - 1,
            ringbuff_pos: 0,
        }
    }

    pub fn set_pan_pullout(&mut self, amount: f32) {
        self.pan_pullout = amount;
    }

    pub fn get_pan_pullout(&self) -> f32 {
        self.pan_pullout
    }

    pub fn set_time_pullout(&mut self, ms: f32) {
        self.time_pullout = ms;
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

    /// `process`.
    pub fn process(
        &self,
        instance: &mut StereoEnhanceInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let intensity = self.pan_pullout;
        let surround_mode = self.surround > 0.0;
        let surround_amount = self.surround;
        let delay_frames = ((self.time_pullout / 1000.0) * sample_rate) as usize;
        let mask = instance.ringbuff_mask;

        for index in 0..count {
            let mut left = src[index].left;
            let mut right = src[index].right;
            let center = (left + right) / 2.0;
            left = center + (left - center) * intensity;
            right = center + (right - center) * intensity;

            if surround_mode {
                let value = (left + right) / 2.0;
                instance.delay_ringbuff[instance.ringbuff_pos & mask] = value;
                let out = instance.delay_ringbuff
                    [(instance.ringbuff_pos.wrapping_sub(delay_frames)) & mask]
                    * surround_amount;
                left += out;
                right += -out;
            } else {
                let value = right;
                instance.delay_ringbuff[instance.ringbuff_pos & mask] = value;
                right = instance.delay_ringbuff
                    [(instance.ringbuff_pos.wrapping_sub(delay_frames)) & mask];
            }

            dst[index] = AudioFrame::new(left, right);
            instance.ringbuff_pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectStereoEnhance::new();
        assert_eq!(effect.get_pan_pullout(), 1.0);
        assert_eq!(effect.get_time_pullout(), 0.0);
        assert_eq!(effect.get_surround(), 0.0);
        effect.set_pan_pullout(0.5);
        effect.set_time_pullout(10.0);
        effect.set_surround(0.25);
        assert_eq!(effect.get_pan_pullout(), 0.5);
        assert_eq!(effect.get_time_pullout(), 10.0);
        assert_eq!(effect.get_surround(), 0.25);
        assert_eq!(MAX_DELAY_MS, 50);
    }

    #[test]
    fn ring_size_is_the_next_power_of_two() {
        let effect = AudioEffectStereoEnhance::new();
        // (50 + 2) ms at 44100 Hz = 2293.2 frames → 4096.
        let instance = effect.instantiate(44100.0);
        assert_eq!(instance.delay_ringbuff.len(), 4096);
        assert_eq!(instance.ringbuff_mask, 4095);
        // At 48 kHz: 2496 frames → 4096.
        let instance = effect.instantiate(48000.0);
        assert_eq!(instance.delay_ringbuff.len(), 4096);
        // A tiny rate still rounds up from zero frames to one.
        let instance = effect.instantiate(1.0);
        assert_eq!(instance.delay_ringbuff.len(), 1);
        assert_eq!(instance.ringbuff_mask, 0);
    }

    #[test]
    fn full_pullout_without_delay_is_the_identity() {
        let effect = AudioEffectStereoEnhance::new();
        let input: Vec<AudioFrame> = (0..128)
            .map(|index| AudioFrame::new(index as f32 * 0.01, -(index as f32) * 0.005))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        for (a, b) in output.iter().zip(&input) {
            assert!((a.left - b.left).abs() < 1e-6 && (a.right - b.right).abs() < 1e-6);
        }
    }

    #[test]
    fn pan_pullout_moves_both_channels_to_the_centre() {
        let mut effect = AudioEffectStereoEnhance::new();
        effect.set_pan_pullout(0.0);
        let input = vec![AudioFrame::new(1.0, 0.0); 4];
        let mut output = vec![AudioFrame::default(); 4];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        // intensity 0 → both channels become the centre (0.5).
        for frame in &output {
            assert!((frame.left - 0.5).abs() < 1e-6);
            assert!((frame.right - 0.5).abs() < 1e-6);
        }

        // Halfway keeps half of the original difference.
        effect.set_pan_pullout(0.5);
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        assert!((output[0].left - 0.75).abs() < 1e-6);
        assert!((output[0].right - 0.25).abs() < 1e-6);
    }

    #[test]
    fn time_pullout_delays_only_the_right_channel() {
        let mut effect = AudioEffectStereoEnhance::new();
        effect.set_time_pullout(1.0);
        let rate = 48000.0;
        let frames = 512;
        let mut input = vec![AudioFrame::default(); frames];
        input[0] = AudioFrame::new(1.0, 2.0);
        let mut output = vec![AudioFrame::default(); frames];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let delay = (0.001 * rate) as usize;
        // The left channel is untouched.
        assert!((output[0].left - 1.0).abs() < 1e-6);
        // The right channel appears one millisecond late.
        assert!(output[delay].right.abs() > 1.9);
        assert!(output[0].right.abs() < 1e-6);
    }

    #[test]
    fn surround_adds_and_subtracts_the_delayed_mid() {
        let mut effect = AudioEffectStereoEnhance::new();
        effect.set_time_pullout(1.0);
        effect.set_surround(0.5);
        let rate = 48000.0;
        let frames = 512;
        let mut input = vec![AudioFrame::default(); frames];
        input[0] = AudioFrame::new(1.0, 1.0);
        let mut output = vec![AudioFrame::default(); frames];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let delay = (0.001 * rate) as usize;
        // At frame 0 the dry impulse passes unchanged.
        assert!((output[0].left - 1.0).abs() < 1e-6);
        assert!((output[0].right - 1.0).abs() < 1e-6);
        // One delay later the dry signal is silent and only the surround mid is
        // added (+0.5 left, -0.5 right).
        assert!((output[delay].left - 0.5).abs() < 1e-6);
        assert!((output[delay].right + 0.5).abs() < 1e-6);
    }
}
