//! `servers/audio/effects/audio_effect_delay.*`: the stereo delay effect.
//!
//! Ported from `audio_effect_delay.cpp`: the two taps (active, delay ms, level
//! dB, pan), the dry level, the feedback path (active, delay ms, level dB,
//! low-pass cutoff), the power-of-two ring buffer sized from `MAX_DELAY_MS + 100`
//! and the 256-frame chunking. Each tap is panned with the engine's
//! `clamp(1 ± pan)` law and the feedback path is filtered with the one-pole
//! low-pass and denormal-flushed.
//!
//! The effect resource binding and the audio server are not ported; processing
//! takes the mix rate explicitly.

use crate::math::audio_frame::{undenormalize, AudioFrame};
use crate::math::funcs;

pub const MAX_DELAY_MS: i32 = 3000;
const CHUNK: usize = 256;

#[derive(Clone, Debug)]
pub struct AudioEffectDelay {
    pub dry: f32,
    pub tap_1_active: bool,
    pub tap_1_delay_ms: f32,
    pub tap_1_level: f32,
    pub tap_1_pan: f32,
    pub tap_2_active: bool,
    pub tap_2_delay_ms: f32,
    pub tap_2_level: f32,
    pub tap_2_pan: f32,
    pub feedback_active: bool,
    pub feedback_delay_ms: f32,
    pub feedback_level: f32,
    pub feedback_lowpass: f32,
}

impl Default for AudioEffectDelay {
    fn default() -> Self {
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
}

#[derive(Clone, Debug)]
pub struct DelayInstance {
    ring_buffer: Vec<AudioFrame>,
    ring_buffer_mask: usize,
    ring_buffer_pos: usize,
    feedback_buffer: Vec<AudioFrame>,
    feedback_buffer_pos: usize,
    h: AudioFrame,
}

fn frame_mul(frame: AudioFrame, scale: f32) -> AudioFrame {
    AudioFrame::new(frame.left * scale, frame.right * scale)
}

fn frame_add(a: AudioFrame, b: AudioFrame) -> AudioFrame {
    AudioFrame::new(a.left + b.left, a.right + b.right)
}

impl AudioEffectDelay {
    pub fn new() -> Self {
        Self::default()
    }

    /// `instantiate`: sizes the ring buffers from the mix rate.
    pub fn instantiate(&self, sample_rate: f32) -> DelayInstance {
        let mut ring_buffer_max_size = (MAX_DELAY_MS + 100) as f32;
        ring_buffer_max_size /= 1000.0;
        ring_buffer_max_size *= sample_rate;
        let mut ringbuff_size = ring_buffer_max_size as usize;
        let mut bits = 0u32;
        while ringbuff_size > 0 {
            bits += 1;
            ringbuff_size /= 2;
        }
        let ringbuff_size = 1usize << bits;
        DelayInstance {
            ring_buffer: vec![AudioFrame::default(); ringbuff_size],
            ring_buffer_mask: ringbuff_size - 1,
            ring_buffer_pos: 0,
            feedback_buffer: vec![AudioFrame::default(); ringbuff_size],
            feedback_buffer_pos: 0,
            h: AudioFrame::default(),
        }
    }

    /// `process`: splits into 256-frame chunks like the engine.
    pub fn process(
        &self,
        instance: &mut DelayInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let mut offset = 0usize;
        while offset < count {
            let todo = (count - offset).min(CHUNK);
            self.process_chunk(
                instance,
                &src[offset..offset + todo],
                &mut dst[offset..offset + todo],
                sample_rate,
            );
            offset += todo;
        }
    }

