//! `servers/audio/audio_rb_resampler.*`: the ring-buffer resampler.
//!
//! Ported from `audio_rb_resampler.cpp`: the power-of-two ring buffer with
//! `read_buf` staging, `write()` channel interleaving, `flush`/`clear`, the
//! reader/writer space accounting, `get_num_of_ready_frames`, and `mix()` with
//! the engine's fixed-point linear interpolation (`MIX_FRAC_BITS = 13`), the
//! channel downmixes (1, 2, 4, 6, 8 → stereo with the -3 dB/-6 dB centre/side/rear
//! coefficients), the end-of-buffer fadeout and silence fill.
//!
//! The fixed-point offset advances before sampling, exactly like the engine (the
//! first produced frame is at integer position 1); that quirk is preserved.

use crate::math::audio_frame::AudioFrame;

pub const MIX_FRAC_BITS: u32 = 13;
pub const MIX_FRAC_LEN: u32 = 1 << MIX_FRAC_BITS;
pub const MIX_FRAC_MASK: u32 = MIX_FRAC_LEN - 1;

const SQRT2: f32 = std::f32::consts::SQRT_2;

#[derive(Clone, Debug)]
pub struct AudioRbResampler {
    rb_bits: u32,
    rb_len: usize,
    rb_mask: usize,
    channels: usize,
    src_mix_rate: u32,
    target_mix_rate: u32,
    playback_speed: f64,
    rb_read_pos: usize,
    rb_write_pos: usize,
    offset: u32,
    rb: Vec<f32>,
    read_buf: Vec<f32>,
}

/// `Math::nearest_shift`: the smallest shift with `1 << shift >= value`.
fn nearest_shift(value: u64) -> u32 {
    let mut shift = 0u32;
    let mut power = 1u64;
    while power < value {
        power <<= 1;
        shift += 1;
    }
    shift
}

impl Default for AudioRbResampler {
    fn default() -> Self {
        Self {
            rb_bits: 0,
            rb_len: 0,
            rb_mask: 0,
            channels: 0,
            src_mix_rate: 0,
            target_mix_rate: 0,
            playback_speed: 1.0,
            rb_read_pos: 0,
            rb_write_pos: 0,
            offset: 0,
            rb: Vec::new(),
            read_buf: Vec::new(),
        }
    }
}

