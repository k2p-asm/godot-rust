//! `servers/audio/effects/audio_effect_record.*`: the recording effect.
//!
//! Ported from `audio_effect_record.cpp`: the 1500 ms ring buffer sized to the
//! next power of two, `init`/`finish` recording state, the pass-through
//! `process` that stores frames while recording, the ring drain into the
//! interleaved float buffer, and `get_recording` for the 8-bit and 16-bit WAV
//! formats (clamped and little-endian).
//!
//! The engine's I/O thread is replaced by an explicit [`RecordInstance::update_buffer`]
//! call; IMA-ADPCM and QOA compression stay pending, so `get_recording` returns
//! `None` for those formats. The audio server is not ported, so `instantiate`
//! takes the mix rate.

use std::cell::RefCell;
use std::rc::Rc;

use crate::audio::stream_wav::{AudioStreamWav, LoopMode, WavFormat};
use crate::math::audio_frame::AudioFrame;

const IO_BUFFER_SIZE_MS: f32 = 1500.0;

#[derive(Clone, Debug, Default)]
struct RecordState {
    is_recording: bool,
    ring_buffer: Vec<AudioFrame>,
    ring_buffer_mask: usize,
    ring_buffer_pos: usize,
    ring_buffer_read_pos: usize,
    recording_data: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct AudioEffectRecord {
    format: WavFormat,
    instance: Option<Rc<RefCell<RecordState>>>,
}

impl Default for AudioEffectRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectRecord {
    pub fn new() -> Self {
        Self {
            format: WavFormat::Format16Bits,
            instance: None,
        }
    }

    pub fn set_format(&mut self, format: WavFormat) {
        self.format = format;
    }

    pub fn get_format(&self) -> WavFormat {
        self.format
    }

    pub fn has_instance(&self) -> bool {
        self.instance.is_some()
    }

    /// `instantiate`: sizes the ring from the mix rate, keeps recording if the
    /// previous instance was recording, and becomes the current instance.
    pub fn instantiate(&mut self, mix_rate: f32) -> RecordInstance {
        let mut ring_buffer_max_size = IO_BUFFER_SIZE_MS / 1000.0;
        ring_buffer_max_size *= mix_rate;
        let mut ringbuff_size = ring_buffer_max_size as usize;
        let mut bits = 0u32;
        while ringbuff_size > 0 {
            bits += 1;
            ringbuff_size /= 2;
        }
        let ringbuff_size = 1usize << bits;

        let was_recording = self
            .instance
            .as_ref()
            .map(|state| state.borrow().is_recording)
            .unwrap_or(false);

        let state = Rc::new(RefCell::new(RecordState {
            is_recording: false,
            ring_buffer: vec![AudioFrame::default(); ringbuff_size],
            ring_buffer_mask: ringbuff_size - 1,
            ring_buffer_pos: 0,
            ring_buffer_read_pos: 0,
            recording_data: Vec::new(),
        }));
        if was_recording {
            Self::init_state(&state);
        }
        self.instance = Some(Rc::clone(&state));
        RecordInstance { state }
    }

    fn init_state(state: &Rc<RefCell<RecordState>>) {
        let mut guard = state.borrow_mut();
        guard.ring_buffer_pos = 0;
        guard.ring_buffer_read_pos = 0;
        guard.recording_data.clear();
        guard.is_recording = true;
    }

    /// `set_recording_active`: without an instantiated effect this is a no-op
    /// (the engine warns and returns).
    pub fn set_recording_active(&mut self, record: bool) {
        let Some(state) = self.instance.as_ref().cloned() else {
            return;
        };
        if record {
            {
                let mut guard = state.borrow_mut();
                guard.is_recording = false;
            }
            Self::init_state(&state);
        } else {
            state.borrow_mut().is_recording = false;
        }
    }

    /// `is_recording_active`.
    pub fn is_recording_active(&self) -> bool {
        self.instance
            .as_ref()
            .map(|state| state.borrow().is_recording)
            .unwrap_or(false)
    }