    fn process_chunk(
        &self,
        instance: &mut DelayInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let main_level = self.dry;
        let tap1_level = if self.tap_1_active {
            funcs::db_to_linear(self.tap_1_level as f64) as f32
        } else {
            0.0
        };
        let tap1_delay = ((self.tap_1_delay_ms / 1000.0) * sample_rate) as i64;
        let tap2_level = if self.tap_2_active {
            funcs::db_to_linear(self.tap_2_level as f64) as f32
        } else {
            0.0
        };
        let tap2_delay = ((self.tap_2_delay_ms / 1000.0) * sample_rate) as i64;
        let feedback_level = if self.feedback_active {
            funcs::db_to_linear(self.feedback_level as f64) as f32
        } else {
            0.0
        };
        let feedback_delay = ((self.feedback_delay_ms / 1000.0) * sample_rate) as i64;

        let tap1_vol = AudioFrame::new(
            tap1_level * (1.0 - self.tap_1_pan).clamp(0.0, 1.0),
            tap1_level * (1.0 + self.tap_1_pan).clamp(0.0, 1.0),
        );
        let tap2_vol = AudioFrame::new(
            tap2_level * (1.0 - self.tap_2_pan).clamp(0.0, 1.0),
            tap2_level * (1.0 + self.tap_2_pan).clamp(0.0, 1.0),
        );

        let lpf_c = (-std::f64::consts::TAU * self.feedback_lowpass as f64
            / sample_rate as f64)
            .exp() as f32;
        let lpf_ic = 1.0 - lpf_c;
        let mask = instance.ring_buffer_mask;

        for index in 0..src.len() {
            let pos = instance.ring_buffer_pos;
            instance.ring_buffer[pos & mask] = src[index];

            let main_val = frame_mul(src[index], main_level);
            let tap1_index = (pos as i64 - tap1_delay).rem_euclid(instance.ring_buffer.len() as i64)
                as usize;
            let tap2_index = (pos as i64 - tap2_delay).rem_euclid(instance.ring_buffer.len() as i64)
                as usize;
            let tap1_val = AudioFrame::new(
                instance.ring_buffer[tap1_index & mask].left * tap1_vol.left,
                instance.ring_buffer[tap1_index & mask].right * tap1_vol.right,
            );
            let tap2_val = AudioFrame::new(
                instance.ring_buffer[tap2_index & mask].left * tap2_vol.left,
                instance.ring_buffer[tap2_index & mask].right * tap2_vol.right,
            );

            let mut out = frame_add(frame_add(main_val, tap1_val), tap2_val);
            out = frame_add(out, instance.feedback_buffer[instance.feedback_buffer_pos]);

            let scaled = frame_mul(out, feedback_level * lpf_ic);
            let mut fb_in = AudioFrame::new(
                scaled.left + instance.h.left * lpf_c,
                scaled.right + instance.h.right * lpf_c,
            );
            fb_in = AudioFrame::new(undenormalize(fb_in.left), undenormalize(fb_in.right));
            instance.h = fb_in;
            instance.feedback_buffer[instance.feedback_buffer_pos] = fb_in;

            dst[index] = out;
            instance.ring_buffer_pos += 1;
            instance.feedback_buffer_pos += 1;
            if feedback_delay <= 0 || instance.feedback_buffer_pos as i64 >= feedback_delay {
                instance.feedback_buffer_pos = 0;
            }
        }
    }

    // -- accessors ----------------------------------------------------------

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

    pub fn set_tap1_delay_ms(&mut self, delay: f32) {
        self.tap_1_delay_ms = delay;
    }

    pub fn get_tap1_delay_ms(&self) -> f32 {
        self.tap_1_delay_ms
    }

