//! `servers/audio/effects/audio_effect_capture.*` and `audio_effect_record.*`.
//!
//! Capture keeps a ring of `AudioFrame`s sized `1 << nearest_shift(mix_rate *
//! buffer_length_seconds)` (strictly larger than the target), with the engine's
//! one-slot-empty ring capacity (`size - 1` frames usable). Processing copies the
//! frames through and either stores them (incrementing `pushed_frames`) or drops
//! the whole block (incrementing `discarded_frames`).
//!
//! Record stores the WAV format (16-bit by default) and the recording flag. The
//! IO thread, actual recording buffer and stream creation are not ported.

use std::collections::VecDeque;

use crate::math::audio_frame::AudioFrame;

fn nearest_shift(number: u64) -> u32 {
    if number == 0 {
        return 0;
    }
    64 - number.leading_zeros()
}

#[derive(Clone, Debug, Default)]
pub struct AudioEffectCaptureData {
    buffer_length_seconds: f32,
    initialized: bool,
    buffer_size: usize,
    frames: VecDeque<AudioFrame>,
    pushed_frames: u64,
    discarded_frames: u64,
}

impl AudioEffectCaptureData {
    pub fn new() -> Self {
        Self {
            buffer_length_seconds: 0.1,
            initialized: false,
            buffer_size: 0,
            frames: VecDeque::new(),
            pushed_frames: 0,
            discarded_frames: 0,
        }
    }

    pub fn set_buffer_length(&mut self, seconds: f32) {
        self.buffer_length_seconds = seconds;
    }

    pub fn get_buffer_length(&self) -> f32 {
        self.buffer_length_seconds
    }

    /// Equivalent of `instantiate`: sizes the ring once for the mix rate.
    pub fn initialize(&mut self, mix_rate: u32) -> bool {
        if !self.initialized {
            let target = mix_rate as f32 * self.buffer_length_seconds;
            if target <= 0.0 || target >= (1u32 << 27) as f32 {
                return false;
            }
            self.buffer_size = 1usize << nearest_shift(target as u64);
            self.initialized = true;
        }
        self.clear_buffer();
        true
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Ring capacity in frames (`buffer.size()`), one slot of which is unused.
    pub fn get_buffer_length_frames(&self) -> usize {
        if self.initialized {
            self.buffer_size
        } else {
            0
        }
    }

    fn usable_capacity(&self) -> usize {
        self.buffer_size.saturating_sub(1)
    }

    pub fn get_frames_available(&self) -> usize {
        if self.initialized {
            self.frames.len()
        } else {
            0
        }
    }

    pub fn can_get_buffer(&self, frames: usize) -> bool {
        self.frames.len() >= frames
    }

    /// Copies frames into the ring or drops the whole block, as the engine does.
    pub fn process(&mut self, incoming: &[AudioFrame]) {
        if self.frames.len() + incoming.len() <= self.usable_capacity() {
            self.frames.extend(incoming.iter().copied());
            self.pushed_frames += incoming.len() as u64;
        } else {
            self.discarded_frames += incoming.len() as u64;
        }
    }

    /// Reads exactly `count` frames; `None` on an invalid request (the engine
    /// returns an empty array for index/data failures).
    pub fn get_buffer(&mut self, count: usize) -> Option<Vec<AudioFrame>> {
        if !self.initialized || count == 0 || count >= self.buffer_size {
            return None;
        }
        if self.frames.len() < count {
            return None;
        }
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(self.frames.pop_front().expect("checked length"));
        }
        Some(out)
    }

    pub fn clear_buffer(&mut self) {
        self.frames.clear();
    }

    pub fn get_pushed_frames(&self) -> u64 {
        self.pushed_frames
    }

    pub fn get_discarded_frames(&self) -> u64 {
        self.discarded_frames
    }
}

/// `AudioStreamWAV::Format`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum WavFormat {
    Bits8,
    Bits16,
    ImaAdpcm,
    Qoa,
}

#[derive(Clone, Debug)]
pub struct AudioEffectRecordData {
    format: WavFormat,
    recording: bool,
}

