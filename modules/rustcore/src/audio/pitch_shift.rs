//! `servers/audio/effects/audio_effect_pitch_shift.*`: the SMB pitch shifter.
//!
//! Ported from `audio_effect_pitch_shift.cpp` (Bernsee's `smbPitchShift`,
//! version 1.2) plus `AudioEffectPitchShift`:
//!
//! - the input/output FIFOs with the `fftFrameSize - stepSize` latency,
//! - the Hann analysis window, the phase-difference analysis with the ±π
//!   unwrapping (`qpd`), the magnitude/frequency bin remapping
//!   (`index = k * pitch`, `gSynMagn[index] += gAnaMagn[k]`,
//!   `gSynFreq[index] = gAnaFreq[k] * pitch`),
//! - the phase accumulation and inverse FFT, the overlapped output accumulator
//!   with `2 * window / (fftFrameSize2 * osamp)` and the accumulator/FIFO shifts
//!   (including the engine's `memset` of only `fftFrameSize` elements and its
//!   `fftFrameSize + 2` negative-frequency zeroing),
//! - the effect's `pitch_scale == 1` bypass, oversampling ≥ 4 guard and FFT size
//!   enum.
//!
//! The audio server is not ported; processing takes the mix rate explicitly.

use crate::audio::fft::smb_fft;
use crate::math::audio_frame::AudioFrame;

pub const MAX_FRAME_LENGTH: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PitchShiftFftSize {
    Fft256 = 0,
    Fft512 = 1,
    Fft1024 = 2,
    Fft2048 = 3,
    Fft4096 = 4,
}

