//! `servers/audio/effects/eq_filter.*` and `audio_effect_eq.*`: the EQ.
//!
//! Ported from `eq_filter.cpp`: the per-band second-order sections
//! (`c1 = 0.5 - r1`, `c2 = 2 r1`, `c3 = 2 (0.5 + r1) cos(w)`), the band
//! coefficients built from logarithmically spaced octave bands (the quadratic
//! root solving included), the 6/8/10/21/31-band presets, and
//! `BandProcess::process_one` with its two-sample history.
//!
//! `AudioEffectEQ` runs two `BandProcess` banks (left/right), converts each band
//! gain from dB and sums the band outputs — exactly like
//! `AudioEffectEQInstance::process`.
//!
//! The resource bindings and the audio server are not ported; the mix rate is
//! passed in.

use crate::math::audio_frame::AudioFrame;
use crate::math::funcs;

/// `Math::SQRT12` from `core/math/math_defs.h`: `sqrt(1/2)`, so `POW2` is 0.5.
pub const SQRT12: f64 = 0.707_106_781_186_547_5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EqPreset {
    Bands6 = 0,
    Bands8 = 1,
    Bands10 = 2,
    Bands21 = 3,
    Bands31 = 4,
}

fn preset_bands(preset: EqPreset) -> &'static [f32] {
    match preset {
        EqPreset::Bands6 => &[32.0, 100.0, 320.0, 1e3, 3200.0, 10e3],
        EqPreset::Bands8 => &[32.0, 72.0, 192.0, 512.0, 1200.0, 3000.0, 7500.0, 16e3],
        EqPreset::Bands10 => &[
            31.25, 62.5, 125.0, 250.0, 500.0, 1e3, 2e3, 4e3, 8e3, 16e3,
        ],
        EqPreset::Bands21 => &[
            22.0, 32.0, 44.0, 63.0, 90.0, 125.0, 175.0, 250.0, 350.0, 500.0, 700.0, 1e3, 1400.0,
            2e3, 2800.0, 4e3, 5600.0, 8e3, 11e3, 16e3, 22e3,
        ],
        EqPreset::Bands31 => &[
            20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0,
            400.0, 500.0, 630.0, 800.0, 1e3, 1250.0, 1600.0, 2e3, 2500.0, 3150.0, 4e3, 5e3, 6300.0,
            8e3, 10e3, 12500.0, 16e3, 20e3,
        ],
    }
}

/// `solve_quadratic`: the number of real roots.
fn solve_quadratic(a: f64, b: f64, c: f64) -> (i32, f64, f64) {
    let base = 2.0 * a;
    if base == 0.0 {
        return (0, 0.0, 0.0);
    }
    let mut squared = b * b - 4.0 * a * c;
    if squared < 0.0 {
        return (0, 0.0, 0.0);
    }
    squared = squared.sqrt();
    let r1 = (-b + squared) / base;
    let r2 = (-b - squared) / base;
    if r1 == r2 {
        (1, r1, r2)
    } else {
        (2, r1, r2)
    }
}

#[derive(Clone, Debug, Default)]
pub struct BandProcess {
    c1: f32,
    c2: f32,
    c3: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    b1: f32,
    b2: f32,
    b3: f32,
}

impl BandProcess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_coefficients(&self) -> (f32, f32, f32) {
        (self.c1, self.c2, self.c3)
    }

    /// `process_one`.
    pub fn process_one(&mut self, data: &mut f32) {
        self.a1 = *data;
        self.b1 = self.c1 * (self.a1 - self.a3) + self.c3 * self.b2 - self.c2 * self.b3;
        *data = self.b1;
        self.a3 = self.a2;
        self.a2 = self.a1;
        self.b3 = self.b2;
        self.b2 = self.b1;
    }
}

#[derive(Clone, Debug)]
pub struct Eq {
    band_freqs: Vec<f32>,
    coefficients: Vec<(f32, f32, f32)>,
    mix_rate: f32,
}