impl Default for AudioEffectRecordData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectRecordData {
    pub fn new() -> Self {
        Self {
            format: WavFormat::Bits16,
            recording: false,
        }
    }

    pub fn set_format(&mut self, format: WavFormat) {
        self.format = format;
    }

    pub fn get_format(&self) -> WavFormat {
        self.format
    }

    pub fn set_recording(&mut self, recording: bool) {
        self.recording = recording;
    }

    pub fn is_recording_active(&self) -> bool {
        self.recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(v: f32) -> AudioFrame {
        AudioFrame { left: v, right: -v }
    }

    #[test]
    fn capture_sizing_matches_nearest_shift() {
        // 0.1 s at 44100 = 4410 -> nearest_shift 13 -> size 8192.
        let mut c = AudioEffectCaptureData::new();
        assert_eq!(c.get_buffer_length(), 0.1);
        assert!(c.initialize(44100));
        assert!(c.is_initialized());
        assert_eq!(c.get_buffer_length_frames(), 8192);
        assert_eq!(c.get_frames_available(), 0);
        assert_eq!(c.get_pushed_frames(), 0);

        // Invalid lengths are rejected.
        let mut bad = AudioEffectCaptureData::new();
        bad.set_buffer_length(0.0);
        assert!(!bad.initialize(44100));
        let mut huge = AudioEffectCaptureData::new();
        huge.set_buffer_length(1e9);
        assert!(!huge.initialize(44100));
    }

    #[test]
    fn capture_push_read_and_clear() {
        let mut c = AudioEffectCaptureData::new();
        c.initialize(44100);
        // Usable capacity is size - 1.
        assert_eq!(c.get_buffer_length_frames() - 1, 8191);
        c.process(&[frame(1.0), frame(2.0)]);
        assert_eq!(c.get_frames_available(), 2);
        assert_eq!(c.get_pushed_frames(), 2);
        assert!(c.can_get_buffer(2));
        assert!(!c.can_get_buffer(3));

        let out = c.get_buffer(2).expect("two frames");
        assert_eq!(out, vec![frame(1.0), frame(2.0)]);
        assert_eq!(c.get_frames_available(), 0);
        assert!(c.get_buffer(1).is_none());
        assert!(c.get_buffer(0).is_none());
        assert!(c.get_buffer(8192).is_none());

        c.process(&[frame(3.0)]);
        c.clear_buffer();
        assert_eq!(c.get_frames_available(), 0);
    }

    #[test]
    fn capture_discards_whole_blocks_when_full() {
        let mut c = AudioEffectCaptureData::new();
        // 4410 target -> 8192 size; fill to 8189 then add 3 (fits exactly).
        c.set_buffer_length(0.001);
        c.initialize(44100);
        // target 44 -> nearest_shift 6 -> size 64, usable 63.
        assert_eq!(c.get_buffer_length_frames(), 64);
        let block = vec![frame(1.0); 60];
        c.process(&block);
        assert_eq!(c.get_frames_available(), 60);
        c.process(&[frame(2.0); 3]);
        assert_eq!(c.get_frames_available(), 63);
        // Any additional frame would overflow: the whole block is discarded.
        c.process(&[frame(3.0); 2]);
        assert_eq!(c.get_frames_available(), 63);
        assert_eq!(c.get_discarded_frames(), 2);
        assert_eq!(c.get_pushed_frames(), 63);
    }

    #[test]
    fn record_defaults_and_flags() {
        let mut r = AudioEffectRecordData::new();
        assert_eq!(r.get_format(), WavFormat::Bits16);
        assert!(!r.is_recording_active());
        r.set_format(WavFormat::Qoa);
        r.set_recording(true);
        assert_eq!(r.get_format(), WavFormat::Qoa);
        assert!(r.is_recording_active());

        assert_eq!(WavFormat::Bits8 as i32, 0);
        assert_eq!(WavFormat::Bits16 as i32, 1);
        assert_eq!(WavFormat::ImaAdpcm as i32, 2);
        assert_eq!(WavFormat::Qoa as i32, 3);
    }
}
