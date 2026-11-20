//! `servers/audio/effects/reverb_filter.*` and `audio_effect_reverb.*`.
//!
//! Ported from `reverb_filter.cpp`: the Freeverb tunings, the pre-delay echo
//! buffer with its feedback, the optional one-pole high-pass on the input, eight
//! damped comb filters with the room-size feedback law, four all-pass stages
//! (feedback 0.7) and the `wet * 0.6 + dry` output mix.
//!
//! `AudioEffectReverb` runs one `Reverb` per channel (the right channel with the
//! engine's `0.000521 s` extra spread), chunked at 1024 frames.
//!
//! The effect resource binding and the audio server are not ported; the mix rate
//! is passed to the constructors.

use crate::math::audio_frame::{undenormalize, AudioFrame};

pub const INPUT_BUFFER_MAX_SIZE: usize = 1024;
const MAX_COMBS: usize = 8;
const MAX_ALLPASS: usize = 4;
const MAX_ECHO_MS: f32 = 500.0;

const COMB_TUNINGS: [f32; MAX_COMBS] = [
    0.025306122448979593,
    0.026938775510204082,
    0.028956916099773241,
    0.03074829931972789,
    0.032244897959183672,
    0.03380952380952381,
    0.035306122448979592,
    0.036666666666666667,
];

const ALLPASS_TUNINGS: [f32; MAX_ALLPASS] = [
    0.0051020408163265302,
    0.007732426303854875,
    0.01,
    0.012607709750566893,
];

#[derive(Clone, Debug, Default)]
struct Comb {
    buffer: Vec<f32>,
    size: usize,
    feedback: f32,
    damp: f32,
    damp_h: f32,
    pos: usize,
    extra_spread_frames: usize,
}

#[derive(Clone, Debug, Default)]
struct AllPass {
    buffer: Vec<f32>,
    size: usize,
    pos: usize,
    extra_spread_frames: usize,
}

#[derive(Clone, Debug)]
pub struct Reverb {
    comb: [Comb; MAX_COMBS],
    allpass: [AllPass; MAX_ALLPASS],
    input_buffer: Vec<f32>,
    echo_buffer: Vec<f32>,
    echo_buffer_pos: usize,
    hpf_h1: f32,
    hpf_h2: f32,
    // Parameters.
    room_size: f32,
    damp: f32,
    wet: f32,
    dry: f32,
    mix_rate: f32,
    extra_spread_base: f32,
    extra_spread: f32,
    predelay: f32,
    predelay_fb: f32,
    hpf: f32,
}

impl Default for Reverb {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

impl Reverb {
    pub fn new(mix_rate: f32) -> Self {
        let mut reverb = Self {
            comb: Default::default(),
            allpass: Default::default(),
            input_buffer: vec![0.0; INPUT_BUFFER_MAX_SIZE],
            echo_buffer: Vec::new(),
            echo_buffer_pos: 0,
            hpf_h1: 0.0,
            hpf_h2: 0.0,
            room_size: 0.8,
            damp: 0.5,
            wet: 0.0,
            dry: 1.0,
            mix_rate,
            extra_spread_base: 0.0,
            extra_spread: 1.0,
            predelay: 150.0,
            predelay_fb: 0.4,
            hpf: 0.0,
        };
        reverb.configure_buffers();
        reverb.update_parameters();
        reverb
    }

    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size;
        self.update_parameters();
    }

    pub fn get_room_size(&self) -> f32 {
        self.room_size
    }

    pub fn set_damp(&mut self, damp: f32) {
        self.damp = damp;
        self.update_parameters();
    }