    /// `get_recording`: the recorded data as a WAV stream, `None` without an
    /// instance, without data, or for the pending compressed formats.
    pub fn get_recording(&self, mix_rate: i32) -> Option<AudioStreamWav> {
        let state = self.instance.as_ref()?.borrow();
        if state.recording_data.is_empty() {
            return None;
        }
        let mut data = Vec::new();
        match self.format {
            // The engine stores 8-bit samples signed internally and its writer
            // adds 128; this port stores the unsigned byte directly.
            WavFormat::Format8Bits => {
                for sample in &state.recording_data {
                    let value = (*sample * 128.0).clamp(-128.0, 127.0);
                    data.push((value as i8) as u8);
                }
            }
            WavFormat::Format16Bits => {
                for sample in &state.recording_data {
                    let value = (*sample * 32768.0).clamp(-32768.0, 32767.0) as i16;
                    data.extend_from_slice(&value.to_le_bytes());
                }
            }
            WavFormat::FormatImaAdpcm | WavFormat::FormatQoa => return None,
        }

        let mut wav = AudioStreamWav::new();
        // The effect stores unsigned 8-bit samples, which is what the port's
        // loader expects; mark the format accordingly.
        wav.set_data(data);
        wav.set_format(self.format);
        wav.set_mix_rate(mix_rate);
        wav.set_loop_mode(LoopMode::Disabled);
        wav.set_loop_begin(0);
        wav.set_loop_end(0);
        // Forcing mono is not implemented in the engine either.
        wav.set_stereo(true);
        Some(wav)
    }
}

/// `AudioEffectRecordInstance`.
#[derive(Clone, Debug)]
pub struct RecordInstance {
    state: Rc<RefCell<RecordState>>,
}

impl RecordInstance {
    /// `process`: pass-through, storing frames in the ring while recording.
    pub fn process(&mut self, src: &[AudioFrame], dst: &mut [AudioFrame]) {
        let count = src.len().min(dst.len());
        dst[..count].copy_from_slice(&src[..count]);

        let mut state = self.state.borrow_mut();
        if !state.is_recording {
            return;
        }
        for frame in &src[..count] {
            let index = state.ring_buffer_pos & state.ring_buffer_mask;
            state.ring_buffer[index] = *frame;
            state.ring_buffer_pos += 1;
        }
    }

    /// `process_silence`.
    pub fn process_silence(&self) -> bool {
        true
    }

    /// `_update_buffer`: drains the ring into the interleaved float data.
    pub fn update_buffer(&mut self) {
        let mut state = self.state.borrow_mut();
        while state.ring_buffer_read_pos < state.ring_buffer_pos {
            let index = state.ring_buffer_read_pos & state.ring_buffer_mask;
            let frame = state.ring_buffer[index];
            state.recording_data.push(frame.left);
            state.recording_data.push(frame.right);
            state.ring_buffer_read_pos += 1;
        }
    }

    /// `init`.
    pub fn init(&mut self) {
        AudioEffectRecord::init_state(&self.state);
    }

    /// `finish`.
    pub fn finish(&mut self) {
        self.state.borrow_mut().is_recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.state.borrow().is_recording
    }

    pub fn get_ring_buffer_size(&self) -> usize {
        self.state.borrow().ring_buffer.len()
    }

