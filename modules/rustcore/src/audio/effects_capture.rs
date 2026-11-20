//! `servers/audio/effects/audio_effect_capture.*`: the capture effect.
//!
//! Ported from `audio_effect_capture.cpp`: the engine's ring-buffer semantics
//! (`size = 1 << nearest_shift(mix_rate * buffer_length)`, one reserved slot,
//! `space_left`/`data_left` exactly as `RingBuffer`), the pass-through capture,
//! the pushed/discarded counters, `get_buffer`/`can_get_buffer`/`clear_buffer`,
//! the buffer-length accessor and the frame queries.
//!
//! The engine shares the ring between the effect and its instances through a
//! `Ref`; this port shares it through `Rc<RefCell<..>>`. The audio server is not
//! ported, so `instantiate` takes the mix rate.

use std::cell::RefCell;
use std::rc::Rc;

use crate::math::audio_frame::AudioFrame;

/// The engine's `nearest_shift`: the shift whose power of two is the first one
/// strictly greater than `number` (0 for 0 and 1).
fn nearest_shift(number: u32) -> u32 {
    if number <= 1 {
        return 0;
    }
    (32 - (number - 1).leading_zeros()).max(1)
}

/// The engine's `RingBuffer<AudioFrame>`.
#[derive(Clone, Debug, Default)]
struct AudioFrameRingBuffer {
    data: Vec<AudioFrame>,
    read_pos: usize,
    write_pos: usize,
    size_mask: usize,
}

impl AudioFrameRingBuffer {
    fn resize(&mut self, power: u32) {
        let new_size = 1usize << power;
        self.data = vec![AudioFrame::default(); new_size];
        self.size_mask = new_size - 1;
        self.read_pos = 0;
        self.write_pos = 0;
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn space_left(&self) -> usize {
        let left = self.read_pos.wrapping_sub(self.write_pos) & self.size_mask;
        if left == 0 {
            self.size().saturating_sub(1)
        } else {
            left - 1
        }
    }

    fn data_left(&self) -> usize {
        self.size()
            .saturating_sub(self.space_left())
            .saturating_sub(1)
    }

    /// `write(buf, size)`: returns the number of frames written.
    fn write(&mut self, frames: &[AudioFrame]) -> usize {
        let count = frames.len().min(self.space_left());
        for (offset, frame) in frames.iter().take(count).enumerate() {
            let index = (self.write_pos + offset) & self.size_mask;
            self.data[index] = *frame;
        }
        self.write_pos = (self.write_pos + count) & self.size_mask;
        count
    }

    /// `read(buf, size)`.
    fn read(&mut self, count: usize) -> Vec<AudioFrame> {
        let take = count.min(self.data_left());
        let mut out = Vec::with_capacity(take);
        for offset in 0..take {
            let index = (self.read_pos + offset) & self.size_mask;
            out.push(self.data[index]);
        }
        self.read_pos = (self.read_pos + take) & self.size_mask;
        out
    }

    fn advance_read(&mut self, count: usize) -> usize {
        let advance = count.min(self.data_left());
        self.read_pos = (self.read_pos + advance) & self.size_mask;
        advance
    }
}

#[derive(Clone, Debug, Default)]
struct CaptureState {
    buffer: AudioFrameRingBuffer,
    buffer_initialized: bool,
    discarded_frames: u64,
    pushed_frames: u64,
}

#[derive(Clone, Debug)]
pub struct AudioEffectCapture {
    state: Rc<RefCell<CaptureState>>,
    buffer_length_seconds: f32,
}

impl Default for AudioEffectCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectCapture {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(CaptureState::default())),
            buffer_length_seconds: 0.1,
        }
    }

    pub fn set_buffer_length(&mut self, seconds: f32) {
        self.buffer_length_seconds = seconds;
    }

    pub fn get_buffer_length(&self) -> f32 {
        self.buffer_length_seconds
    }

    pub fn is_buffer_initialized(&self) -> bool {
        self.state.borrow().buffer_initialized
    }

    /// `instantiate`: sizes the ring on the first call, then clears it.
    pub fn instantiate(&mut self, mix_rate: f32) -> Option<CaptureInstance> {
        let mut state = self.state.borrow_mut();
        if !state.buffer_initialized {
            let target_buffer_size = mix_rate * self.buffer_length_seconds;
            if target_buffer_size <= 0.0 || target_buffer_size >= (1u32 << 27) as f32 {
                return None;
            }
            state.buffer.resize(nearest_shift(target_buffer_size as u32));
            state.buffer_initialized = true;
        }
        let data_left = state.buffer.data_left();
        state.buffer.advance_read(data_left);
        drop(state);
        Some(CaptureInstance {
            state: Rc::clone(&self.state),
        })
    }

    /// `can_get_buffer`.
    pub fn can_get_buffer(&self, frames: usize) -> bool {
        self.state.borrow().buffer.data_left() >= frames
    }

    /// `get_buffer`: empty when uninitialised, when `frames >= buffer.size()`,
    /// when fewer frames are buffered, or when `frames == 0`.
    pub fn get_buffer(&mut self, frames: usize) -> Vec<AudioFrame> {
        let mut state = self.state.borrow_mut();
        if !state.buffer_initialized {
            return Vec::new();
        }
        if frames >= state.buffer.size() {
            return Vec::new();
        }
        let data_left = state.buffer.data_left();
        if data_left < frames || frames == 0 {
            return Vec::new();
        }
        state.buffer.read(frames)
    }

    /// `clear_buffer`.
    pub fn clear_buffer(&mut self) {
        let mut state = self.state.borrow_mut();
        let data_left = state.buffer.data_left();
        state.buffer.advance_read(data_left);
    }

    pub fn get_frames_available(&self) -> usize {
        let state = self.state.borrow();
        if !state.buffer_initialized {
            return 0;
        }
        state.buffer.data_left()
    }

    pub fn get_discarded_frames(&self) -> u64 {
        self.state.borrow().discarded_frames
    }

    pub fn get_buffer_length_frames(&self) -> usize {
        let state = self.state.borrow();
        if !state.buffer_initialized {
            return 0;
        }
        state.buffer.size()
    }

    pub fn get_pushed_frames(&self) -> u64 {
        self.state.borrow().pushed_frames
    }
}