    pub fn set_tap1_level_db(&mut self, level: f32) {
        self.tap_1_level = level;
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

    pub fn set_tap2_delay_ms(&mut self, delay: f32) {
        self.tap_2_delay_ms = delay;
    }

    pub fn get_tap2_delay_ms(&self) -> f32 {
        self.tap_2_delay_ms
    }

    pub fn set_tap2_level_db(&mut self, level: f32) {
        self.tap_2_level = level;
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

    pub fn set_feedback_delay_ms(&mut self, delay: f32) {
        self.feedback_delay_ms = delay;
    }

    pub fn get_feedback_delay_ms(&self) -> f32 {
        self.feedback_delay_ms
    }

    pub fn set_feedback_level_db(&mut self, level: f32) {
        self.feedback_level = level;
    }

    pub fn get_feedback_level_db(&self) -> f32 {
        self.feedback_level
    }

    pub fn set_feedback_lowpass(&mut self, cutoff: f32) {
        self.feedback_lowpass = cutoff;
    }

    pub fn get_feedback_lowpass(&self) -> f32 {
        self.feedback_lowpass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn impulse(len: usize) -> Vec<AudioFrame> {
        let mut frames = vec![AudioFrame::default(); len];
        frames[0] = AudioFrame::new(1.0, 1.0);
        frames
    }

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectDelay::new();
        assert_eq!(effect.get_dry(), 1.0);
        assert!(effect.is_tap1_active());
        assert_eq!(effect.get_tap1_delay_ms(), 250.0);
        assert_eq!(effect.get_tap1_level_db(), -6.0);
        assert_eq!(effect.get_tap1_pan(), 0.2);
        assert!(effect.is_tap2_active());
        assert_eq!(effect.get_tap2_delay_ms(), 500.0);
        assert_eq!(effect.get_tap2_level_db(), -12.0);
        assert_eq!(effect.get_tap2_pan(), -0.4);
        assert!(!effect.is_feedback_active());
        assert_eq!(effect.get_feedback_delay_ms(), 340.0);
        assert_eq!(effect.get_feedback_level_db(), -6.0);
        assert_eq!(effect.get_feedback_lowpass(), 16000.0);

        effect.set_dry(0.5);
        effect.set_tap1_active(false);
        effect.set_tap1_delay_ms(10.0);
        effect.set_tap1_level_db(-3.0);
        effect.set_tap1_pan(-1.0);
        effect.set_tap2_active(false);
        effect.set_tap2_delay_ms(20.0);
        effect.set_tap2_level_db(-9.0);
        effect.set_tap2_pan(1.0);
        effect.set_feedback_active(true);
        effect.set_feedback_delay_ms(30.0);
        effect.set_feedback_level_db(-1.0);
        effect.set_feedback_lowpass(4000.0);
        assert_eq!(effect.get_dry(), 0.5);
        assert!(!effect.is_tap1_active());
        assert_eq!(effect.get_tap1_delay_ms(), 10.0);
        assert_eq!(effect.get_tap1_level_db(), -3.0);
        assert_eq!(effect.get_tap1_pan(), -1.0);
        assert!(!effect.is_tap2_active());
        assert_eq!(effect.get_tap2_delay_ms(), 20.0);
        assert_eq!(effect.get_tap2_level_db(), -9.0);
        assert_eq!(effect.get_tap2_pan(), 1.0);
        assert!(effect.is_feedback_active());
        assert_eq!(effect.get_feedback_delay_ms(), 30.0);
        assert_eq!(effect.get_feedback_level_db(), -1.0);
        assert_eq!(effect.get_feedback_lowpass(), 4000.0);
    }

    #[test]
    fn dry_path_is_identity_when_effects_are_off() {
        let mut effect = AudioEffectDelay::new();
        effect.set_tap1_active(false);
        effect.set_tap2_active(false);
        effect.set_feedback_active(false);
        let input: Vec<AudioFrame> = (0..700)
            .map(|index| AudioFrame::new(index as f32 * 0.001, -(index as f32) * 0.001))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        assert_eq!(output, input);
    }

    #[test]
    fn tap_delay_places_the_echo() {
        let mut effect = AudioEffectDelay::new();
        effect.set_dry(0.0);
        effect.set_tap1_active(true);
        effect.set_tap1_delay_ms(1.0);
        effect.set_tap1_level_db(0.0);
        effect.set_tap1_pan(0.0);
        effect.set_tap2_active(false);
        effect.set_feedback_active(false);

        let rate = 48000.0;
        let input = impulse(4096);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let delayed = (0.001 * rate) as usize;
        assert!((output[delayed].left - 1.0).abs() < 1e-5, "echo at {delayed}");
        assert!((output[delayed].right - 1.0).abs() < 1e-5);
        assert!(output[delayed + 1].left.abs() < 1e-5);
        // The dry signal is muted.
        assert!(output[0].left.abs() < 1e-5);
    }

    #[test]
    fn feedback_repeats_and_decays() {
        let mut effect = AudioEffectDelay::new();
        // The feedback path is fed from the mixed output, so the dry signal must
        // be present for the loop to start.
        effect.set_dry(1.0);
        effect.set_tap1_active(false);
        effect.set_tap2_active(false);
        effect.set_feedback_active(true);
        effect.set_feedback_delay_ms(1.0);
        effect.set_feedback_level_db(-6.0);
        effect.set_feedback_lowpass(16000.0);

        let rate = 48000.0;
        let input = impulse(8192);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let delay = (0.001 * rate) as usize;
        // The feedback path adds the impulse again one delay later and decays.
        assert!(output[delay].left.abs() > 0.1, "first repeat {}", output[delay].left);
        assert!(
            output[delay * 2].left.abs() < output[delay].left.abs(),
            "repeats decay"
        );
        // The first repeat is the loop gain (-6 dB).
        assert!((output[delay].left.abs() - 0.5).abs() < 0.1);
    }
}