    pub fn get_recording_data(&self) -> Vec<f32> {
        self.state.borrow().recording_data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_state() {
        let mut record = AudioEffectRecord::new();
        assert_eq!(record.get_format(), WavFormat::Format16Bits);
        assert!(!record.has_instance());
        assert!(!record.is_recording_active());
        // Without an instance both calls are harmless.
        record.set_recording_active(true);
        assert!(!record.is_recording_active());
        assert!(record.get_recording(44100).is_none());

        record.set_format(WavFormat::Format8Bits);
        assert_eq!(record.get_format(), WavFormat::Format8Bits);
    }

    #[test]
    fn instantiation_sizes_the_ring() {
        let mut record = AudioEffectRecord::new();
        let instance = record.instantiate(44100.0);
        assert!(record.has_instance());
        // 1500 ms at 44.1 kHz = 66150 frames -> 2^17 = 131072.
        assert_eq!(instance.get_ring_buffer_size(), 131072);
        assert!(!record.is_recording_active());

        let mut low = AudioEffectRecord::new();
        let instance = low.instantiate(8000.0);
        // 12000 frames -> 2^14 = 16384.
        assert_eq!(instance.get_ring_buffer_size(), 16384);
        assert!(!instance.is_recording());
    }

    #[test]
    fn recording_captures_passed_through_audio() {
        let mut record = AudioEffectRecord::new();
        let mut instance = record.instantiate(44100.0);
        let input: Vec<AudioFrame> = (0..100)
            .map(|index| AudioFrame::new(index as f32 * 0.001, -(index as f32) * 0.001))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];

        // Not recording: pass-through only.
        instance.process(&input, &mut output);
        assert_eq!(output, input);
        instance.update_buffer();
        assert!(instance.get_recording_data().is_empty());

        record.set_recording_active(true);
        assert!(record.is_recording_active());
        instance.process(&input, &mut output);
        instance.update_buffer();
        let data = instance.get_recording_data();
        assert_eq!(data.len(), 200);
        assert_eq!(data[0], 0.0);
        assert!((data[2] - 0.001).abs() < 1e-6);
        assert!((data[3] + 0.001).abs() < 1e-6);

        // Finishing stops capture; data stays until a new init.
        instance.finish();
        assert!(!instance.is_recording());
        let before = instance.get_recording_data().len();
        instance.process(&input, &mut output);
        instance.update_buffer();
        assert_eq!(instance.get_recording_data().len(), before);

        // init resets everything.
        instance.init();
        assert!(instance.is_recording());
        assert!(instance.get_recording_data().is_empty());
    }

    #[test]
    fn get_recording_builds_the_wav() {
        let mut record = AudioEffectRecord::new();
        let mut instance = record.instantiate(44100.0);
        record.set_recording_active(true);
        let input: Vec<AudioFrame> = (0..8)
            .map(|index| AudioFrame::new(index as f32 / 16.0, 0.0))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        instance.process(&input, &mut output);
        instance.update_buffer();

        // 16-bit: two channels interleaved, little-endian, clamped.
        let wav = record.get_recording(44100).expect("recording");
        assert_eq!(wav.get_format(), WavFormat::Format16Bits);
        assert_eq!(wav.get_mix_rate(), 44100);
        assert!(wav.is_stereo());
        assert_eq!(wav.get_data().len(), 16 * 2);
        let samples = wav.get_samples();
        assert_eq!(samples.len(), 16);
        assert!((samples[2] - 1.0 / 16.0).abs() < 1e-3);
        let bytes = wav.to_wav_bytes();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[22..24], &2u16.to_le_bytes(), "stereo channels");

        // 8-bit: the port's unsigned storage.
        let mut eight = AudioEffectRecord::new();
        eight.set_format(WavFormat::Format8Bits);
        let mut instance = eight.instantiate(44100.0);
        eight.set_recording_active(true);
        instance.process(&input, &mut output);
        instance.update_buffer();
        let wav = eight.get_recording(44100).expect("recording");
        assert_eq!(wav.get_format(), WavFormat::Format8Bits);
        assert_eq!(wav.get_data().len(), 16);
        assert_eq!(wav.get_data()[0], 0, "0.0 maps to 0 signed");

        // Compressed formats stay pending.
        let mut compressed = AudioEffectRecord::new();
        compressed.set_format(WavFormat::FormatImaAdpcm);
        let mut instance = compressed.instantiate(44100.0);
        compressed.set_recording_active(true);
        instance.process(&input, &mut output);
        instance.update_buffer();
        assert!(compressed.get_recording(44100).is_none());
    }

    #[test]
    fn reinstall_keeps_recording() {
        let mut record = AudioEffectRecord::new();
        let mut instance = record.instantiate(44100.0);
        record.set_recording_active(true);
        let input = vec![AudioFrame::new(0.5, 0.5); 4];
        let mut output = vec![AudioFrame::default(); 4];
        instance.process(&input, &mut output);
        instance.update_buffer();

        // A re-instantiated effect continues an active recording with a fresh
        // buffer, like the engine's `instantiate`.
        let mut fresh = record.instantiate(44100.0);
        assert!(record.is_recording_active());
        assert!(fresh.get_recording_data().is_empty());
        fresh.process(&input, &mut output);
        fresh.update_buffer();
        assert_eq!(fresh.get_recording_data().len(), 8);

        record.set_recording_active(false);
        assert!(!record.is_recording_active());
    }
}
