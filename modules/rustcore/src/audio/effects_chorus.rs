//! `servers/audio/effects/audio_effect_chorus.*`: the chorus effect.
//!
//! Ported from `audio_effect_chorus.cpp`: the four voices (delay ms, rate Hz,
//! depth ms, level dB, cutoff, pan), the dry/wet mix, the power-of-two ring
//! buffer sized from `MAX_DELAY_MS + MAX_DEPTH_MS + MAX_WIDTH_MS` doubled, the
//! 256-frame chunking, the 16.16 phase accumulators and per-voice one-pole
//! filters, the sine LFO delay modulation with linear interpolation, and the
//! read-ahead guard.
//!
//! The effect resource binding and the audio server are not ported; processing
//! takes the mix rate explicitly.

use crate::math::audio_frame::{undenormalize, AudioFrame};
use crate::math::funcs;

pub const MAX_DELAY_MS: i32 = 50;
pub const MAX_DEPTH_MS: i32 = 20;
pub const MAX_WIDTH_MS: i32 = 50;
pub const MAX_VOICES: usize = 4;
pub const CYCLES_FRAC: i32 = 16;
pub const CYCLES_MASK: u64 = (1 << CYCLES_FRAC) - 1;
pub const MS_CUTOFF_MAX: f32 = 16000.0;
const CHUNK: usize = 256;

#[derive(Clone, Debug)]
pub struct ChorusVoice {
    pub delay: f32,
    pub rate: f32,
    pub depth: f32,
    pub level: f32,
    pub cutoff: f32,
    pub pan: f32,
}

impl Default for ChorusVoice {
    fn default() -> Self {
        Self {
            delay: 12.0,
            rate: 1.0,
            depth: 0.0,
            level: 0.0,
            cutoff: MS_CUTOFF_MAX,
            pan: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioEffectChorus {
    pub voices: [ChorusVoice; MAX_VOICES],
    pub voice_count: usize,
    pub wet: f32,
    pub dry: f32,
}

impl Default for AudioEffectChorus {
    fn default() -> Self {
        let mut voices = [
            ChorusVoice::default(),
            ChorusVoice::default(),
            ChorusVoice::default(),
            ChorusVoice::default(),
        ];
        voices[0] = ChorusVoice {
            delay: 15.0,
            rate: 0.8,
            depth: 2.0,
            level: 0.0,
            cutoff: 8000.0,
            pan: -0.5,
        };
        voices[1] = ChorusVoice {
            delay: 20.0,
            rate: 1.2,
            depth: 3.0,
            level: 0.0,
            cutoff: 8000.0,
            pan: 0.5,
        };
        Self {
            voices,
            voice_count: 2,
            wet: 0.5,
            dry: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChorusInstance {
    audio_buffer: Vec<AudioFrame>,
    buffer_mask: usize,
    buffer_pos: usize,
    filter_h: [AudioFrame; MAX_VOICES],
    cycles: [u64; MAX_VOICES],
}

fn frame_mul(frame: AudioFrame, scale: f32) -> AudioFrame {
    AudioFrame::new(frame.left * scale, frame.right * scale)
}

impl AudioEffectChorus {
    pub fn new() -> Self {
        Self::default()
    }

    /// `instantiate`: sizes the ring buffer from the mix rate.
    pub fn instantiate(&self, sample_rate: f32) -> ChorusInstance {
        let mut ring_buffer_max_size =
            (MAX_DELAY_MS + MAX_DEPTH_MS + MAX_WIDTH_MS) as f32;
        ring_buffer_max_size *= 2.0;
        ring_buffer_max_size /= 1000.0;
        ring_buffer_max_size *= sample_rate;
        let mut ringbuff_size = ring_buffer_max_size as usize;
        let mut bits = 0u32;
        while ringbuff_size > 0 {
            bits += 1;
            ringbuff_size /= 2;
        }
        let ringbuff_size = 1usize << bits;
        ChorusInstance {
            audio_buffer: vec![AudioFrame::default(); ringbuff_size],
            buffer_mask: ringbuff_size - 1,
            buffer_pos: 0,
            filter_h: [AudioFrame::default(); MAX_VOICES],
            cycles: [0; MAX_VOICES],
        }
    }

    pub fn set_voice_count(&mut self, voices: i32) -> bool {
        if !(1..=MAX_VOICES as i32).contains(&voices) {
            return false;
        }
        self.voice_count = voices as usize;
        true
    }

    pub fn get_voice_count(&self) -> usize {
        self.voice_count
    }

    pub fn set_voice_delay_ms(&mut self, voice: usize, delay: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.delay = delay;
        }
    }

    pub fn get_voice_delay_ms(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.delay).unwrap_or(0.0)
    }

    pub fn set_voice_rate_hz(&mut self, voice: usize, rate: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.rate = rate;
        }
    }

    pub fn get_voice_rate_hz(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.rate).unwrap_or(0.0)
    }

    pub fn set_voice_depth_ms(&mut self, voice: usize, depth: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.depth = depth;
        }
    }

    pub fn get_voice_depth_ms(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.depth).unwrap_or(0.0)
    }

