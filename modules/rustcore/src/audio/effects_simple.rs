//! `servers/audio/effects/audio_effect_{amplify,panner,limiter,distortion}.*`.
//!
//! Four self-contained effects, ported with their process loops and state:
//!
//! - `AudioEffectAmplify`: dB volume with per-buffer interpolation from the last
//!   mix value (click-free changes);
//! - `AudioEffectPanner`: the engine's cross-feed pan law
//!   (`lvol = clamp(1 - pan)`, `rvol = clamp(1 + pan)`);
//! - `AudioEffectLimiter`: the soft-clip limiter (threshold 0 dB, ceiling -0.1 dB,
//!   soft clip 2 dB, ratio 10) with makeup gain, per-sample soft clipping and the
//!   ceiling clamp;
//! - `AudioEffectDistortion`: five modes (clip, atan, lofi, overdrive, waveshape)
//!   with the pre/post gain, the one-pole high-frequency keep filter and
//!   denormal flushing.
//!
//! The effect resources' object bindings and the audio server are not ported;
//! instances are plain structs processed over `AudioFrame` buffers.

use crate::math::audio_frame::{undenormalize, AudioFrame};
use crate::math::funcs;

// -- Amplify ----------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct AudioEffectAmplify {
    pub volume_db: f32,
}

#[derive(Clone, Debug)]
pub struct AmplifyInstance {
    volume_db: f32,
    mix_volume_db: f32,
}

impl AudioEffectAmplify {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_volume_db(&mut self, volume_db: f32) {
        self.volume_db = volume_db;
    }

    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }

    /// `set_volume_linear`.
    pub fn set_volume_linear(&mut self, volume: f32) {
        self.volume_db = funcs::linear_to_db(volume as f64) as f32;
    }

    pub fn get_volume_linear(&self) -> f32 {
        funcs::db_to_linear(self.volume_db as f64) as f32
    }

    /// `instantiate`: the instance starts at the current volume.
    pub fn instantiate(&self) -> AmplifyInstance {
        AmplifyInstance {
            volume_db: self.volume_db,
            mix_volume_db: self.volume_db,
        }
    }
}

impl AmplifyInstance {
    pub fn process(&mut self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        if count == 0 {
            return;
        }
        let mut volume = funcs::db_to_linear(self.mix_volume_db as f64) as f32;
        let target = funcs::db_to_linear(self.volume_db as f64) as f32;
        let increment = (target - volume) / count as f32;
        for index in 0..count {
            dst[index] = AudioFrame::new(
                src[index].left * volume,
                src[index].right * volume,
            );
            volume += increment;
        }
        self.mix_volume_db = self.volume_db;
    }

    pub fn set_volume_db(&mut self, volume_db: f32) {
        self.volume_db = volume_db;
    }

    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }
}

// -- Panner -----------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct AudioEffectPanner {
    pub pan: f32,
}

#[derive(Clone, Debug, Default)]
pub struct PannerInstance;

impl AudioEffectPanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instantiate(&self) -> PannerInstance {
        PannerInstance
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan;
    }

    pub fn get_pan(&self) -> f32 {
        self.pan
    }

    pub fn process(&self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        let left_volume = (1.0 - self.pan).clamp(0.0, 1.0);
        let right_volume = (1.0 + self.pan).clamp(0.0, 1.0);
        for index in 0..count {
            let left = src[index].left;
            let right = src[index].right;
            dst[index] = AudioFrame::new(
                left * left_volume + right * (1.0 - right_volume),
                right * right_volume + left * (1.0 - left_volume),
            );
        }
    }
}

// -- Limiter ----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AudioEffectLimiter {
    pub threshold: f32,
    pub ceiling: f32,
    pub soft_clip: f32,
    pub soft_clip_ratio: f32,
}

impl Default for AudioEffectLimiter {
    fn default() -> Self {
        Self {
            threshold: 0.0,
            ceiling: -0.1,
            soft_clip: 2.0,
            soft_clip_ratio: 10.0,
        }
    }
}

impl AudioEffectLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        let threshold_db = self.threshold;
        let ceiling_linear = funcs::db_to_linear(self.ceiling as f64) as f32;
        let ceiling_db = self.ceiling;
        let makeup = funcs::db_to_linear((ceiling_db - threshold_db) as f64) as f32;
        let sc = -self.soft_clip;
        let scv = funcs::db_to_linear(sc as f64) as f32;
        let peak_db = ceiling_db + 25.0;
        let scmult = ((ceiling_db - sc) / (peak_db - sc)).abs();

        for index in 0..count {
            let mut left = src[index].left * makeup;
            let mut right = src[index].right * makeup;
            let sign_left = if left < 0.0 { -1.0 } else { 1.0 };
            let sign_right = if right < 0.0 { -1.0 } else { 1.0 };
            let abs_left = left.abs();
            let abs_right = right.abs();
            let over_left = funcs::linear_to_db(abs_left as f64) as f32 - ceiling_db;
            let over_right = funcs::linear_to_db(abs_right as f64) as f32 - ceiling_db;

            if abs_left > scv {
                left = sign_left
                    * (scv + funcs::db_to_linear((over_left * scmult) as f64) as f32);
            }
            if abs_right > scv {
                right = sign_right
                    * (scv + funcs::db_to_linear((over_right * scmult) as f64) as f32);
            }

            left = ceiling_linear.min(left.abs()) * if left < 0.0 { -1.0 } else { 1.0 };
            right = ceiling_linear.min(right.abs()) * if right < 0.0 { -1.0 } else { 1.0 };
            dst[index] = AudioFrame::new(left, right);
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

// -- Distortion -------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DistortionMode {
    Clip = 0,
    Atan = 1,
    Lofi = 2,
    Overdrive = 3,
    Waveshape = 4,
}

