//! `servers/audio/effects/audio_effect_phaser.*`: the phaser effect.
//!
//! Ported from `audio_effect_phaser.cpp`: the six-stage all-pass chain per
//! channel (`AllpassDelay` with `a = (1 - d) / (1 + d)`), the LFO sweeping the
//! all-pass delay between `range_min` and `range_max` (both normalised by
//! Nyquist), the feedback tap and the wet depth.
//!
//! The effect resource binding and the audio server are not ported; processing
//! takes the mix rate explicitly.

use crate::math::audio_frame::AudioFrame;

#[derive(Clone, Debug, Default)]
pub struct AllpassDelay {
    a: f32,
    h: f32,
}

impl AllpassDelay {
    pub fn set_delay(&mut self, delay: f32) {
        self.a = (1.0 - delay) / (1.0 + delay);
    }

    pub fn update(&mut self, sample: f32) -> f32 {
        let out = sample * -self.a + self.h;
        self.h = out * self.a + sample;
        out
    }

    pub fn get_a(&self) -> f32 {
        self.a
    }

    pub fn get_h(&self) -> f32 {
        self.h
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectPhaser {
    pub range_min: f32,
    pub range_max: f32,
    pub rate: f32,
    pub feedback: f32,
    pub depth: f32,
}

impl Default for AudioEffectPhaser {
    fn default() -> Self {
        Self {
            range_min: 440.0,
            range_max: 1600.0,
            rate: 0.5,
            feedback: 0.7,
            depth: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhaserInstance {
    allpass: [[AllpassDelay; 6]; 2],
    phase: f32,
    h: AudioFrame,
}

impl AudioEffectPhaser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instantiate(&self) -> PhaserInstance {
        PhaserInstance {
            allpass: Default::default(),
            phase: 0.0,
            h: AudioFrame::default(),
        }
    }

    pub fn set_range_min_hz(&mut self, hz: f32) {
        self.range_min = hz;
    }

    pub fn get_range_min_hz(&self) -> f32 {
        self.range_min
    }

    pub fn set_range_max_hz(&mut self, hz: f32) {
        self.range_max = hz;
    }

    pub fn get_range_max_hz(&self) -> f32 {
        self.range_max
    }

    pub fn set_rate_hz(&mut self, hz: f32) {
        self.rate = hz;
    }

    pub fn get_rate_hz(&self) -> f32 {
        self.rate
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    pub fn get_feedback(&self) -> f32 {
        self.feedback
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth;
    }

    pub fn get_depth(&self) -> f32 {
        self.depth
    }

    /// `process`.
    pub fn process(
        &self,
        instance: &mut PhaserInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let dmin = self.range_min / (sample_rate / 2.0);
        let dmax = self.range_max / (sample_rate / 2.0);
        let increment = std::f32::consts::TAU * (self.rate / sample_rate);

        for index in 0..count {
            instance.phase += increment;
            while instance.phase >= std::f32::consts::TAU {
                instance.phase -= std::f32::consts::TAU;
            }
            let d = dmin + (dmax - dmin) * ((instance.phase.sin() + 1.0) / 2.0);
            for stage in 0..6 {
                instance.allpass[0][stage].set_delay(d);
                instance.allpass[1][stage].set_delay(d);
            }

            let mut y = src[index].left + instance.h.left * self.feedback;
            for stage in (0..6).rev() {
                y = instance.allpass[0][stage].update(y);
            }
            instance.h.left = y;
            dst[index].left = src[index].left + y * self.depth;

            let mut y = src[index].right + instance.h.right * self.feedback;
            for stage in (0..6).rev() {
                y = instance.allpass[1][stage].update(y);
            }
            instance.h.right = y;
            dst[index].right = src[index].right + y * self.depth;
        }
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
    fn allpass_delay_math() {
        let mut delay = AllpassDelay::default();
        assert_eq!(delay.get_a(), 0.0);
        delay.set_delay(0.5);
        // a = (1 - 0.5) / (1 + 0.5) = 1/3
        assert!((delay.get_a() - 1.0 / 3.0).abs() < 1e-6);
        let out = delay.update(1.0);
        // out = s * -a + h = -1/3; h = out * a + s = -1/9 + 1 = 8/9.
        assert!((out + 1.0 / 3.0).abs() < 1e-6);
        assert!((delay.get_h() - 8.0 / 9.0).abs() < 1e-6);
        // d = 0 gives a = 1: the first sample comes out inverted.
        let mut zero = AllpassDelay::default();
        zero.set_delay(0.0);
        assert!((zero.get_a() - 1.0).abs() < 1e-6);
        assert!((zero.update(0.5) + 0.5).abs() < 1e-6);
        // h = out * a + s = -0.5 + 0.5 = 0.
        assert!(zero.get_h().abs() < 1e-6);
    }

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectPhaser::new();
        assert_eq!(effect.get_range_min_hz(), 440.0);
        assert_eq!(effect.get_range_max_hz(), 1600.0);
        assert_eq!(effect.get_rate_hz(), 0.5);
        assert_eq!(effect.get_feedback(), 0.7);
        assert_eq!(effect.get_depth(), 1.0);

        effect.set_range_min_hz(100.0);
        effect.set_range_max_hz(2000.0);
        effect.set_rate_hz(2.0);
        effect.set_feedback(0.25);
        effect.set_depth(0.5);
        assert_eq!(effect.get_range_min_hz(), 100.0);
        assert_eq!(effect.get_range_max_hz(), 2000.0);
        assert_eq!(effect.get_rate_hz(), 2.0);
        assert_eq!(effect.get_feedback(), 0.25);
        assert_eq!(effect.get_depth(), 0.5);
    }

    #[test]
    fn zero_depth_and_feedback_is_the_dry_signal() {
        let mut effect = AudioEffectPhaser::new();
        effect.set_depth(0.0);
        effect.set_feedback(0.0);
        let input: Vec<AudioFrame> = (0..512)
            .map(|index| AudioFrame::new(index as f32 * 0.01, -(index as f32) * 0.01))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, 44100.0);
        // y is still produced but scaled by zero depth; the dry path is intact.
        assert_eq!(output, input);
    }

    #[test]
    fn wet_path_colours_an_impulse() {
        let mut effect = AudioEffectPhaser::new();
        effect.set_depth(1.0);
        effect.set_feedback(0.0);
        effect.set_rate_hz(0.5);
        let input = impulse(1024);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, 44100.0);

        // Six all-pass stages multiply the first sample by a^6 (~0.5), and the
        // depth adds that to the dry signal, so the peak exceeds unity.
        assert!(output[0].left > 1.0 && output[0].left < 2.0);
        let tail: f32 = output[1..].iter().map(|frame| frame.left.abs()).sum();
        assert!(tail > 0.0, "the all-pass chain smears the impulse");
        // Both channels are processed identically.
        assert!((output[3].left - output[3].right).abs() < 1e-9);
    }

    #[test]
    fn phase_wraps_and_stays_bounded() {
        let mut effect = AudioEffectPhaser::new();
        effect.set_rate_hz(50.0);
        let input = vec![AudioFrame::new(0.1, 0.1); 4410];
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output, 44100.0);
        assert!(instance.phase >= 0.0 && instance.phase < std::f32::consts::TAU);
        // Many LFO cycles later the output is still finite.
        assert!(output.iter().all(|frame| frame.left.is_finite()));
    }
}