    pub fn get_damp(&self) -> f32 {
        self.damp
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

    pub fn set_predelay(&mut self, predelay: f32) {
        self.predelay = predelay;
    }

    pub fn get_predelay(&self) -> f32 {
        self.predelay
    }

    pub fn set_predelay_feedback(&mut self, feedback: f32) {
        self.predelay_fb = feedback;
    }

    pub fn get_predelay_feedback(&self) -> f32 {
        self.predelay_fb
    }

    /// `set_highpass`: the range is clamped to `0..=1`.
    pub fn set_highpass(&mut self, frequency: f32) {
        self.hpf = frequency.clamp(0.0, 1.0);
    }

    pub fn get_highpass(&self) -> f32 {
        self.hpf
    }

    pub fn set_extra_spread(&mut self, spread: f32) {
        self.extra_spread = spread;
    }

    pub fn get_extra_spread(&self) -> f32 {
        self.extra_spread
    }

    /// `set_mix_rate`: reconfigures the buffers like the engine.
    pub fn set_mix_rate(&mut self, mix_rate: f32) {
        self.mix_rate = mix_rate;
        self.configure_buffers();
        self.update_parameters();
    }

    pub fn get_mix_rate(&self) -> f32 {
        self.mix_rate
    }

    /// `set_extra_spread_base`: reconfigures the buffers.
    pub fn set_extra_spread_base(&mut self, seconds: f32) {
        self.extra_spread_base = seconds;
        self.configure_buffers();
        self.update_parameters();
    }

    pub fn get_echo_buffer_size(&self) -> usize {
        self.echo_buffer.len()
    }

    pub fn get_comb_size(&self, index: usize) -> usize {
        self.comb.get(index).map(|c| c.size).unwrap_or(0)
    }

    pub fn get_allpass_size(&self, index: usize) -> usize {
        self.allpass.get(index).map(|a| a.size).unwrap_or(0)
    }

    fn configure_buffers(&mut self) {
        let spread_frames = (self.extra_spread_base * self.mix_rate).round() as usize;
        for (index, comb) in self.comb.iter_mut().enumerate() {
            comb.extra_spread_frames = spread_frames;
            let mut len = (COMB_TUNINGS[index] * self.mix_rate).round() as usize
                + comb.extra_spread_frames;
            if len < 5 {
                len = 5;
            }
            comb.buffer = vec![0.0; len];
            comb.size = len;
            comb.pos = 0;
            comb.damp_h = 0.0;
        }
        for (index, allpass) in self.allpass.iter_mut().enumerate() {
            allpass.extra_spread_frames = spread_frames;
            let mut len = (ALLPASS_TUNINGS[index] * self.mix_rate).round() as usize
                + allpass.extra_spread_frames;
            if len < 5 {
                len = 5;
            }
            allpass.buffer = vec![0.0; len];
            allpass.size = len;
            allpass.pos = 0;
        }
        let echo_size = (MAX_ECHO_MS / 1000.0 * self.mix_rate + 1.0) as usize;
        self.echo_buffer = vec![0.0; echo_size];
        self.echo_buffer_pos = 0;
        self.hpf_h1 = 0.0;
        self.hpf_h2 = 0.0;
    }

    fn update_parameters(&mut self) {
        const ROOM_SCALE: f32 = 0.28;
        const ROOM_OFFSET: f32 = 0.7;
        let mut auxdmp = self.damp / 2.0 + 0.5;
        auxdmp *= auxdmp;
        let damp = (-std::f32::consts::TAU * auxdmp * 10000.0 / self.mix_rate).exp();
        for comb in self.comb.iter_mut() {
            let mut feedback = ROOM_OFFSET + self.room_size * ROOM_SCALE;
            if feedback < ROOM_OFFSET {
                feedback = ROOM_OFFSET;
            } else if feedback > ROOM_OFFSET + ROOM_SCALE {
                feedback = ROOM_OFFSET + ROOM_SCALE;
            }
            comb.feedback = feedback;
            comb.damp = damp;
        }
    }

    /// `process`: one mono channel, at most `INPUT_BUFFER_MAX_SIZE` frames.
    pub fn process(&mut self, src: &[f32], dst: &mut [f32]) {
        let frames = src.len().min(dst.len()).min(INPUT_BUFFER_MAX_SIZE);
        if frames == 0 || self.echo_buffer.is_empty() {
            return;
        }
        let echo_size = self.echo_buffer.len();

        let mut predelay_frames =
            (self.predelay / 1000.0 * self.mix_rate).round() as i64;
        if predelay_frames < 10 {
            predelay_frames = 10;
        }
        if predelay_frames >= echo_size as i64 {
            predelay_frames = echo_size as i64 - 1;
        }
        let predelay_frames = predelay_frames as usize;

        for index in 0..frames {
            if self.echo_buffer_pos >= echo_size {
                self.echo_buffer_pos = 0;
            }
            let mut read_pos = self.echo_buffer_pos as i64 - predelay_frames as i64;
            while read_pos < 0 {
                read_pos += echo_size as i64;
            }
            let input = undenormalize(
                self.echo_buffer[read_pos as usize] * self.predelay_fb + src[index],
            );
            self.echo_buffer[self.echo_buffer_pos] = input;
            self.input_buffer[index] = input;
            dst[index] = 0.0;
            self.echo_buffer_pos += 1;
        }

        if self.hpf > 0.0 {
            let hpaux = (-std::f32::consts::TAU * self.hpf * 6000.0 / self.mix_rate).exp();
            let hp_a1 = (1.0 + hpaux) / 2.0;
            let hp_a2 = -(1.0 + hpaux) / 2.0;
            let hp_b1 = hpaux;
            for index in 0..frames {
                let input = self.input_buffer[index];
                let out = input * hp_a1 + self.hpf_h1 * hp_a2 + self.hpf_h2 * hp_b1;
                self.input_buffer[index] = out;
                self.hpf_h2 = out;
                self.hpf_h1 = input;
            }
        }

        for comb in self.comb.iter_mut() {
            let size_limit = comb.size
                - (comb.extra_spread_frames as f32 * (1.0 - self.extra_spread)).round() as usize;
            for index in 0..frames {
                if comb.pos >= size_limit {
                    comb.pos = 0;
                }
                let mut out = undenormalize(comb.buffer[comb.pos] * comb.feedback);
                out = out * (1.0 - comb.damp) + comb.damp_h * comb.damp;
                comb.damp_h = out;
                comb.buffer[comb.pos] = self.input_buffer[index] + out;
                dst[index] += out;
                comb.pos += 1;
            }
        }

        const ALLPASS_FEEDBACK: f32 = 0.7;
        for allpass in self.allpass.iter_mut() {
            let size_limit = allpass.size
                - (allpass.extra_spread_frames as f32 * (1.0 - self.extra_spread)).round() as usize;
            for index in 0..frames {
                if allpass.pos >= size_limit {
                    allpass.pos = 0;
                }
                let aux = allpass.buffer[allpass.pos];
                allpass.buffer[allpass.pos] =
                    undenormalize(ALLPASS_FEEDBACK * aux + dst[index]);
                dst[index] = aux - ALLPASS_FEEDBACK * allpass.buffer[allpass.pos];
                allpass.pos += 1;
            }
        }

        const WET_SCALE: f32 = 0.6;
        for index in 0..frames {
            dst[index] = dst[index] * self.wet * WET_SCALE + src[index] * self.dry;
        }
    }
}

/// `AudioEffectReverb`: the two-channel wrapper.
#[derive(Clone, Debug)]
pub struct AudioEffectReverb {
    pub predelay: f32,
    pub predelay_fb: f32,
    pub hpf: f32,
    pub room_size: f32,
    pub damping: f32,
    pub spread: f32,
    pub dry: f32,
    pub wet: f32,
}

impl Default for AudioEffectReverb {
    fn default() -> Self {
        Self {
            predelay: 150.0,
            predelay_fb: 0.4,
            hpf: 0.0,
            room_size: 0.8,
            damping: 0.5,
            spread: 1.0,
            dry: 1.0,
            wet: 0.5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReverbInstance {
    reverb: [Reverb; 2],
    tmp_src: Vec<f32>,
    tmp_dst: Vec<f32>,
}

impl AudioEffectReverb {
    pub fn new(mix_rate: f32) -> Self {
        AudioEffectReverb::default().with_mix_rate(mix_rate)
    }

    /// A default effect bound to `mix_rate` (the constructor takes no arguments
    /// in this port, so `instantiate` uses the rate instead).
    pub fn with_mix_rate(self, _mix_rate: f32) -> Self {
        self
    }

    pub fn set_predelay_msec(&mut self, msec: f32) {
        self.predelay = msec;
    }

    pub fn get_predelay_msec(&self) -> f32 {
        self.predelay
    }

    /// `set_predelay_feedback`: clamped to `0..=0.98`.
    pub fn set_predelay_feedback(&mut self, feedback: f32) {
        self.predelay_fb = feedback.clamp(0.0, 0.98);
    }

    pub fn get_predelay_feedback(&self) -> f32 {
        self.predelay_fb
    }

    pub fn set_highpass(&mut self, hpf: f32) {
        self.hpf = hpf;
    }

    pub fn get_highpass(&self) -> f32 {
        self.hpf
    }

    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size;
    }

    pub fn get_room_size(&self) -> f32 {
        self.room_size
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn get_damping(&self) -> f32 {
        self.damping
    }

    pub fn set_spread(&mut self, spread: f32) {
        self.spread = spread;
    }

    pub fn get_spread(&self) -> f32 {
        self.spread
    }

    pub fn set_dry(&mut self, dry: f32) {
        self.dry = dry;
    }

    pub fn get_dry(&self) -> f32 {
        self.dry
    }

    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet;
    }

    pub fn get_wet(&self) -> f32 {
        self.wet
    }

    /// `instantiate`: left spread base 0, right `0.000521`.
    pub fn instantiate(&self, mix_rate: f32) -> ReverbInstance {
        let mut left = Reverb::new(mix_rate);
        left.set_extra_spread_base(0.0);
        let mut right = Reverb::new(mix_rate);
        right.set_extra_spread_base(0.000521);
        ReverbInstance {
            reverb: [left, right],
            tmp_src: vec![0.0; INPUT_BUFFER_MAX_SIZE],
            tmp_dst: vec![0.0; INPUT_BUFFER_MAX_SIZE],
        }
    }

    /// `process`.
    pub fn process(
        &self,
        instance: &mut ReverbInstance,
        src: &[AudioFrame],
        dst: &mut [AudioFrame],
        mix_rate: f32,
    ) {
        for reverb in instance.reverb.iter_mut() {
            if reverb.get_mix_rate() != mix_rate {
                reverb.set_mix_rate(mix_rate);
            }
            reverb.set_predelay(self.predelay);
            reverb.set_predelay_feedback(self.predelay_fb);
            reverb.set_highpass(self.hpf);
            reverb.set_room_size(self.room_size);
            reverb.set_damp(self.damping);
            reverb.set_extra_spread(self.spread);
            reverb.set_wet(self.wet);
            reverb.set_dry(self.dry);
        }

        let count = src.len().min(dst.len());
        let mut offset = 0usize;
        while offset < count {
            let todo = (count - offset).min(INPUT_BUFFER_MAX_SIZE);
            for index in 0..todo {
                instance.tmp_src[index] = src[offset + index].left;
            }
            instance.reverb[0].process(&instance.tmp_src[..todo], &mut instance.tmp_dst[..todo]);
            for index in 0..todo {
                dst[offset + index].left = instance.tmp_dst[index];
                instance.tmp_src[index] = src[offset + index].right;
            }
            instance.reverb[1].process(&instance.tmp_src[..todo], &mut instance.tmp_dst[..todo]);
            for index in 0..todo {
                dst[offset + index].right = instance.tmp_dst[index];
            }
            offset += todo;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_accessors() {
        let mut effect = AudioEffectReverb::new(44100.0);
        assert_eq!(effect.get_predelay_msec(), 150.0);
        assert_eq!(effect.get_predelay_feedback(), 0.4);
        assert_eq!(effect.get_highpass(), 0.0);
        assert_eq!(effect.get_room_size(), 0.8);
        assert_eq!(effect.get_damping(), 0.5);
        assert_eq!(effect.get_spread(), 1.0);
        assert_eq!(effect.get_dry(), 1.0);
        assert_eq!(effect.get_wet(), 0.5);

        effect.set_predelay_msec(20.0);
        effect.set_predelay_feedback(2.0); // clamped to 0.98
        effect.set_highpass(0.5);
        effect.set_room_size(0.2);
        effect.set_damping(0.9);
        effect.set_spread(0.0);
        effect.set_dry(0.25);
        effect.set_wet(0.75);
        assert_eq!(effect.get_predelay_msec(), 20.0);
        assert_eq!(effect.get_predelay_feedback(), 0.98);
        assert_eq!(effect.get_highpass(), 0.5);
        assert_eq!(effect.get_room_size(), 0.2);
        assert_eq!(effect.get_damping(), 0.9);
        assert_eq!(effect.get_spread(), 0.0);
        assert_eq!(effect.get_dry(), 0.25);
        assert_eq!(effect.get_wet(), 0.75);

        // The raw filter clamps the high-pass into 0..=1.
        let mut reverb = Reverb::new(44100.0);
        reverb.set_highpass(3.0);
        assert_eq!(reverb.get_highpass(), 1.0);
        reverb.set_highpass(-1.0);
        assert_eq!(reverb.get_highpass(), 0.0);
    }

    #[test]
    fn buffer_sizes_follow_the_tunings() {
        let reverb = Reverb::new(44100.0);
        // 0.0253061 * 44100 = 1116; the echo buffer holds 500 ms + 1.
        assert_eq!(reverb.get_comb_size(0), 1116);
        assert_eq!(reverb.get_allpass_size(0), 225);
        assert_eq!(reverb.get_echo_buffer_size(), 22051);
        // The engine clamps degenerate rates to the 5-frame minimum.
        let small = Reverb::new(1.0);
        assert_eq!(small.get_comb_size(0), 5);
        assert_eq!(small.get_allpass_size(0), 5);
    }

    #[test]
    fn dry_only_is_transparent() {
        let mut effect = AudioEffectReverb::new(44100.0);
        effect.set_wet(0.0);
        effect.set_dry(1.0);
        let input: Vec<AudioFrame> = (0..2048)
            .map(|index| AudioFrame::new(index as f32 * 0.0001, -(index as f32) * 0.0001))
            .collect();
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);
        for (a, b) in output.iter().zip(&input) {
            assert!((a.left - b.left).abs() < 1e-5);
            assert!((a.right - b.right).abs() < 1e-5);
        }
    }

    #[test]
    fn wet_path_produces_a_tail() {
        let mut effect = AudioEffectReverb::new(44100.0);
        effect.set_dry(0.0);
        effect.set_wet(1.0);
        effect.set_predelay_msec(1.0);
        let mut input = vec![AudioFrame::default(); 32768];
        input[0] = AudioFrame::new(1.0, 1.0);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(44100.0), &input, &mut output, 44100.0);

        let energy: f32 = output.iter().map(|frame| frame.left.abs()).sum();
        assert!(energy > 0.0, "the reverb must ring");
        // The tail starts after the first comb delay (~1116 frames at 44.1 kHz)
        // and decays: the final window is quieter than the loudest one.
        let early: f32 = output[1200..1500].iter().map(|frame| frame.left.abs()).sum();
        assert!(early > 0.0);
        let windows: Vec<f32> = output
            .chunks_exact(300)
            .map(|window| window.iter().map(|frame| frame.left.abs()).sum())
            .collect();
        let peak = windows.iter().cloned().fold(0.0f32, f32::max);
        let last = *windows.last().expect("windows");
        assert!(last < peak, "tail decays (last {last}, peak {peak})");
        // The pre-delay keeps the very first damped output near silence.
        assert!(output[0].left.abs() < 0.1);
    }

    #[test]
    fn stereo_spread_makes_the_channels_differ() {
        let mut effect = AudioEffectReverb::new(48000.0);
        effect.set_dry(0.0);
        effect.set_wet(1.0);
        effect.set_predelay_msec(1.0);
        let mut input = vec![AudioFrame::default(); 4096];
        input[0] = AudioFrame::new(1.0, 1.0);
        let mut output = vec![AudioFrame::default(); input.len()];
        effect.process(&mut effect.instantiate(48000.0), &input, &mut output, 48000.0);
        let differs = output
            .iter()
            .any(|frame| (frame.left - frame.right).abs() > 1e-6);
        assert!(differs, "extra_spread_base offsets the right channel");
    }
}