/// `AudioEffectCaptureInstance`: the per-playback writer sharing the effect's
/// ring.
#[derive(Clone, Debug)]
pub struct CaptureInstance {
    state: Rc<RefCell<CaptureState>>,
}

impl CaptureInstance {
    /// `process`: passes the input through and records it when the ring has room
    /// for the whole block; otherwise the whole block is counted as discarded.
    pub fn process(&mut self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        dst[..count].copy_from_slice(&src[..count]);

        let mut state = self.state.borrow_mut();
        if state.buffer.space_left() >= count {
            let written = state.buffer.write(&src[..count]);
            state.pushed_frames = state.pushed_frames.wrapping_add(written as u64);
        } else {
            state.discarded_frames = state.discarded_frames.wrapping_add(count as u64);
        }
    }

    /// `process_silence`.
    pub fn process_silence(&self) -> bool {
        true
    }

    pub fn get_frames_available(&self) -> usize {
        self.state.borrow().buffer.data_left()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(count: usize) -> Vec<AudioFrame> {
        (0..count)
            .map(|index| AudioFrame::new(index as f32, -(index as f32)))
            .collect()
    }

    #[test]
    fn defaults_before_instantiation() {
        let mut capture = AudioEffectCapture::new();
        assert_eq!(capture.get_buffer_length(), 0.1);
        assert!(!capture.is_buffer_initialized());
        assert_eq!(capture.get_frames_available(), 0);
        assert_eq!(capture.get_buffer_length_frames(), 0);
        assert_eq!(capture.get_pushed_frames(), 0);
        assert_eq!(capture.get_discarded_frames(), 0);
        assert!(!capture.can_get_buffer(1));
        assert!(capture.get_buffer(1).is_empty());
        capture.clear_buffer();
    }

    #[test]
    fn instantiation_sizes_the_ring() {
        let mut capture = AudioEffectCapture::new();
        let instance = capture.instantiate(44100.0).expect("instance");
        assert!(capture.is_buffer_initialized());
        // 44100 * 0.1 = 4410 -> nearest_shift 13 -> 8192 frames.
        assert_eq!(capture.get_buffer_length_frames(), 8192);
        assert_eq!(capture.get_frames_available(), 0);
        assert!(instance.process_silence());

        // A custom length is used on the first instantiate only.
        let mut small = AudioEffectCapture::new();
        small.set_buffer_length(0.01);
        small.instantiate(48000.0);
        // 480 -> 512.
        assert_eq!(small.get_buffer_length_frames(), 512);
        small.set_buffer_length(1.0);
        small.instantiate(48000.0);
        assert_eq!(small.get_buffer_length_frames(), 512);
    }

    #[test]
    fn capture_reads_back_in_order() {
        let mut capture = AudioEffectCapture::new();
        let mut instance = capture.instantiate(44100.0).expect("instance");
        let input = ramp(1000);
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);
        assert_eq!(output, input);
        assert_eq!(capture.get_pushed_frames(), 1000);
        assert_eq!(capture.get_frames_available(), 1000);
        assert_eq!(capture.get_discarded_frames(), 0);
        assert!(capture.can_get_buffer(1000));

        let frames = capture.get_buffer(1000);
        assert_eq!(frames.len(), 1000);
        assert_eq!(frames[0], input[0]);
        assert_eq!(frames[999], input[999]);
        assert_eq!(capture.get_frames_available(), 0);
        assert!(capture.get_buffer(1).is_empty());
        assert!(capture.get_buffer(0).is_empty());
        assert!(capture.get_buffer(8192).is_empty());
    }

    #[test]
    fn overflow_counts_discarded_frames() {
        let mut capture = AudioEffectCapture::new();
        let mut instance = capture.instantiate(44100.0).expect("instance");
        // The ring holds 8191 frames; 9000 do not fit in one block.
        let input = ramp(9000);
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);
        assert_eq!(capture.get_pushed_frames(), 0);
        assert_eq!(capture.get_discarded_frames(), 9000);
        assert_eq!(capture.get_frames_available(), 0);

        // Filling to capacity then clearing frees the space again.
        let block = ramp(8191);
        let mut out = vec![AudioFrame::default(); block.len()];
        instance.process(&block, &mut out);
        assert_eq!(capture.get_frames_available(), 8191);
        assert!(!capture.can_get_buffer(8192));
        capture.clear_buffer();
        assert_eq!(capture.get_frames_available(), 0);
        assert_eq!(capture.get_pushed_frames(), 8191);
        assert_eq!(instance.get_frames_available(), 0);
    }

    #[test]
    fn nearest_shift_matches_the_engine() {
        assert_eq!(nearest_shift(0), 0);
        assert_eq!(nearest_shift(1), 0);
        assert_eq!(nearest_shift(2), 1);
        assert_eq!(nearest_shift(3), 2);
        assert_eq!(nearest_shift(4), 2);
        assert_eq!(nearest_shift(5), 3);
        assert_eq!(nearest_shift(4410), 13);
        assert_eq!(nearest_shift(480), 9);
    }
}
