//! `servers/audio/effects/audio_stream_generator.*`: the user-fed stream.
//!
//! Ported from `audio_stream_generator.cpp`:
//!
//! - `AudioStreamGenerator`'s mix-rate mode (`OUTPUT`/`INPUT`/`CUSTOM`, with the
//!   engine's index guard), custom rate (44100) and buffer length (0.5 s), the
//!   `"UserFeed"` stream name, zero length and monophonic flag;
//! - `AudioStreamGeneratorPlayback`'s ring buffer, sized as
//!   `1 << nearest_shift(rate * buffer_len)` (one reserved slot like the engine's
//!   `RingBuffer`), with `push_frame`, `push_buffer`, `can_push_buffer`,
//!   `get_frames_available` and `clear_buffer`.
//!
//! Resampling into the mix (`_mix_internal`) and the playback position/skip
//! accounting are not ported; the buffer is fed and drained directly.

use crate::math::audio_frame::AudioFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GeneratorMixRate {
    Output = 0,
    Input = 1,
    Custom = 2,
    Max = 3,
}

#[derive(Clone, Debug)]
pub struct AudioStreamGenerator {
    mix_rate_mode: GeneratorMixRate,
    mix_rate: f32,
    buffer_len: f32,
}

impl Default for AudioStreamGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamGenerator {
    pub fn new() -> Self {
        Self {
            mix_rate_mode: GeneratorMixRate::Custom,
            mix_rate: 44100.0,
            buffer_len: 0.5,
        }
    }

    pub fn set_mix_rate(&mut self, rate: f32) {
        self.mix_rate = rate;
    }

    pub fn get_mix_rate(&self) -> f32 {
        self.mix_rate
    }

    /// `set_mix_rate_mode`: out-of-range values are refused.
    pub fn set_mix_rate_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => GeneratorMixRate::Output,
            1 => GeneratorMixRate::Input,
            2 => GeneratorMixRate::Custom,
            _ => return false,
        };
        self.mix_rate_mode = parsed;
        true
    }

    pub fn get_mix_rate_mode(&self) -> GeneratorMixRate {
        self.mix_rate_mode
    }

    pub fn set_buffer_length(&mut self, seconds: f32) {
        self.buffer_len = seconds;
    }

    pub fn get_buffer_length(&self) -> f32 {
        self.buffer_len
    }

    /// `_get_target_rate`: without the audio server, custom and output/input
    /// modes fall back to the custom rate (the server rates are unavailable).
    pub fn get_target_rate(&self) -> f32 {
        self.mix_rate
    }

    pub fn get_stream_name(&self) -> &'static str {
        "UserFeed"
    }

    pub fn get_length(&self) -> f64 {
        0.0
    }

    pub fn is_monophonic(&self) -> bool {
        true
    }

    /// `instantiate_playback`.
    pub fn instantiate_playback(&self) -> GeneratorPlayback {
        let target_buffer_size = (self.get_target_rate() * self.buffer_len) as u64;
        let size = nearest_power_of_two(target_buffer_size);
        GeneratorPlayback {
            buffer: Vec::with_capacity(size),
            capacity: size,
        }
    }
}

/// `Math::nearest_shift` followed by `1 << shift`.
fn nearest_power_of_two(value: u64) -> usize {
    let mut shift = 0u32;
    let mut power = 1u64;
    while power < value.max(1) {
        power <<= 1;
        shift += 1;
    }
    let _ = shift;
    1usize << shift
}

#[derive(Clone, Debug, Default)]
pub struct GeneratorPlayback {
    buffer: Vec<AudioFrame>,
    capacity: usize,
}

impl GeneratorPlayback {
    /// `push_frame`: false when the buffer is full.
    pub fn push_frame(&mut self, frame: AudioFrame) -> bool {
        if self.space_left() < 1 {
            return false;
        }
        self.buffer.push(frame);
        true
    }

    pub fn can_push_buffer(&self, frames: usize) -> bool {
        self.space_left() >= frames
    }

    /// `push_buffer`: all-or-nothing.
    pub fn push_buffer(&mut self, frames: &[AudioFrame]) -> bool {
        if self.space_left() < frames.len() {
            return false;
        }
        self.buffer.extend_from_slice(frames);
        true
    }

    pub fn get_frames_available(&self) -> usize {
        self.space_left()
    }