impl AudioRbResampler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_ready(&self) -> bool {
        !self.rb.is_empty()
    }

    pub fn get_channel_count(&self) -> usize {
        if !self.is_ready() {
            0
        } else {
            self.channels
        }
    }

    pub fn get_total(&self) -> usize {
        self.rb_len.saturating_sub(1)
    }

    pub fn flush(&mut self) {
        self.rb_read_pos = 0;
        self.rb_write_pos = 0;
        self.offset = 0;
    }

    pub fn clear(&mut self) {
        if !self.is_ready() {
            return;
        }
        self.rb.clear();
        self.read_buf.clear();
        self.offset = 0;
        self.rb_read_pos = 0;
        self.rb_write_pos = 0;
        self.rb_len = 0;
        self.rb_bits = 0;
        self.rb_mask = 0;
        self.channels = 0;
    }

    /// `get_writer_space`.
    pub fn get_writer_space(&self) -> usize {
        let read = self.rb_read_pos;
        let write = self.rb_write_pos;
        if read == write {
            self.rb_len - 1
        } else if write < read {
            read - write - 1
        } else {
            ((self.rb_len as isize - write as isize) + (read as isize - 1)) as usize
        }
    }

    /// `get_reader_space`.
    pub fn get_reader_space(&self) -> usize {
        let read = self.rb_read_pos;
        let write = self.rb_write_pos;
        if read == write {
            0
        } else if write < read {
            self.rb_len - read + write
        } else {
            write - read
        }
    }

    pub fn has_data(&self) -> bool {
        self.is_ready() && self.rb_read_pos != self.rb_write_pos
    }

    /// `setup`: 1/2/4/6/8 channels; the ring size follows the buffered
    /// milliseconds (or the minimum).
    pub fn setup(
        &mut self,
        channels: usize,
        src_mix_rate: u32,
        target_mix_rate: u32,
        buffer_msec: u32,
        minbuff_needed: u32,
    ) -> bool {
        if !matches!(channels, 1 | 2 | 4 | 6 | 8) {
            return false;
        }
        let requested = ((buffer_msec as f64 / 1000.0) * src_mix_rate as f64) as u64;
        let desired_rb_bits = nearest_shift(requested.max(minbuff_needed as u64));

        let recreate = !self.is_ready()
            || desired_rb_bits != self.rb_bits
            || channels != self.channels;
        if recreate {
            self.channels = channels;
            self.rb_bits = desired_rb_bits;
            self.rb_len = 1usize << self.rb_bits;
            self.rb_mask = self.rb_len - 1;
            let size = self.rb_len * channels;
            self.rb = vec![0.0; size];
            self.read_buf = vec![0.0; size];
        } else {
            // Re-zero the existing buffers (the engine does this always).
            for value in self.rb.iter_mut() {
                *value = 0.0;
            }
            for value in self.read_buf.iter_mut() {
                *value = 0.0;
            }
        }

        self.src_mix_rate = src_mix_rate;
        self.target_mix_rate = target_mix_rate;
        self.offset = 0;
        self.rb_read_pos = 0;
        self.rb_write_pos = 0;
        true
    }

    /// `get_write_buffer`: the staging buffer for `write(frames)`.
    pub fn get_write_buffer(&mut self) -> &mut [f32] {
        &mut self.read_buf
    }

    /// `write`: interleaves `frames` from the staging buffer into the ring.
    pub fn write(&mut self, frames: usize) -> bool {
        if !self.is_ready() || frames >= self.rb_len {
            return false;
        }
        let channels = self.channels;
        let mask = self.rb_mask;
        let mut wp = self.rb_write_pos;
        for frame in 0..frames {
            for channel in 0..channels {
                self.rb[(wp * channels) + channel] = self.read_buf[(frame * channels) + channel];
            }
            wp = (wp + 1) & mask;
        }
        self.rb_write_pos = wp;
        true
    }

    /// `get_num_of_ready_frames`.
    pub fn get_num_of_ready_frames(&self) -> usize {
        if !self.is_ready() {
            return 0;
        }
        let increment = ((self.src_mix_rate as u64 * MIX_FRAC_LEN as u64)
            / self.target_mix_rate.max(1) as u64) as i64;
        if increment <= 0 {
            return 0;
        }
        let read_space = self.get_reader_space() as i64;
        ((read_space << MIX_FRAC_BITS) / increment) as usize
    }

    /// `_resample`: the fixed-point linear interpolation and channel downmix.
    fn resample(&mut self, dest: &mut [AudioFrame], todo: usize, increment: u32) -> usize {
        let mask_position = (1u32 << (self.rb_bits + MIX_FRAC_BITS)) - 1;
        let mut read: u64 = (self.offset & MIX_FRAC_MASK) as u64;
        for frame in 0..todo {
            self.offset = (self.offset.wrapping_add(increment)) & mask_position;
            read += increment as u64;
            let pos = (self.offset >> MIX_FRAC_BITS) as usize;
            let frac = (self.offset & MIX_FRAC_MASK) as f32 / MIX_FRAC_LEN as f32;
            if pos >= self.rb_len {
                return 0;
            }
            let pos_next = (pos + 1) & self.rb_mask;
            let sample = match self.channels {
                1 => {
                    let v0 = self.rb[pos];
                    let v0n = self.rb[pos_next];
                    let v = v0 + (v0n - v0) * frac;
                    AudioFrame::new(v, v)
                }
                2 => {
                    let v0 = self.rb[pos * 2];
                    let v1 = self.rb[pos * 2 + 1];
                    let v0n = self.rb[pos_next * 2];
                    let v1n = self.rb[pos_next * 2 + 1];
                    AudioFrame::new(
                        v0 + (v0n - v0) * frac,
                        v1 + (v1n - v1) * frac,
                    )
                }
                4 => {
                    // front left, front right, rear left, rear right
                    let v0 = self.rb[pos * 4] + self.rb[pos * 4 + 2] / 2.0;
                    let v1 = self.rb[pos * 4 + 1] + self.rb[pos * 4 + 3] / 2.0;
                    let v0n = self.rb[pos_next * 4] + self.rb[pos_next * 4 + 2] / 2.0;
                    let v1n = self.rb[pos_next * 4 + 1] + self.rb[pos_next * 4 + 3] / 2.0;
                    AudioFrame::new(
                        v0 + (v0n - v0) * frac,
                        v1 + (v1n - v1) * frac,
                    )
                }
                6 => {
                    // front left, center, front right, rear left, rear right, LFE
                    let v0 = self.rb[pos * 6]
                        + self.rb[pos * 6 + 1] / SQRT2
                        + self.rb[pos * 6 + 3] / 2.0;
                    let v1 = self.rb[pos * 6 + 2]
                        + self.rb[pos * 6 + 1] / SQRT2
                        + self.rb[pos * 6 + 4] / 2.0;
                    let v0n = self.rb[pos_next * 6]
                        + self.rb[pos_next * 6 + 1] / SQRT2
                        + self.rb[pos_next * 6 + 3] / 2.0;
                    let v1n = self.rb[pos_next * 6 + 2]
                        + self.rb[pos_next * 6 + 1] / SQRT2
                        + self.rb[pos_next * 6 + 4] / 2.0;
                    AudioFrame::new(
                        v0 + (v0n - v0) * frac,
                        v1 + (v1n - v1) * frac,
                    )
                }
                _ => {
                    // eight channels: FL, C, FR, SL, SR, RL, RR, LFE
                    let v0 = self.rb[pos * 8]
                        + self.rb[pos * 8 + 1] / SQRT2
                        + self.rb[pos * 8 + 3] / SQRT2
                        + self.rb[pos * 8 + 5] / 2.0;
                    let v1 = self.rb[pos * 8 + 2]
                        + self.rb[pos * 8 + 1] / SQRT2
                        + self.rb[pos * 8 + 4] / SQRT2
                        + self.rb[pos * 8 + 6] / 2.0;
                    let v0n = self.rb[pos_next * 8]
                        + self.rb[pos_next * 8 + 1] / SQRT2
                        + self.rb[pos_next * 8 + 3] / SQRT2
                        + self.rb[pos_next * 8 + 5] / 2.0;
                    let v1n = self.rb[pos_next * 8 + 2]
                        + self.rb[pos_next * 8 + 1] / SQRT2
                        + self.rb[pos_next * 8 + 4] / SQRT2
                        + self.rb[pos_next * 8 + 6] / 2.0;
                    AudioFrame::new(
                        v0 + (v0n - v0) * frac,
                        v1 + (v1n - v1) * frac,
                    )
                }
            };
            dest[frame] = sample;
        }
        (read >> MIX_FRAC_BITS) as usize
    }

    /// `mix`: fills `dest` with at most `dest.len()` frames; returns false when
    /// the resampler is not set up.
    pub fn mix(&mut self, dest: &mut [AudioFrame]) -> bool {
        if !self.is_ready() {
            return false;
        }
        let frames = dest.len();
        let increment = ((self.src_mix_rate as f64 * MIX_FRAC_LEN as f64 * self.playback_speed)
            / self.target_mix_rate.max(1) as f64) as u32;
        let read_space = self.get_reader_space();
        let target_todo = self.get_num_of_ready_frames().min(frames);

        {
            let scratch = &mut dest[..target_todo];
            let mut src_read = self.resample(scratch, target_todo, increment);
            if src_read > read_space {
                src_read = read_space;
            }
            self.rb_read_pos = (self.rb_read_pos + src_read) & self.rb_mask;
        }

        // Fade out the final frames when the writer fell behind.
        if frames - target_todo > 0 {
            for index in 0..target_todo {
                let gain = (target_todo - index) as f32 / target_todo as f32;
                dest[index] = AudioFrame::new(dest[index].left * gain, dest[index].right * gain);
            }
        }
        for frame in dest.iter_mut().skip(target_todo) {
            *frame = AudioFrame::new(0.0, 0.0);
        }
        true
    }

    pub fn set_playback_speed(&mut self, speed: f64) {
        self.playback_speed = speed;
    }

    pub fn get_playback_speed(&self) -> f64 {
        self.playback_speed
    }

    pub fn get_rb_len(&self) -> usize {
        self.rb_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_frames(count: usize) -> Vec<f32> {
        let mut values = Vec::with_capacity(count * 2);
        for frame in 0..count {
            values.push(frame as f32);
            values.push(-(frame as f32));
        }
        values
    }

    #[test]
    fn setup_and_space_accounting() {
        let mut resampler = AudioRbResampler::new();
        assert!(!resampler.is_ready());
        assert_eq!(resampler.get_channel_count(), 0);
        assert_eq!(resampler.get_num_of_ready_frames(), 0);
        assert!(!resampler.write(1));

        assert!(!resampler.setup(3, 44100, 44100, 100, 256));
        assert!(resampler.setup(2, 44100, 44100, 100, 1024));
        assert!(resampler.is_ready());
        assert_eq!(resampler.get_channel_count(), 2);
        assert!(resampler.get_rb_len() >= 1024);
        assert_eq!(resampler.get_total(), resampler.get_rb_len() - 1);
        assert_eq!(resampler.get_writer_space(), resampler.get_rb_len() - 1);
        assert_eq!(resampler.get_reader_space(), 0);
        assert!(!resampler.has_data());

        resampler.get_write_buffer()[..4].copy_from_slice(&[1.0, -1.0, 2.0, -2.0]);
        assert!(resampler.write(2));
        assert_eq!(resampler.get_reader_space(), 2);
        assert!(resampler.has_data());
        assert_eq!(resampler.get_writer_space(), resampler.get_rb_len() - 3);

        resampler.flush();
        assert!(!resampler.has_data());
        assert_eq!(resampler.get_reader_space(), 0);

        resampler.clear();
        assert!(!resampler.is_ready());
        assert_eq!(resampler.get_channel_count(), 0);
    }

    #[test]
    fn equal_rate_interpolation_uses_the_engine_offset() {
        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(2, 44100, 44100, 100, 1024));
        let frames = 100;
        resampler.get_write_buffer()[..frames * 2].copy_from_slice(&ramp_frames(frames));
        assert!(resampler.write(frames));

        let mut dest = vec![AudioFrame::default(); 10];
        assert!(resampler.mix(&mut dest));
        // The engine advances the offset before sampling, so the first produced
        // frame is integer position 1; positions 1..=10 are read.
        for index in 0..10 {
            assert_eq!(dest[index], AudioFrame::new((index + 1) as f32, -((index + 1) as f32)));
        }
        // Ten integer positions were consumed.
        assert_eq!(resampler.get_reader_space(), frames - 10);
    }

    #[test]
    fn upsampling_interpolates() {
        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(1, 22050, 44100, 100, 1024));
        let mut source = Vec::new();
        for frame in 0..20 {
            source.push(frame as f32);
        }
        resampler.get_write_buffer()[..source.len()].copy_from_slice(&source);
        assert!(resampler.write(source.len()));

        let mut dest = vec![AudioFrame::default(); 4];
        assert!(resampler.mix(&mut dest));
        // increment = 4096: positions 0.5, 1, 1.5, 2 → interpolated values.
        assert!((dest[0].left - 0.5).abs() < 1e-6);
        assert!((dest[1].left - 1.0).abs() < 1e-6);
        assert!((dest[2].left - 1.5).abs() < 1e-6);
        assert!((dest[3].left - 2.0).abs() < 1e-6);
        // Mono is duplicated to both channels.
        assert_eq!(dest[0].left, dest[0].right);
    }

    #[test]
    fn downmixes_to_stereo() {
        // 4 channels: FL, FR, RL, RR → rear at half gain.
        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(4, 44100, 44100, 100, 1024));
        let buffer = resampler.get_write_buffer();
        // Two frames: frame 0 all ones, frame 1 zeros.
        for channel in 0..4 {
            buffer[channel] = 1.0;
        }
        assert!(resampler.write(2));
        let mut dest = vec![AudioFrame::default(); 1];
        assert!(resampler.mix(&mut dest));
        // Position 1 is the zero frame.
        assert_eq!(dest[0], AudioFrame::new(0.0, 0.0));

        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(4, 44100, 44100, 100, 1024));
        let buffer = resampler.get_write_buffer();
        // Frames 0 and 1 both: FL=FR=1, RL=RR=0.5 → 1 + 0.25 = 1.25.
        for frame in 0..2 {
            buffer[frame * 4] = 1.0;
            buffer[frame * 4 + 1] = 1.0;
            buffer[frame * 4 + 2] = 0.5;
            buffer[frame * 4 + 3] = 0.5;
        }
        assert!(resampler.write(2));
        let mut dest = vec![AudioFrame::default(); 1];
        assert!(resampler.mix(&mut dest));
        assert!((dest[0].left - 1.25).abs() < 1e-6);
        assert!((dest[0].right - 1.25).abs() < 1e-6);
    }

    #[test]
    fn fadeout_and_silence() {
        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(1, 44100, 44100, 100, 1024));
        let mut source = vec![1.0f32; 5];
        source[0] = 1.0;
        resampler.get_write_buffer()[..source.len()].copy_from_slice(&source);
        assert!(resampler.write(source.len()));

        let mut dest = vec![AudioFrame::default(); 8];
        assert!(resampler.mix(&mut dest));
        // Five frames ready → the first five are faded by 5/5, 4/5, ... and the
        // rest are silence.
        for index in 0..4 {
            let expected = (5 - index) as f32 / 5.0;
            assert!((dest[index].left - expected).abs() < 1e-6, "frame {index}");
        }
        // The fifth produced frame reads position 5, past the written data, so it
        // is silent before the fadeout is applied.
        assert_eq!(dest[4], AudioFrame::new(0.0, 0.0));
        for frame in dest.iter().skip(5) {
            assert_eq!(*frame, AudioFrame::new(0.0, 0.0));
        }
    }

    #[test]
    fn ready_frames_and_playback_speed() {
        let mut resampler = AudioRbResampler::new();
        assert!(resampler.setup(2, 44100, 44100, 100, 1024));
        resampler.get_write_buffer()[..8].copy_from_slice(&[
            1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0,
        ]);
        assert!(resampler.write(4));
        assert_eq!(resampler.get_num_of_ready_frames(), 4);
        resampler.set_playback_speed(2.0);
        assert_eq!(resampler.get_playback_speed(), 2.0);

        // At double speed the increment doubles, so half the source frames fill
        // the same output.
        let mut dest = vec![AudioFrame::default(); 2];
        assert!(resampler.mix(&mut dest));
        assert_eq!(dest[0], AudioFrame::new(3.0, 3.0), "position 1 skipped at 2x");

        let mut never = AudioRbResampler::new();
        assert!(!never.mix(&mut dest));
    }
}