#[derive(Clone, Debug)]
pub struct AudioEffectDistortion {
    pub mode: DistortionMode,
    pub pre_gain: f32,
    pub keep_hf_hz: f32,
    pub post_gain: f32,
    pub drive: f32,
}

impl Default for AudioEffectDistortion {
    fn default() -> Self {
        Self {
            mode: DistortionMode::Clip,
            pre_gain: 0.0,
            keep_hf_hz: 16000.0,
            post_gain: 0.0,
            drive: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DistortionInstance {
    h: [f32; 2],
}

impl AudioEffectDistortion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instantiate(&self) -> DistortionInstance {
        DistortionInstance::default()
    }

    pub fn set_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => DistortionMode::Clip,
            1 => DistortionMode::Atan,
            2 => DistortionMode::Lofi,
            3 => DistortionMode::Overdrive,
            4 => DistortionMode::Waveshape,
            _ => return false,
        };
        self.mode = parsed;
        true
    }

    /// `process`: interleaved stereo in/out; `sample_rate` comes from the audio
    /// server in the engine.
    pub fn process(
        &self,
        instance: &mut DistortionInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        if count == 0 {
            return;
        }
        let lpf_c = (-std::f64::consts::TAU * self.keep_hf_hz as f64 / sample_rate as f64).exp()
            as f32;
        let lpf_ic = 1.0 - lpf_c;
        let drive = self.drive;
        let pregain = funcs::db_to_linear(self.pre_gain as f64) as f32;
        let postgain = funcs::db_to_linear(self.post_gain as f64) as f32;
        let atan_mult = 10f32.powf(drive * drive * 3.0) - 1.0 + 0.001;
        let atan_div = 1.0 / (atan_mult.atan() * (1.0 + drive * 8.0));
        let lofi_mult = 2f32.powf(2.0 + (1.0 - drive) * 14.0);

        for frame in 0..count {
            for channel in 0..2 {
                let input = if channel == 0 {
                    src[frame].left
                } else {
                    src[frame].right
                };
                let out = undenormalize(input * lpf_ic + lpf_c * instance.h[channel]);
                instance.h[channel] = out;
                let high_frequencies = input - out;
                let mut value = out * pregain;

                match self.mode {
                    DistortionMode::Clip => {
                        let sign = if value < 0.0 { -1.0 } else { 1.0 };
                        value = value.abs().powf(1.0001 - drive) * sign;
                        value = value.clamp(-1.0, 1.0);
                    }
                    DistortionMode::Atan => {
                        value = (value * atan_mult).atan() * atan_div;
                    }
                    DistortionMode::Lofi => {
                        value = (value * lofi_mult + 0.5).floor() / lofi_mult;
                    }
                    DistortionMode::Overdrive => {
                        let x = value * 0.686306;
                        let z = 1.0 + ((x.abs().sqrt()) * -0.75).exp();
                        value = (x.exp() - (-x * z).exp()) / (x.exp() + (-x).exp());
                    }
                    DistortionMode::Waveshape => {
                        let k = 2.0 * drive / (1.00001 - drive);
                        value = (1.0 + k) * value / (1.0 + k * value.abs());
                    }
                }

                let output = value * postgain + high_frequencies;
                if channel == 0 {
                    dst[frame].left = output;
                } else {
                    dst[frame].right = output;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(values: &[(f32, f32)]) -> Vec<AudioFrame> {
        values
            .iter()
            .map(|(left, right)| AudioFrame::new(*left, *right))
            .collect()
    }

    #[test]
    fn amplify_interpolates_volume() {
        let mut effect = AudioEffectAmplify::new();
        assert_eq!(effect.get_volume_db(), 0.0);
        let mut instance = effect.instantiate();
        let input = frames(&[(1.0, -1.0); 4]);
        let mut output = vec![AudioFrame::default(); 4];
        instance.process(&input, &mut output);
        assert_eq!(output[0], AudioFrame::new(1.0, -1.0));

        // +6.0206 dB is a gain of 2; the first frame of a changed buffer starts
        // at the old volume and ramps.
        effect.set_volume_db(6.0206);
        instance.set_volume_db(6.0206);
        instance.process(&input, &mut output);
        assert!((output[0].left - 1.002).abs() < 0.01, "ramp start {}", output[0].left);
        assert!((output[3].left - 1.75).abs() < 1e-4, "the ramp reaches target minus one step");

        // Linear setters round-trip through dB.
        effect.set_volume_linear(0.5);
        assert!((effect.get_volume_db() + 6.0206).abs() < 0.01);
        assert!((effect.get_volume_linear() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn panner_cross_feeds() {
        let mut effect = AudioEffectPanner::new();
        assert_eq!(effect.get_pan(), 0.0);
        let input = frames(&[(0.5, 0.25)]);
        let mut output = vec![AudioFrame::default(); 1];
        effect.process(&input, &mut output);
        assert_eq!(output[0], AudioFrame::new(0.5, 0.25), "centre is identity");

        effect.set_pan(-1.0);
        effect.process(&input, &mut output);
        assert!((output[0].left - 0.75).abs() < 1e-6, "both channels to the left");
        assert!(output[0].right.abs() < 1e-6);

        effect.set_pan(1.0);
        effect.process(&input, &mut output);
        assert!(output[0].left.abs() < 1e-6);
        assert!((output[0].right - 0.75).abs() < 1e-6);

        // Out-of-range pans clamp through the same law.
        effect.set_pan(2.0);
        effect.process(&input, &mut output);
        assert!(output[0].right >= 0.75);
    }

    #[test]
    fn limiter_clamps_to_the_ceiling() {
        let effect = AudioEffectLimiter::new();
        assert_eq!(effect.get_threshold_db(), 0.0);
        assert_eq!(effect.get_ceiling_db(), -0.1);
        assert_eq!(effect.get_soft_clip_db(), 2.0);
        assert_eq!(effect.get_soft_clip_ratio(), 10.0);

        let input = frames(&[(4.0, -4.0), (0.1, -0.1), (0.0, 0.0)]);
        let mut output = vec![AudioFrame::default(); 3];
        effect.process(&input, &mut output);
        let ceiling = funcs::db_to_linear(-0.1) as f32;
        assert!(output[0].left <= ceiling + 1e-6);
        assert!(output[0].right >= -ceiling - 1e-6);
        assert!(output[1].left <= ceiling);
        assert_eq!(output[2], AudioFrame::new(0.0, 0.0));

        // A lower ceiling limits harder.
        let mut strict = AudioEffectLimiter::new();
        strict.set_ceiling_db(-24.0);
        strict.set_threshold_db(-12.0);
        strict.process(&input, &mut output);
        let strict_ceiling = funcs::db_to_linear(-24.0) as f32;
        assert!(output[0].left <= strict_ceiling + 1e-6);
    }

    #[test]
    fn distortion_modes() {
        let mut effect = AudioEffectDistortion::new();
        assert_eq!(effect.mode, DistortionMode::Clip);
        assert_eq!(effect.pre_gain, 0.0);
        assert_eq!(effect.post_gain, 0.0);
        assert_eq!(effect.keep_hf_hz, 16000.0);
        assert_eq!(effect.drive, 0.0);
        assert!(effect.set_mode(4));
        assert!(!effect.set_mode(5));
        effect.set_mode(0);

        // Clip mode: the low-passed component saturates at 1 in steady state (the
        // high-frequency remainder is added back, so isolated samples can exceed it).
        let constant = frames(&[(8.0, -8.0); 2048]);
        let mut steady = vec![AudioFrame::default(); 2048];
        effect.process(&mut effect.instantiate(), &constant, &mut steady, 44100.0);
        assert!(
            steady[2047].left <= 1.0 + 1e-3,
            "clip steady state {}",
            steady[2047].left
        );
        assert!(steady[2047].left >= -1.0 - 1e-3);
        let zeros = frames(&[(0.0, 0.0)]);
        let mut zero_out = vec![AudioFrame::default(); 1];
        effect.process(&mut effect.instantiate(), &zeros, &mut zero_out, 44100.0);
        assert_eq!(zero_out[0], AudioFrame::new(0.0, 0.0));

        // Waveshape stays bounded and maps zero to zero.
        let mut shape = AudioEffectDistortion::new();
        shape.set_mode(4);
        shape.drive = 0.5;
        let mut shaped = vec![AudioFrame::default(); 2048];
        shape.process(&mut shape.instantiate(), &constant, &mut shaped, 44100.0);
        // The waveshaper's asymptote is (1 + k) / k = 1.5 at this drive.
        assert!(shaped[2047].left.abs() <= 1.5 + 1e-3);
        let mut shaped_zero = vec![AudioFrame::default(); 1];
        shape.process(&mut shape.instantiate(), &zeros, &mut shaped_zero, 44100.0);
        assert_eq!(shaped_zero[0], AudioFrame::new(0.0, 0.0));

        // The keep-HF filter passes DC unchanged in steady state.
        let mut lofi = AudioEffectDistortion::new();
        lofi.set_mode(2);
        lofi.drive = 0.0;
        let constant = frames(&[(0.5, 0.5); 4096]);
        let mut steady = vec![AudioFrame::default(); 4096];
        lofi.process(&mut lofi.instantiate(), &constant, &mut steady, 44100.0);
        assert!((steady[4095].left - 0.5).abs() < 1e-3);
    }
}