    /// The engine's `RingBuffer` keeps one slot empty.
    fn space_left(&self) -> usize {
        self.capacity.saturating_sub(1 + self.buffer.len())
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Port helper: drains up to `count` frames (the engine mixes them through
    /// `_mix_internal`).
    pub fn pop_frames(&mut self, count: usize) -> Vec<AudioFrame> {
        let take = count.min(self.buffer.len());
        self.buffer.drain(..take).collect()
    }

    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn get_buffered_frames(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_defaults_and_mode_guard() {
        let mut generator = AudioStreamGenerator::new();
        assert_eq!(generator.get_mix_rate_mode(), GeneratorMixRate::Custom);
        assert_eq!(generator.get_mix_rate(), 44100.0);
        assert_eq!(generator.get_buffer_length(), 0.5);
        assert_eq!(generator.get_stream_name(), "UserFeed");
        assert_eq!(generator.get_length(), 0.0);
        assert!(generator.is_monophonic());
        assert_eq!(generator.get_target_rate(), 44100.0);

        assert!(generator.set_mix_rate_mode(0));
        assert_eq!(generator.get_mix_rate_mode(), GeneratorMixRate::Output);
        assert!(generator.set_mix_rate_mode(1));
        assert_eq!(generator.get_mix_rate_mode(), GeneratorMixRate::Input);
        assert!(generator.set_mix_rate_mode(2));
        assert!(!generator.set_mix_rate_mode(3));
        assert_eq!(generator.get_mix_rate_mode(), GeneratorMixRate::Custom);
        assert_eq!(GeneratorMixRate::Max as i32, 3);

        generator.set_mix_rate(22050.0);
        generator.set_buffer_length(1.0);
        assert_eq!(generator.get_mix_rate(), 22050.0);
        assert_eq!(generator.get_buffer_length(), 1.0);
    }

    #[test]
    fn playback_buffer_is_the_next_power_of_two() {
        let generator = AudioStreamGenerator::new();
        let playback = generator.instantiate_playback();
        // 44100 * 0.5 = 22050 → 32768.
        assert_eq!(playback.get_capacity(), 32768);
        assert_eq!(playback.get_frames_available(), 32767);
        assert_eq!(playback.get_buffered_frames(), 0);

        let mut generator = AudioStreamGenerator::new();
        generator.set_mix_rate(1000.0);
        generator.set_buffer_length(0.1);
        let playback = generator.instantiate_playback();
        assert_eq!(playback.get_capacity(), 128);
        assert_eq!(playback.get_frames_available(), 127);
    }

    #[test]
    fn push_frame_and_full_behaviour() {
        let mut generator = AudioStreamGenerator::new();
        generator.set_mix_rate(8.0);
        generator.set_buffer_length(1.0);
        let mut playback = generator.instantiate_playback();
        assert_eq!(playback.get_capacity(), 8);

        for index in 0..7 {
            assert!(playback.push_frame(AudioFrame::new(index as f32, -(index as f32))));
        }
        assert_eq!(playback.get_buffered_frames(), 7);
        assert_eq!(playback.get_frames_available(), 0);
        assert!(!playback.push_frame(AudioFrame::new(9.0, 9.0)));

        playback.clear_buffer();
        assert_eq!(playback.get_frames_available(), 7);
    }

    #[test]
    fn push_buffer_is_all_or_nothing() {
        let mut generator = AudioStreamGenerator::new();
        generator.set_mix_rate(8.0);
        generator.set_buffer_length(1.0);
        let mut playback = generator.instantiate_playback();

        let frames = vec![AudioFrame::new(1.0, 1.0); 4];
        assert!(playback.can_push_buffer(4));
        assert!(playback.push_buffer(&frames));
        assert_eq!(playback.get_buffered_frames(), 4);
        assert_eq!(playback.get_frames_available(), 3);

        let more = vec![AudioFrame::new(2.0, 2.0); 4];
        assert!(!playback.can_push_buffer(4));
        assert!(!playback.push_buffer(&more));
        assert_eq!(playback.get_buffered_frames(), 4);

        // Draining frees space and returns the frames in order.
        let drained = playback.pop_frames(2);
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0], AudioFrame::new(1.0, 1.0));
        assert_eq!(playback.get_frames_available(), 5);
    }
}