    pub fn set_voice_level_db(&mut self, voice: usize, level: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.level = level;
        }
    }

    pub fn get_voice_level_db(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.level).unwrap_or(0.0)
    }

    pub fn set_voice_cutoff_hz(&mut self, voice: usize, cutoff: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.cutoff = cutoff;
        }
    }

    pub fn get_voice_cutoff_hz(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.cutoff).unwrap_or(0.0)
    }

    pub fn set_voice_pan(&mut self, voice: usize, pan: f32) {
        if let Some(entry) = self.voices.get_mut(voice) {
            entry.pan = pan;
        }
    }

    pub fn get_voice_pan(&self, voice: usize) -> f32 {
        self.voices.get(voice).map(|entry| entry.pan).unwrap_or(0.0)
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet;
    }

    pub fn get_wet(&self) -> f32 {
        self.wet
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }

    /// `process`: 256-frame chunks like the engine.
    pub fn process(
        &self,
        instance: &mut ChorusInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        let count = src.len().min(dst.len());
        let mut offset = 0usize;
        while offset < count {
            let todo = (count - offset).min(CHUNK);
            self.process_chunk(
                instance,
                &src[offset..offset + todo],
                &mut dst[offset..offset + todo],
                sample_rate,
            );
            offset += todo;
        }
    }

    fn process_chunk(
        &self,
        instance: &mut ChorusInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        sample_rate: f32,
    ) {
        // Fill the ring buffer and apply the dry path.
        for index in 0..src.len() {
            instance.audio_buffer[(instance.buffer_pos + index) & instance.buffer_mask] = src[index];
            dst[index] = frame_mul(src[index], self.dry);
        }

        let frames = src.len();
        for voice_index in 0..self.voice_count.min(MAX_VOICES) {
            let voice = &self.voices[voice_index];
            let time_to_mix = frames as f64 / sample_rate as f64;
            let cycles_to_mix = time_to_mix * voice.rate as f64;

            let mut local_rb_pos = instance.buffer_pos;
            let delay_msec = voice.delay as f64;
            let mut delay_frames = ((delay_msec / 1000.0) * sample_rate as f64) as i64;
            let max_depth_frames = (voice.depth / 1000.0) * sample_rate;

            let mut local_cycles = instance.cycles[voice_index];
            let increment = (cycles_to_mix / frames as f64
                * (1u64 << CYCLES_FRAC) as f64)
                .round() as u64;

            // Prevent the LFO from reading ahead of the write position.
            if (max_depth_frames as i64 + 10) > delay_frames {
                delay_frames += max_depth_frames as i64 - delay_frames;
                delay_frames += 10;
            }

            if voice.cutoff == 0.0 {
                continue;
            }
            let auxlp = (-std::f64::consts::TAU * voice.cutoff as f64 / sample_rate as f64).exp()
                as f32;
            let mut c1 = 1.0 - auxlp;
            let mut c2 = auxlp;
            let mut h = instance.filter_h[voice_index];
            if voice.cutoff >= MS_CUTOFF_MAX {
                c1 = 1.0;
                c2 = 0.0;
            }

            let wet_scale = funcs::db_to_linear(voice.level as f64) as f32 * self.wet;
            let vol_modifier = AudioFrame::new(
                wet_scale * (1.0 - voice.pan).clamp(0.0, 1.0),
                wet_scale * (1.0 + voice.pan).clamp(0.0, 1.0),
            );

            for index in 0..frames {
                let phase = (local_cycles & CYCLES_MASK) as f32 / (1u64 << CYCLES_FRAC) as f32;
                let wave_delay = (phase * std::f32::consts::TAU).sin() * max_depth_frames;
                let wave_delay_frames = wave_delay.floor() as i64;
                let wave_delay_frac = wave_delay - wave_delay_frames as f32;

                let rb_source = local_rb_pos as i64 - delay_frames - wave_delay_frames;
                let mask = instance.buffer_mask as i64;
                let val = instance.audio_buffer[(rb_source & mask) as usize];
                let val_next = instance.audio_buffer[((rb_source - 1) & mask) as usize];
                let mut val = AudioFrame::new(
                    val.left + (val_next.left - val.left) * wave_delay_frac,
                    val.right + (val_next.right - val.right) * wave_delay_frac,
                );

                let filtered = AudioFrame::new(
                    undenormalize(val.left * c1 + h.left * c2),
                    undenormalize(val.right * c1 + h.right * c2),
                );
                h = filtered;
                val = filtered;

                dst[index] = AudioFrame::new(
                    dst[index].left + val.left * vol_modifier.left,
                    dst[index].right + val.right * vol_modifier.right,
                );
                local_cycles += increment;
                local_rb_pos += 1;
            }

            instance.filter_h[voice_index] = h;
            instance.cycles[voice_index] +=
                (cycles_to_mix * (1u64 << CYCLES_FRAC) as f64) as u64;
        }

        instance.buffer_pos += frames;
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
    fn defaults_and_accessors() {
        let mut effect = AudioEffectChorus::new();
        assert_eq!(effect.get_voice_count(), 2);
        assert_eq!(effect.get_voice_delay_ms(0), 15.0);
        assert_eq!(effect.get_voice_rate_hz(0), 0.8);
        assert_eq!(effect.get_voice_depth_ms(0), 2.0);
        assert_eq!(effect.get_voice_level_db(0), 0.0);
        assert_eq!(effect.get_voice_cutoff_hz(0), 8000.0);
        assert_eq!(effect.get_voice_pan(0), -0.5);
        assert_eq!(effect.get_voice_delay_ms(1), 20.0);
        assert_eq!(effect.get_voice_pan(1), 0.5);
        assert_eq!(effect.get_wet(), 0.5);
        assert_eq!(effect.get_dry(), 1.0);
        // The unused voices keep the base defaults.
        assert_eq!(effect.get_voice_delay_ms(2), 12.0);
        assert_eq!(effect.get_voice_pan(2), 0.0);

        assert!(!effect.set_voice_count(0));
        assert!(!effect.set_voice_count(5));
        assert!(effect.set_voice_count(4));
        assert_eq!(effect.get_voice_count(), 4);

        effect.set_voice_delay_ms(1, 33.0);
        effect.set_voice_rate_hz(1, 2.0);
        effect.set_voice_depth_ms(1, 4.0);
        effect.set_voice_level_db(1, -3.0);
        effect.set_voice_cutoff_hz(1, 4000.0);
        effect.set_voice_pan(1, 0.25);
        effect.set_wet(0.75);
        effect.set_dry(0.25);
        assert_eq!(effect.get_voice_delay_ms(1), 33.0);
        assert_eq!(effect.get_voice_rate_hz(1), 2.0);
        assert_eq!(effect.get_voice_depth_ms(1), 4.0);
        assert_eq!(effect.get_voice_level_db(1), -3.0);
        assert_eq!(effect.get_voice_cutoff_hz(1), 4000.0);
        assert_eq!(effect.get_voice_pan(1), 0.25);
        assert_eq!(effect.get_wet(), 0.75);
        assert_eq!(effect.get_dry(), 0.25);
        assert_eq!(MAX_VOICES, 4);
        assert_eq!(CYCLES_MASK, 65535);
    }

    #[test]
    fn dry_only_is_identity() {
        let mut effect = AudioEffectChorus::new();
        effect.set_voice_count(1);
        effect.set_voice_cutoff_hz(0, 0.0); // skip the voice
        let input: Vec<AudioFrame> = (0..700)
            .map(|index| AudioFrame::new(index as f32 * 0.01, -(index as f32) * 0.01))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        assert_eq!(output, input);

        // The dry gain scales the output when no voice runs.
        effect.set_dry(0.5);
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        assert!((output[100].left - input[100].left * 0.5).abs() < 1e-6);
    }

    #[test]
    fn single_voice_delays_the_signal() {
        let mut effect = AudioEffectChorus::new();
        effect.set_voice_count(1);
        effect.set_voice_delay_ms(0, 10.0);
        effect.set_voice_depth_ms(0, 0.0);
        effect.set_voice_rate_hz(0, 0.0);
        effect.set_voice_level_db(0, 0.0);
        effect.set_voice_cutoff_hz(0, MS_CUTOFF_MAX);
        effect.set_voice_pan(0, 0.0);
        effect.set_dry(0.0);
        effect.set_wet(1.0);

        let rate = 48000.0;
        let input = impulse(4096);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let delayed = (0.010 * rate) as usize;
        assert!((output[delayed].left - 1.0).abs() < 1e-4, "voice echo at {delayed}");
        assert!((output[delayed].right - 1.0).abs() < 1e-4);
        assert!(output[0].left.abs() < 1e-6, "dry is muted");
        // Beyond the echo the ring is silent.
        assert!(output[delayed + 10].left.abs() < 1e-6);
    }

    #[test]
    fn lfo_modulation_moves_the_delay() {
        // With depth and a nonzero rate the echo spreads over neighbouring
        // frames instead of a single one.
        let mut effect = AudioEffectChorus::new();
        effect.set_voice_count(1);
        effect.set_voice_delay_ms(0, 20.0);
        effect.set_voice_depth_ms(0, 5.0);
        effect.set_voice_rate_hz(0, 2.0);
        effect.set_voice_level_db(0, 0.0);
        effect.set_voice_cutoff_hz(0, MS_CUTOFF_MAX);
        effect.set_voice_pan(0, 0.0);
        effect.set_dry(0.0);
        effect.set_wet(1.0);

        let rate = 44100.0;
        let mut input = vec![AudioFrame::default(); 44100];
        for frame in input.iter_mut() {
            *frame = AudioFrame::new(0.1, 0.1);
        }
        // A short burst in the middle.
        for frame in input.iter_mut().skip(1000).take(20) {
            *frame = AudioFrame::new(1.0, 1.0);
        }
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);

        let energy: f32 = output.iter().map(|frame| frame.left.abs()).sum();
        assert!(energy > 1.0, "modulated voice produces output");

        // The same voice without depth reads a fixed offset, so any difference
        // between the two runs is the LFO moving the read position.
        let mut plain = AudioEffectChorus::new();
        plain.set_voice_count(1);
        plain.set_voice_delay_ms(0, 20.0);
        plain.set_voice_depth_ms(0, 0.0);
        plain.set_voice_rate_hz(0, 2.0);
        plain.set_voice_level_db(0, 0.0);
        plain.set_voice_cutoff_hz(0, MS_CUTOFF_MAX);
        plain.set_voice_pan(0, 0.0);
        plain.set_dry(0.0);
        plain.set_wet(1.0);
        let mut fixed = vec![AudioFrame::default(); input.len()];
        plain.process(&mut plain.instantiate(rate), &input, &mut fixed, rate);

        let differs = output
            .iter()
            .zip(&fixed)
            .any(|(a, b)| (a.left - b.left).abs() > 1e-6);
        assert!(differs, "the LFO moves the read position");
    }

    #[test]
    fn voices_sum_into_the_output() {
        let mut effect = AudioEffectChorus::new();
        effect.set_voice_count(2);
        for voice in 0..2 {
            effect.set_voice_delay_ms(voice, 10.0);
            effect.set_voice_depth_ms(voice, 0.0);
            effect.set_voice_rate_hz(voice, 0.0);
            effect.set_voice_level_db(voice, 0.0);
            effect.set_voice_cutoff_hz(voice, MS_CUTOFF_MAX);
            effect.set_voice_pan(voice, 0.0);
        }
        effect.set_dry(0.0);
        effect.set_wet(1.0);

        let rate = 48000.0;
        let input = impulse(4096);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(rate), &input, &mut output, rate);
        let delayed = (0.010 * rate) as usize;
        // Both voices contribute the same echo.
        assert!((output[delayed].left - 2.0).abs() < 1e-4);
    }
}
