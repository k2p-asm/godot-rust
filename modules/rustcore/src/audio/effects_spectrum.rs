//! `servers/audio/effects/audio_effect_spectrum_analyzer.*`: the analyzer.
//!
//! Ported from `audio_effect_spectrum_analyzer.cpp`: the pass-through capture,
//! the half-overlapping Hann window filling two complex FFT buffers (left/right),
//! the SMB FFT per channel, the magnitude history ring (`buffer_length` worth of
//! spectra) and `get_magnitude_for_frequency_range` with the average and max
//! modes (index clamped into `0..fft_size`, swapped when reversed).
//!
//! The audio server is not ported; the instance takes the mix rate at
//! construction.

use crate::audio::fft::smb_fft;
use crate::math::audio_frame::AudioFrame;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MagnitudeMode {
    Average = 0,
    Max = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FftSize {
    Fft256 = 0,
    Fft512 = 1,
    Fft1024 = 2,
    Fft2048 = 3,
    Fft4096 = 4,
}

impl FftSize {
    pub fn samples(self) -> usize {
        match self {
            FftSize::Fft256 => 256,
            FftSize::Fft512 => 512,
            FftSize::Fft1024 => 1024,
            FftSize::Fft2048 => 2048,
            FftSize::Fft4096 => 4096,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectSpectrumAnalyzer {
    pub buffer_length: f32,
    pub fft_size: FftSize,
}

impl Default for AudioEffectSpectrumAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectSpectrumAnalyzer {
    pub fn new() -> Self {
        Self {
            buffer_length: 2.0,
            fft_size: FftSize::Fft1024,
        }
    }

    pub fn set_buffer_length(&mut self, seconds: f32) {
        self.buffer_length = seconds;
    }

    pub fn get_buffer_length(&self) -> f32 {
        self.buffer_length
    }

    /// `set_fft_size`: out-of-range values are refused.
    pub fn set_fft_size(&mut self, size: i32) -> bool {
        let parsed = match size {
            0 => FftSize::Fft256,
            1 => FftSize::Fft512,
            2 => FftSize::Fft1024,
            3 => FftSize::Fft2048,
            4 => FftSize::Fft4096,
            _ => return false,
        };
        self.fft_size = parsed;
        true
    }

    pub fn get_fft_size(&self) -> FftSize {
        self.fft_size
    }

    /// `instantiate`: the history length is `buffer_length / (fft_size / rate) + 1`
    /// truncated to an integer.
    pub fn instantiate(&self, mix_rate: f32) -> SpectrumAnalyzerInstance {
        let fft_size = self.fft_size.samples();
        let fft_count = (self.buffer_length / (fft_size as f32 / mix_rate)) as usize + 1;
        SpectrumAnalyzerInstance {
            fft_history: vec![vec![AudioFrame::default(); fft_size]; fft_count],
            temporal_fft: vec![0.0; fft_size * 8],
            temporal_fft_pos: 0,
            fft_size,
            fft_count,
            fft_pos: 0,
            mix_rate,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpectrumAnalyzerInstance {
    fft_history: Vec<Vec<AudioFrame>>,
    temporal_fft: Vec<f32>,
    temporal_fft_pos: usize,
    fft_size: usize,
    fft_count: usize,
    fft_pos: usize,
    mix_rate: f32,
}

impl SpectrumAnalyzerInstance {
    /// `process`: copies the input through and captures spectra.
    pub fn process(&mut self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        dst[..count].copy_from_slice(&src[..count]);

        let mut cursor = 0usize;
        let mut remaining = count;
        while remaining > 0 {
            let to_fill = (self.fft_size * 2 - self.temporal_fft_pos).min(remaining);
            let to_fill_step = std::f64::consts::TAU / (self.fft_size * 2) as f64;

            for _ in 0..to_fill {
                let window = -0.5
                    * (to_fill_step * self.temporal_fft_pos as f64).cos() as f32
                    + 0.5;
                let frame = src[cursor];
                self.temporal_fft[self.temporal_fft_pos * 2] = window * frame.left;
                self.temporal_fft[self.temporal_fft_pos * 2 + 1] = 0.0;
                self.temporal_fft[(self.temporal_fft_pos + self.fft_size * 2) * 2] =
                    window * frame.right;
                self.temporal_fft[(self.temporal_fft_pos + self.fft_size * 2) * 2 + 1] = 0.0;
                cursor += 1;
                self.temporal_fft_pos += 1;
            }

            remaining -= to_fill;

            if self.temporal_fft_pos == self.fft_size * 2 {
                let split = self.fft_size * 4;
                {
                    let (left, right) = self.temporal_fft.split_at_mut(split);
                    smb_fft(left, self.fft_size * 2, -1);
                    smb_fft(right, self.fft_size * 2, -1);
                }
                let next = (self.fft_pos + 1) % self.fft_count;
                let history = &mut self.fft_history[next];
                for index in 0..self.fft_size {
                    let left_re = self.temporal_fft[index * 2];
                    let left_im = self.temporal_fft[index * 2 + 1];
                    history[index].left =
                        Vector2::new(left_re, left_im).length() / self.fft_size as f32;
                    let right_re = self.temporal_fft[split + index * 2];
                    let right_im = self.temporal_fft[split + index * 2 + 1];
                    history[index].right =
                        Vector2::new(right_re, right_im).length() / self.fft_size as f32;
                }
                self.fft_pos = next;
                self.temporal_fft_pos = 0;
            }
        }
    }

    /// `get_magnitude_for_frequency_range`.
    pub fn get_magnitude_for_frequency_range(
        &self,
        begin_hz: f32,
        end_hz: f32,
        mode: MagnitudeMode,
    ) -> Vector2 {
        let nyquist = self.mix_rate * 0.5;
        let mut begin_pos = (begin_hz * self.fft_size as f32 / nyquist) as i32;
        let mut end_pos = (end_hz * self.fft_size as f32 / nyquist) as i32;
        let max_index = self.fft_size as i32 - 1;
        begin_pos = begin_pos.clamp(0, max_index);
        end_pos = end_pos.clamp(0, max_index);
        if begin_pos > end_pos {
            std::mem::swap(&mut begin_pos, &mut end_pos);
        }
        let history = &self.fft_history[self.fft_pos];

        if mode == MagnitudeMode::Average {
            let mut avg = Vector2::ZERO;
            for index in begin_pos..=end_pos {
                avg += Vector2::new(history[index as usize].left, history[index as usize].right);
            }
            avg / (end_pos - begin_pos + 1) as f32
        } else {
            let mut max = Vector2::ZERO;
            for index in begin_pos..=end_pos {
                max.x = max.x.max(history[index as usize].left);
                max.y = max.y.max(history[index as usize].right);
            }
            max
        }
    }

    pub fn get_fft_size(&self) -> usize {
        self.fft_size
    }

    pub fn get_fft_count(&self) -> usize {
        self.fft_count
    }

    pub fn get_mix_rate(&self) -> f32 {
        self.mix_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frames: usize, frequency: f32, rate: f32, amplitude: f32) -> Vec<AudioFrame> {
        (0..frames)
            .map(|index| {
                let value =
                    amplitude * (std::f32::consts::TAU * frequency * index as f32 / rate).sin();
                AudioFrame::new(value, value)
            })
            .collect()
    }

    fn rms(frames: &[AudioFrame]) -> f32 {
        (frames.iter().map(|frame| frame.left * frame.left).sum::<f32>()
            / frames.len() as f32)
            .sqrt()
    }

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectSpectrumAnalyzer::new();
        assert_eq!(effect.get_buffer_length(), 2.0);
        assert_eq!(effect.get_fft_size(), FftSize::Fft1024);
        assert!(effect.set_fft_size(0));
        assert_eq!(effect.get_fft_size(), FftSize::Fft256);
        assert!(!effect.set_fft_size(5));
        assert_eq!(effect.get_fft_size(), FftSize::Fft256);
        effect.set_buffer_length(0.5);
        assert_eq!(effect.get_buffer_length(), 0.5);
        assert_eq!(FftSize::Fft4096.samples(), 4096);
        assert_eq!(FftSize::Fft4096 as i32, 4);
        assert_eq!(MagnitudeMode::Average as i32, 0);
    }

    #[test]
    fn instance_layout() {
        let effect = AudioEffectSpectrumAnalyzer::new();
        let instance = effect.instantiate(44100.0);
        assert_eq!(instance.get_fft_size(), 1024);
        // 2 / (1024 / 44100) + 1 = 87.13 -> 87.
        assert_eq!(instance.get_fft_count(), 87);
        assert_eq!(instance.get_mix_rate(), 44100.0);

        let mut small = AudioEffectSpectrumAnalyzer::new();
        small.set_buffer_length(0.1);
        small.set_fft_size(0);
        let instance = small.instantiate(48000.0);
        // 0.1 / (256 / 48000) + 1 = 19.75 -> 19.
        assert_eq!(instance.get_fft_count(), 19);
        assert_eq!(instance.get_fft_size(), 256);
    }

    #[test]
    fn process_is_a_pass_through() {
        let effect = AudioEffectSpectrumAnalyzer::new();
        let mut instance = effect.instantiate(44100.0);
        // Fewer frames than one analysis window: no FFT has been captured yet.
        let input = sine(1000, 1000.0, 44100.0, 0.5);
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);
        assert_eq!(output, input);
        // Unknown bands read as zero before any FFT has been captured.
        assert_eq!(
            instance.get_magnitude_for_frequency_range(0.0, 1000.0, MagnitudeMode::Max),
            Vector2::ZERO
        );
    }

    #[test]
    fn tone_peaks_in_its_frequency_bin() {
        let effect = AudioEffectSpectrumAnalyzer::new();
        let mut instance = effect.instantiate(44100.0);
        let rate = 44100.0;
        // Two full analysis frames: the spectra land in the history.
        let input = sine(4096, 1000.0, rate, 1.0);
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);

        // The Hann window halves the amplitude, so a unit sine peaks at ~0.5.
        let in_band =
            instance.get_magnitude_for_frequency_range(900.0, 1100.0, MagnitudeMode::Max);
        let elsewhere =
            instance.get_magnitude_for_frequency_range(5000.0, 7000.0, MagnitudeMode::Max);
        assert!(in_band.x > 0.3, "in-band {in_band:?}");
        assert!(in_band.y > 0.3, "in-band {in_band:?}");
        assert!(elsewhere.x < 0.1, "out of band {elsewhere:?}");

        // Only a couple of bins carry the tone, so the band average is a small
        // fraction of the peak.
        let average =
            instance.get_magnitude_for_frequency_range(900.0, 1100.0, MagnitudeMode::Average);
        assert!(average.x > 0.02, "average {average:?}");
        assert!(average.x < in_band.x);
    }

    #[test]
    fn stereo_channels_are_independent() {
        let effect = AudioEffectSpectrumAnalyzer::new();
        let mut instance = effect.instantiate(44100.0);
        let rate = 44100.0;
        let input: Vec<AudioFrame> = (0..4096)
            .map(|index| {
                let tone = (std::f32::consts::TAU * 1000.0 * index as f32 / rate).sin();
                AudioFrame::new(tone, 0.0)
            })
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);

        let left = instance.get_magnitude_for_frequency_range(900.0, 1100.0, MagnitudeMode::Max);
        assert!(left.x > 0.3);
        assert!(left.y < 1e-6, "the right channel stays silent: {left:?}");
        assert_eq!(output, input);
    }

    #[test]
    fn reversed_and_out_of_range_ranges_are_clamped() {
        let effect = AudioEffectSpectrumAnalyzer::new();
        let mut instance = effect.instantiate(44100.0);
        let input = sine(2048, 440.0, 44100.0, 1.0);
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);

        // Reversed ranges are swapped, so both calls agree.
        let forward = instance.get_magnitude_for_frequency_range(300.0, 600.0, MagnitudeMode::Max);
        let reversed =
            instance.get_magnitude_for_frequency_range(600.0, 300.0, MagnitudeMode::Max);
        assert_eq!(forward, reversed);
        // Negative/oversized ranges clamp to the valid bins.
        let clamped =
            instance.get_magnitude_for_frequency_range(-100.0, 100_000.0, MagnitudeMode::Max);
        assert!(clamped.x >= forward.x);
        // A zero-length range still reads one bin.
        let single = instance.get_magnitude_for_frequency_range(440.0, 440.0, MagnitudeMode::Max);
        assert!(single.x.is_finite());

        // The captured signal is a scaled copy, so the pass-through holds.
        assert!((rms(&output) - rms(&input)).abs() < 1e-6);
    }
}