impl Default for Eq {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl Eq {
    pub fn new(mix_rate: f32) -> Self {
        let mut eq = Self {
            band_freqs: Vec::new(),
            coefficients: Vec::new(),
            mix_rate,
        };
        eq.set_preset_band_mode(EqPreset::Bands6);
        eq
    }

    pub fn get_band_count(&self) -> usize {
        self.band_freqs.len()
    }

    pub fn get_band_frequency(&self, band: usize) -> f32 {
        self.band_freqs.get(band).copied().unwrap_or(0.0)
    }

    pub fn set_mix_rate(&mut self, mix_rate: f32) {
        self.mix_rate = mix_rate;
        self.recalculate_band_coefficients();
    }

    pub fn get_mix_rate(&self) -> f32 {
        self.mix_rate
    }

    /// `set_preset_band_mode`.
    pub fn set_preset_band_mode(&mut self, preset: EqPreset) {
        self.band_freqs = preset_bands(preset).to_vec();
        self.recalculate_band_coefficients();
    }

    /// `set_bands`.
    pub fn set_bands(&mut self, bands: &[f32]) {
        self.band_freqs = bands.to_vec();
        self.recalculate_band_coefficients();
    }

    /// `get_band_processor`.
    pub fn get_band_processor(&self, band: usize) -> Option<BandProcess> {
        let (c1, c2, c3) = *self.coefficients.get(band)?;
        let mut processor = BandProcess::new();
        processor.c1 = c1;
        processor.c2 = c2;
        processor.c3 = c3;
        Some(processor)
    }

    fn recalculate_band_coefficients(&mut self) {
        const LOG2: f64 = std::f64::consts::LN_2;
        let band_log = |f: f32| (f as f64).ln() / LOG2;
        self.coefficients = Vec::with_capacity(self.band_freqs.len());
        for index in 0..self.band_freqs.len() {
            let frequency = self.band_freqs[index] as f64;
            let octave_size = if index == 0 {
                if self.band_freqs.len() < 2 {
                    0.0
                } else {
                    band_log(self.band_freqs[1]) - band_log(self.band_freqs[0])
                }
            } else if index == self.band_freqs.len() - 1 {
                band_log(frequency as f32) - band_log(self.band_freqs[index - 1])
            } else {
                let next = band_log(self.band_freqs[index + 1]) - band_log(frequency as f32);
                let prev = band_log(frequency as f32) - band_log(self.band_freqs[index - 1]);
                (next + prev) / 2.0
            };

            let frq_l = (frequency / 2.0f64.powf(octave_size / 2.0)).round();
            let side_gain2 = SQRT12 * SQRT12;
            let th = std::f64::consts::TAU * frequency / self.mix_rate as f64;
            let th_l = std::f64::consts::TAU * frq_l / self.mix_rate as f64;

            let c2a = side_gain2 * th.cos().powi(2)
                - 2.0 * side_gain2 * th_l.cos() * th.cos()
                + side_gain2
                - th_l.sin().powi(2);
            let c2b = 2.0 * side_gain2 * th_l.cos().powi(2)
                + side_gain2 * th.cos().powi(2)
                - 2.0 * side_gain2 * th_l.cos() * th.cos()
                - side_gain2
                + th_l.sin().powi(2);
            let c2c = 0.25 * side_gain2 * th.cos().powi(2)
                - 0.5 * side_gain2 * th_l.cos() * th.cos()
                + 0.25 * side_gain2
                - 0.25 * th_l.sin().powi(2);

            let (roots, r1, _) = solve_quadratic(c2a, c2b, c2c);
            if roots == 0 {
                self.coefficients.push((0.0, 0.0, 0.0));
                continue;
            }
            let c1 = 2.0 * ((0.5 - r1) / 2.0);
            let c2 = 2.0 * r1;
            let c3 = 2.0 * (0.5 + r1) * th.cos();
            self.coefficients.push((c1 as f32, c2 as f32, c3 as f32));
        }
    }
}

/// `AudioEffectEQ` (and the `EQ6`/`EQ10`/`EQ21` preset subclasses).
#[derive(Clone, Debug)]
pub struct AudioEffectEq {
    eq: Eq,
    gains: Vec<f32>,
}

impl AudioEffectEq {
    pub fn new(preset: EqPreset, mix_rate: f32) -> Self {
        let eq = Eq::new(mix_rate);
        let count = preset_bands(preset).len();
        let mut effect = Self {
            eq,
            gains: vec![0.0; count],
        };
        effect.eq.set_preset_band_mode(preset);
        effect.gains = vec![0.0; effect.eq.get_band_count()];
        effect
    }