impl PitchShiftFftSize {
    pub fn samples(self) -> usize {
        match self {
            PitchShiftFftSize::Fft256 => 256,
            PitchShiftFftSize::Fft512 => 512,
            PitchShiftFftSize::Fft1024 => 1024,
            PitchShiftFftSize::Fft2048 => 2048,
            PitchShiftFftSize::Fft4096 => 4096,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SmbPitchShift {
    g_in_fifo: Vec<f32>,
    g_out_fifo: Vec<f32>,
    g_fft_worksp: Vec<f32>,
    g_last_phase: Vec<f32>,
    g_sum_phase: Vec<f32>,
    g_output_accum: Vec<f32>,
    g_ana_freq: Vec<f32>,
    g_ana_magn: Vec<f32>,
    g_syn_freq: Vec<f32>,
    g_syn_magn: Vec<f32>,
    g_rover: usize,
}

impl Default for SmbPitchShift {
    fn default() -> Self {
        Self::new()
    }
}

impl SmbPitchShift {
    pub fn new() -> Self {
        Self {
            g_in_fifo: vec![0.0; MAX_FRAME_LENGTH],
            g_out_fifo: vec![0.0; MAX_FRAME_LENGTH],
            g_fft_worksp: vec![0.0; 2 * MAX_FRAME_LENGTH],
            g_last_phase: vec![0.0; MAX_FRAME_LENGTH / 2 + 1],
            g_sum_phase: vec![0.0; MAX_FRAME_LENGTH / 2 + 1],
            g_output_accum: vec![0.0; 2 * MAX_FRAME_LENGTH],
            g_ana_freq: vec![0.0; MAX_FRAME_LENGTH],
            g_ana_magn: vec![0.0; MAX_FRAME_LENGTH],
            g_syn_freq: vec![0.0; MAX_FRAME_LENGTH],
            g_syn_magn: vec![0.0; MAX_FRAME_LENGTH],
            g_rover: 0,
        }
    }

    /// `PitchShift`: `input`/`output` are mono frames of equal length.
    pub fn pitch_shift(
        &mut self,
        pitch_shift: f32,
        input: &[f32],
        output: &mut [f32],
        fft_frame_size: usize,
        osamp: usize,
        sample_rate: f32,
    ) {
        let count = input.len().min(output.len());
        if fft_frame_size == 0 || fft_frame_size > MAX_FRAME_LENGTH || osamp == 0 {
            return;
        }

        let fft_frame_size2 = fft_frame_size / 2;
        let step_size = fft_frame_size / osamp;
        let freq_per_bin = sample_rate as f64 / fft_frame_size as f64;
        let expct =
            2.0 * std::f64::consts::PI * step_size as f64 / fft_frame_size as f64;
        let in_fifo_latency = fft_frame_size - step_size;
        if self.g_rover == 0 {
            self.g_rover = in_fifo_latency;
        }

        for i in 0..count {
            self.g_in_fifo[self.g_rover] = input[i];
            output[i] = self.g_out_fifo[self.g_rover - in_fifo_latency];
            self.g_rover += 1;

            if self.g_rover >= fft_frame_size {
                self.g_rover = in_fifo_latency;

                // Windowing and re/im interleave.
                for k in 0..fft_frame_size {
                    let window =
                        -0.5 * (2.0 * std::f64::consts::PI * k as f64
                            / fft_frame_size as f64)
                            .cos() as f32
                            + 0.5;
                    self.g_fft_worksp[2 * k] = self.g_in_fifo[k] * window;
                    self.g_fft_worksp[2 * k + 1] = 0.0;
                }

                smb_fft(
                    &mut self.g_fft_worksp[..2 * fft_frame_size],
                    fft_frame_size,
                    -1,
                );

                // Analysis.
                for k in 0..=fft_frame_size2 {
                    let real = self.g_fft_worksp[2 * k] as f64;
                    let imag = self.g_fft_worksp[2 * k + 1] as f64;

                    let magn = 2.0 * (real * real + imag * imag).sqrt();
                    let phase = imag.atan2(real);

                    let mut tmp = phase - self.g_last_phase[k] as f64;
                    self.g_last_phase[k] = phase as f32;

                    tmp -= k as f64 * expct;

                    // Map the delta phase into the +/- Pi interval.
                    let mut qpd = (tmp / std::f64::consts::PI) as i64;
                    if qpd >= 0 {
                        qpd += qpd & 1;
                    } else {
                        qpd -= qpd & 1;
                    }
                    tmp -= std::f64::consts::PI * qpd as f64;

                    // Deviation from the bin frequency.
                    tmp = osamp as f64 * tmp / (2.0 * std::f64::consts::PI);
                    tmp = k as f64 * freq_per_bin + tmp * freq_per_bin;

                    self.g_ana_magn[k] = magn as f32;
                    self.g_ana_freq[k] = tmp as f32;
                }

                // Processing: the engine clears `fftFrameSize` elements only.
                for value in self.g_syn_magn.iter_mut().take(fft_frame_size) {
                    *value = 0.0;
                }
                for value in self.g_syn_freq.iter_mut().take(fft_frame_size) {
                    *value = 0.0;
                }
                for k in 0..=fft_frame_size2 {
                    let index = (k as f64 * pitch_shift as f64) as usize;
                    if index <= fft_frame_size2 {
                        self.g_syn_magn[index] += self.g_ana_magn[k];
                        self.g_syn_freq[index] = self.g_ana_freq[k] * pitch_shift;
                    }
                }

                // Synthesis.
                for k in 0..=fft_frame_size2 {
                    let magn = self.g_syn_magn[k] as f64;
                    let mut tmp = self.g_syn_freq[k] as f64;

                    tmp -= k as f64 * freq_per_bin;
                    tmp /= freq_per_bin;
                    tmp = 2.0 * std::f64::consts::PI * tmp / osamp as f64;
                    tmp += k as f64 * expct;

                    self.g_sum_phase[k] += tmp as f32;
                    let phase = self.g_sum_phase[k] as f64;

                    self.g_fft_worksp[2 * k] = (magn * phase.cos()) as f32;
                    self.g_fft_worksp[2 * k + 1] = (magn * phase.sin()) as f32;
                }

                // Zero the negative frequencies (from `fftFrameSize + 2`, as the
                // engine does).
                for k in fft_frame_size + 2..2 * fft_frame_size {
                    self.g_fft_worksp[k] = 0.0;
                }

                smb_fft(
                    &mut self.g_fft_worksp[..2 * fft_frame_size],
                    fft_frame_size,
                    1,
                );

                // Window and add to the output accumulator.
                for k in 0..fft_frame_size {
                    let window =
                        -0.5 * (2.0 * std::f64::consts::PI * k as f64
                            / fft_frame_size as f64)
                            .cos() as f32
                            + 0.5;
                    self.g_output_accum[k] += 2.0 * window * self.g_fft_worksp[2 * k]
                        / (fft_frame_size2 * osamp) as f32;
                }
                for k in 0..step_size {
                    self.g_out_fifo[k] = self.g_output_accum[k];
                }

                // Shift the accumulator (`memmove` of `fftFrameSize` floats).
                let accum_len = self.g_output_accum.len();
                self.g_output_accum
                    .copy_within(step_size..step_size + fft_frame_size.min(accum_len - step_size), 0);

                // Move the input FIFO.
                for k in 0..in_fifo_latency {
                    self.g_in_fifo[k] = self.g_in_fifo[k + step_size];
                }
            }
        }
    }

    pub fn get_rover(&self) -> usize {
        self.g_rover
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectPitchShift {
    pub pitch_scale: f32,
    pub oversampling: i32,
    pub fft_size: PitchShiftFftSize,
    pub wet: f32,
    pub dry: f32,
    pub filter: bool,
}

impl Default for AudioEffectPitchShift {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct PitchShiftInstance {
    fft_size: usize,
    shift_l: SmbPitchShift,
    shift_r: SmbPitchShift,
}

impl AudioEffectPitchShift {
    pub fn new() -> Self {
        Self {
            pitch_scale: 1.0,
            oversampling: 4,
            fft_size: PitchShiftFftSize::Fft2048,
            wet: 0.0,
            dry: 0.0,
            filter: false,
        }
    }

    /// `set_pitch_scale`: values that are not positive are refused.
    pub fn set_pitch_scale(&mut self, scale: f32) -> bool {
        if !(scale > 0.0) {
            return false;
        }
        self.pitch_scale = scale;
        true
    }

    pub fn get_pitch_scale(&self) -> f32 {
        self.pitch_scale
    }

    /// `set_oversampling`: values below 4 are refused.
    pub fn set_oversampling(&mut self, amount: i32) -> bool {
        if amount < 4 {
            return false;
        }
        self.oversampling = amount;
        true
    }

    pub fn get_oversampling(&self) -> i32 {
        self.oversampling
    }

    /// `set_fft_size`: out-of-range values are refused.
    pub fn set_fft_size(&mut self, size: i32) -> bool {
        let parsed = match size {
            0 => PitchShiftFftSize::Fft256,
            1 => PitchShiftFftSize::Fft512,
            2 => PitchShiftFftSize::Fft1024,
            3 => PitchShiftFftSize::Fft2048,
            4 => PitchShiftFftSize::Fft4096,
            _ => return false,
        };
        self.fft_size = parsed;
        true
    }

    pub fn get_fft_size(&self) -> PitchShiftFftSize {
        self.fft_size
    }

    pub fn instantiate(&self) -> PitchShiftInstance {
        PitchShiftInstance {
            fft_size: self.fft_size.samples(),
            shift_l: SmbPitchShift::new(),
            shift_r: SmbPitchShift::new(),
        }
    }

    /// `process`.
    pub fn process(
        &self,
        instance: &mut PitchShiftInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        // The engine skips processing entirely at scale 1.0 to avoid distortion
        // (`Math::is_equal_approx(pitch_scale, 1.0)`).
        if (self.pitch_scale - 1.0).abs() < 1e-5 {
            dst[..count].copy_from_slice(&src[..count]);
            return;
        }

        let left_in: Vec<f32> = src[..count].iter().map(|frame| frame.left).collect();
        let right_in: Vec<f32> = src[..count].iter().map(|frame| frame.right).collect();
        let mut left_out = vec![0.0f32; count];
        let mut right_out = vec![0.0f32; count];

        instance.shift_l.pitch_shift(
            self.pitch_scale,
            &left_in,
            &mut left_out,
            instance.fft_size,
            self.oversampling as usize,
            sample_rate,
        );
        instance.shift_r.pitch_shift(
            self.pitch_scale,
            &right_in,
            &mut right_out,
            instance.fft_size,
            self.oversampling as usize,
            sample_rate,
        );

        for index in 0..count {
            dst[index] = AudioFrame::new(left_out[index], right_out[index]);
        }
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

    /// Zero-crossing estimate over the settled part of the signal.
    fn estimate_frequency(frames: &[AudioFrame], rate: f32, skip: usize) -> f32 {
        let window = &frames[skip..];
        let mut crossings = 0usize;
        for pair in window.windows(2) {
            if pair[0].left <= 0.0 && pair[1].left > 0.0 {
                crossings += 1;
            }
        }
        crossings as f32 * rate / window.len() as f32
    }

    #[test]
    fn defaults_and_guards() {
        let mut effect = AudioEffectPitchShift::new();
        assert_eq!(effect.get_pitch_scale(), 1.0);
        assert_eq!(effect.get_oversampling(), 4);
        assert_eq!(effect.get_fft_size(), PitchShiftFftSize::Fft2048);
        assert_eq!(effect.wet, 0.0);
        assert_eq!(effect.dry, 0.0);
        assert!(!effect.filter);

        assert!(!effect.set_pitch_scale(0.0));
        assert!(!effect.set_pitch_scale(-1.0));
        assert!(effect.set_pitch_scale(2.0));
        assert_eq!(effect.get_pitch_scale(), 2.0);

        assert!(!effect.set_oversampling(3));
        assert!(effect.set_oversampling(8));
        assert_eq!(effect.get_oversampling(), 8);

        assert!(effect.set_fft_size(0));
        assert_eq!(effect.get_fft_size(), PitchShiftFftSize::Fft256);
        assert!(!effect.set_fft_size(5));
        assert_eq!(PitchShiftFftSize::Fft4096 as i32, 4);
        assert_eq!(PitchShiftFftSize::Fft4096.samples(), 4096);
        assert_eq!(MAX_FRAME_LENGTH, 8192);
    }

    #[test]
    fn unity_scale_is_a_bypass() {
        let effect = AudioEffectPitchShift::new();
        let input = sine(1024, 440.0, 44100.0, 0.5);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, 44100.0);
        assert_eq!(output, input);
    }

    #[test]
    fn pitch_shift_doubles_the_frequency() {
        let mut effect = AudioEffectPitchShift::new();
        effect.set_pitch_scale(2.0);
        effect.set_fft_size(3); // 2048
        effect.set_oversampling(4);
        let rate = 44100.0;
        let input = sine(32768, 440.0, rate, 0.5);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, rate);

        // The latency is fft_size - step_size = 1536 frames.
        let measured = estimate_frequency(&output, rate, 8192);
        assert!(
            (measured - 880.0).abs() < 90.0,
            "shifted 440 Hz by 2 gives {measured} Hz"
        );
        assert!(output.iter().all(|frame| frame.left.is_finite()));
    }

    #[test]
    fn pitch_shift_halves_the_frequency() {
        let mut effect = AudioEffectPitchShift::new();
        effect.set_pitch_scale(0.5);
        let rate = 44100.0;
        let input = sine(32768, 880.0, rate, 0.5);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(), &input, &mut output, rate);

        let measured = estimate_frequency(&output, rate, 8192);
        assert!(
            (measured - 440.0).abs() < 60.0,
            "shifted 880 Hz by 0.5 gives {measured} Hz"
        );
    }

    #[test]
    fn channels_are_processed_independently() {
        let mut effect = AudioEffectPitchShift::new();
        effect.set_pitch_scale(1.5);
        let rate = 48000.0;
        let mut instance = effect.instantiate();
        let input: Vec<AudioFrame> = (0..8192)
            .map(|index| {
                let value =
                    0.5 * (std::f32::consts::TAU * 440.0 * index as f32 / rate).sin();
                AudioFrame::new(value, 0.0)
            })
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut instance, &input, &mut output, rate);
        assert!(output.iter().any(|frame| frame.left.abs() > 0.01));
        assert!(output.iter().all(|frame| frame.right == 0.0));

        // A fresh instance starts from a zeroed rover and untouched FIFOs.
        let fresh = effect.instantiate();
        assert_eq!(fresh.shift_l.get_rover(), 0);
        assert_eq!(fresh.shift_r.get_rover(), 0);
        assert_eq!(fresh.fft_size, 2048);
    }
}