    pub fn eq(&self) -> &Eq {
        &self.eq
    }

    pub fn get_band_count(&self) -> usize {
        self.gains.len()
    }

    /// `set_band_gain_db`: out-of-range bands are ignored.
    pub fn set_band_gain_db(&mut self, band: usize, volume: f32) {
        if let Some(slot) = self.gains.get_mut(band) {
            *slot = volume;
        }
    }

    pub fn get_band_gain_db(&self, band: usize) -> f32 {
        self.gains.get(band).copied().unwrap_or(0.0)
    }

    /// `instantiate`: two processors per band (left/right).
    pub fn instantiate(&self) -> EqInstance {
        let count = self.eq.get_band_count();
        let mut bands: [Vec<BandProcess>; 2] = [Vec::with_capacity(count), Vec::with_capacity(count)];
        for bank in bands.iter_mut() {
            for band in 0..count {
                bank.push(self.eq.get_band_processor(band).unwrap_or_default());
            }
        }
        EqInstance {
            bands,
            gains: Vec::with_capacity(count),
        }
    }

    /// `process`: sums every band output scaled by its linear gain.
    pub fn process(&self, instance: &mut EqInstance, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        let band_count = self.gains.len().min(instance.bands[0].len());
        if instance.gains.len() != band_count {
            instance.gains = vec![0.0; band_count];
        }
        for index in 0..band_count {
            instance.gains[index] = funcs::db_to_linear(self.gains[index] as f64) as f32;
        }
        for frame in 0..count {
            let mut left = 0.0f32;
            let mut right = 0.0f32;
            for band in 0..band_count {
                let mut l = src[frame].left;
                let mut r = src[frame].right;
                instance.bands[0][band].process_one(&mut l);
                instance.bands[1][band].process_one(&mut r);
                left += l * instance.gains[band];
                right += r * instance.gains[band];
            }
            dst[frame] = AudioFrame::new(left, right);
        }
    }
}

#[derive(Clone, Debug)]
pub struct EqInstance {
    bands: [Vec<BandProcess>; 2],
    gains: Vec<f32>,
}

impl Default for EqInstance {
    fn default() -> Self {
        Self {
            bands: [Vec::new(), Vec::new()],
            gains: Vec::new(),
        }
    }
}

impl EqInstance {
    pub fn get_gain(&self, band: usize) -> f32 {
        self.gains.get(band).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_the_engine_bands() {
        let eq = Eq::new(44100.0);
        assert_eq!(eq.get_band_count(), 6);
        assert_eq!(eq.get_band_frequency(0), 32.0);
        assert_eq!(eq.get_band_frequency(5), 10e3);

        let mut eq = Eq::new(44100.0);
        eq.set_preset_band_mode(EqPreset::Bands8);
        assert_eq!(eq.get_band_count(), 8);
        assert_eq!(eq.get_band_frequency(3), 512.0);
        eq.set_preset_band_mode(EqPreset::Bands10);
        assert_eq!(eq.get_band_count(), 10);
        eq.set_preset_band_mode(EqPreset::Bands21);
        assert_eq!(eq.get_band_count(), 21);
        eq.set_preset_band_mode(EqPreset::Bands31);
        assert_eq!(eq.get_band_count(), 31);
        assert_eq!(eq.get_band_frequency(0), 20.0);
        assert_eq!(eq.get_band_frequency(30), 20e3);
        assert_eq!(eq.get_band_frequency(99), 0.0);
    }

    #[test]
    fn coefficients_are_finite_for_every_range() {
        let mut eq = Eq::new(44100.0);
        for preset in [
            EqPreset::Bands6,
            EqPreset::Bands8,
            EqPreset::Bands10,
            EqPreset::Bands21,
            EqPreset::Bands31,
        ] {
            eq.set_preset_band_mode(preset);
            for band in 0..eq.get_band_count() {
                let processor = eq.get_band_processor(band).expect("band");
                let (c1, c2, c3) = processor.get_coefficients();
                assert!(c1.is_finite() && c2.is_finite() && c3.is_finite());
            }
        }
        assert!(eq.get_band_processor(99).is_none());

        // A custom band set and a different mix rate recalculate cleanly.
        eq.set_bands(&[100.0, 1000.0, 10000.0]);
        assert_eq!(eq.get_band_count(), 3);
        assert_eq!(eq.get_band_frequency(1), 1000.0);
        eq.set_mix_rate(48000.0);
        assert_eq!(eq.get_mix_rate(), 48000.0);
        assert!(eq.get_band_processor(0).is_some());
    }

    #[test]
    fn band_process_filters_and_keeps_history() {
        let eq = Eq::new(44100.0);
        let mut processor = eq.get_band_processor(2).expect("band");

        // An impulse produces a non-zero, finite response.
        let mut impulse = 1.0f32;
        processor.process_one(&mut impulse);
        assert!(impulse.is_finite());
        assert!(impulse.abs() > 0.0);
        let mut tail = 0.0f32;
        processor.process_one(&mut tail);
        assert!(tail.is_finite());

        // Two processors have independent state.
        let mut a = eq.get_band_processor(0).unwrap();
        let mut b = eq.get_band_processor(0).unwrap();
        let mut x = 1.0f32;
        a.process_one(&mut x);
        let mut y = 0.0f32;
        b.process_one(&mut y);
        assert!(b.get_coefficients() == a.get_coefficients());
        assert_ne!(x, 0.0);
        assert_eq!(y, 0.0, "a fresh processor of a zero-input sample starts at 0");
    }

    #[test]
    fn eq_effect_sums_band_outputs() {
        let mut effect = AudioEffectEq::new(EqPreset::Bands6, 44100.0);
        assert_eq!(effect.get_band_count(), 6);
        for band in 0..6 {
            assert_eq!(effect.get_band_gain_db(band), 0.0);
        }
        effect.set_band_gain_db(0, -60.0);
        assert_eq!(effect.get_band_gain_db(0), -60.0);
        effect.set_band_gain_db(99, 5.0); // ignored

        // A constant input reaches a steady state through the band bank.
        let input = vec![AudioFrame::new(0.5, -0.5); 4096];
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output);
        // A 1 kHz tone sits in the middle of the 6-band bank, so the summed
        // band outputs reconstruct most of it (the bank rejects DC by design).
        let mut input = vec![AudioFrame::default(); 4096];
        for (index, frame) in input.iter_mut().enumerate() {
            let value = 0.5 * (std::f32::consts::TAU * 1000.0 * index as f32 / 44100.0).sin();
            *frame = AudioFrame::new(value, -value);
        }
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output);
        assert!(output[4095].left.is_finite());
        let rms = |frames: &[AudioFrame], start: usize| -> f32 {
            let window = &frames[start..];
            (window.iter().map(|frame| frame.left * frame.left).sum::<f32>()
                / window.len() as f32)
                .sqrt()
        };
        let input_rms = rms(&input, 2048);
        let output_rms = rms(&output, 2048);
        assert!(output_rms > input_rms * 0.5, "bank passes the mid band");
        assert!(output_rms < input_rms * 2.0, "bank does not blow up");
    }

    #[test]
    fn muted_bands_silence_the_output() {
        let mut effect = AudioEffectEq::new(EqPreset::Bands10, 44100.0);
        for band in 0..effect.get_band_count() {
            effect.set_band_gain_db(band, -400.0);
        }
        let input = vec![AudioFrame::new(1.0, 1.0); 512];
        let mut output = vec![AudioFrame::default(); input.len()];
        let mut instance = effect.instantiate();
        effect.process(&mut instance, &input, &mut output);
        assert!(output[511].left.abs() < 1e-6);
        assert!(output[511].right.abs() < 1e-6);
    }
}
