//! `servers/audio/audio_stream.*` (`AudioStreamPlaybackResampled`): the
//! fixed-point cubic resampler shared by stream playbacks.
//!
//! Ported from `AudioStreamPlaybackResampled`: the 16.16 fixed-point mix offset,
//! the 128-frame internal buffer with four frames of interpolation history, the
//! cubic Hermite weights (`0.5`-scaled `(y2-y0)`/`(y3-y1)` terms), the
//! buffer-shift + refill loop, the "first silent frame" accounting and
//! `begin_resample`.
//!
//! The engine's `_mix_internal`/`get_stream_sampling_rate` virtuals become the
//! caller's `mix_internal` callback and `stream_rate` argument; the audio server
//! supplies the target rate and playback speed scale.

use crate::math::audio_frame::AudioFrame;

pub const FP_BITS: u32 = 16;
pub const FP_LEN: u64 = 1 << FP_BITS;
pub const FP_MASK: u64 = FP_LEN - 1;
pub const INTERNAL_BUFFER_LEN: usize = 128;
pub const CUBIC_INTERP_HISTORY: usize = 4;

#[derive(Clone, Debug)]
pub struct ResampledPlayback {
    internal_buffer: [AudioFrame; INTERNAL_BUFFER_LEN + CUBIC_INTERP_HISTORY],
    /// `-1` means "the buffer has no silent frame" (the engine stores
    /// `(unsigned)-1`).
    internal_buffer_end: i64,
    mix_offset: u64,
}

impl Default for ResampledPlayback {
    fn default() -> Self {
        Self::new()
    }
}

impl ResampledPlayback {
    pub fn new() -> Self {
        Self {
            internal_buffer: [AudioFrame::default(); INTERNAL_BUFFER_LEN + CUBIC_INTERP_HISTORY],
            internal_buffer_end: -1,
            mix_offset: 0,
        }
    }

    /// `begin_resample`: clears the interpolation history, pre-fills the buffer
    /// with `mix_internal` and resets the offset.
    pub fn begin_resample<F>(&mut self, mix_internal: &mut F)
    where
        F: FnMut(&mut [AudioFrame]) -> usize,
    {
        for frame in self.internal_buffer.iter_mut().take(CUBIC_INTERP_HISTORY) {
            *frame = AudioFrame::new(0.0, 0.0);
        }
        mix_internal(&mut self.internal_buffer[CUBIC_INTERP_HISTORY..]);
        self.mix_offset = 0;
    }

    /// `mix`: `stream_rate` is the playback's own sampling rate, `rate_scale`
    /// the instance's pitch scale, `playback_speed_scale` the global scale and
    /// `target_rate` the audio-server mix rate. Returns the number of frames
    /// that were mixed before the internal buffer ran out.
    pub fn mix<F>(
        &mut self,
        buffer: &mut [AudioFrame],
        rate_scale: f32,
        target_rate: f32,
        playback_speed_scale: f32,
        stream_rate: f32,
        mix_internal: &mut F,
    ) -> usize
    where
        F: FnMut(&mut [AudioFrame]) -> usize,
    {
        let mix_increment =
            (((stream_rate * rate_scale * playback_speed_scale) / target_rate as f32) as f64
                * FP_LEN as f64) as u64;
        let frames = buffer.len();
        let mut mixed_frames_total: i64 = -1;

        for i in 0..frames {
            let idx = CUBIC_INTERP_HISTORY + (self.mix_offset >> FP_BITS) as usize;
            let mu = (self.mix_offset & FP_MASK) as f32 / FP_LEN as f32;

            let y0 = self.internal_buffer[idx - 3];
            let y1 = self.internal_buffer[idx - 2];
            let y2 = self.internal_buffer[idx - 1];
            let y3 = self.internal_buffer[idx];

            if self.internal_buffer_end >= 0
                && idx as i64 >= self.internal_buffer_end
                && mixed_frames_total == -1
            {
                // The internal buffer ends somewhere in this range and the number
                // of good frames has not been recorded yet.
                mixed_frames_total = i as i64;
            }

            let mu2 = mu * mu;
            let h11 = mu2 * (mu - 1.0);
            let z = mu2 - h11;
            let h01 = z - h11;
            let h10 = mu - z;

            buffer[i] = AudioFrame::new(
                y1.left
                    + (y2.left - y1.left) * h01
                    + ((y2.left - y0.left) * h10 + (y3.left - y1.left) * h11) * 0.5,
                y1.right
                    + (y2.right - y1.right) * h01
                    + ((y2.right - y0.right) * h10 + (y3.right - y1.right) * h11) * 0.5,
            );

            self.mix_offset += mix_increment;

            while (self.mix_offset >> FP_BITS) >= INTERNAL_BUFFER_LEN as u64 {
                for offset in 0..CUBIC_INTERP_HISTORY {
                    self.internal_buffer[offset] =
                        self.internal_buffer[INTERNAL_BUFFER_LEN + offset];
                }
                let mixed =
                    mix_internal(&mut self.internal_buffer[CUBIC_INTERP_HISTORY..]);
                if mixed != INTERNAL_BUFFER_LEN {
                    // `internal_buffer[mixed]` is the first frame of silence.
                    self.internal_buffer_end = mixed as i64;
                } else {
                    self.internal_buffer_end = -1;
                }
                self.mix_offset -= (INTERNAL_BUFFER_LEN as u64) << FP_BITS;
            }
        }

        if mixed_frames_total == -1 && frames > 0 {
            mixed_frames_total = frames as i64;
        }
        mixed_frames_total.max(0) as usize
    }

    pub fn get_mix_offset(&self) -> u64 {
        self.mix_offset
    }

    pub fn get_internal_buffer_end(&self) -> i64 {
        self.internal_buffer_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fills the internal buffer from a ramp and reports a short mix after it
    /// runs out.
    struct RampSource {
        frames: Vec<AudioFrame>,
        position: usize,
    }

    impl RampSource {
        fn new(count: usize) -> Self {
            Self {
                frames: (0..count)
                    .map(|index| AudioFrame::new(index as f32, -(index as f32)))
                    .collect(),
                position: 0,
            }
        }

        fn fill(&mut self, out: &mut [AudioFrame]) -> usize {
            let remaining = self.frames.len().saturating_sub(self.position);
            let take = remaining.min(out.len());
            out[..take].copy_from_slice(&self.frames[self.position..self.position + take]);
            for frame in out.iter_mut().skip(take) {
                *frame = AudioFrame::default();
            }
            self.position += take;
            take
        }
    }

    #[test]
    fn unity_rate_delays_by_two_frames() {
        let mut playback = ResampledPlayback::new();
        let mut source = RampSource::new(1000);
        playback.begin_resample(&mut |out| source.fill(out));

        let mut output = vec![AudioFrame::default(); 8];
        let mixed = playback.mix(&mut output, 1.0, 44100.0, 1.0, 44100.0, &mut |out| {
            source.fill(out)
        });
        assert_eq!(mixed, 8);
        // The cubic history delays the signal by two frames at unity rate.
        assert_eq!(output[0], AudioFrame::default());
        assert_eq!(output[1], AudioFrame::default());
        assert_eq!(output[2], AudioFrame::new(0.0, 0.0));
        assert!((output[3].left - 1.0).abs() < 1e-5, "third {output:?}");
        assert!((output[4].left - 2.0).abs() < 1e-5);
        assert_eq!(FP_LEN, 65536);
    }

    #[test]
    fn double_rate_advances_two_source_frames_per_output() {
        let mut playback = ResampledPlayback::new();
        let mut source = RampSource::new(1000);
        playback.begin_resample(&mut |out| source.fill(out));

        let mut output = vec![AudioFrame::default(); 6];
        playback.mix(&mut output, 2.0, 44100.0, 1.0, 44100.0, &mut |out| {
            source.fill(out)
        });
        // At 2x the interpolator reads source frames 0, 2, 4, 6... after the
        // two-frame history.
        assert!((output[2].left - 2.0).abs() < 1e-4, "frames {output:?}");
        assert!((output[3].left - 4.0).abs() < 1e-4);
    }

    #[test]
    fn half_rate_needs_two_outputs_per_source_frame() {
        let mut playback = ResampledPlayback::new();
        let mut source = RampSource::new(1000);
        playback.begin_resample(&mut |out| source.fill(out));

        let mut output = vec![AudioFrame::default(); 8];
        playback.mix(&mut output, 0.5, 44100.0, 1.0, 44100.0, &mut |out| {
            source.fill(out)
        });
        // The interpolator steps half a source frame per output frame.
        assert!((output[6].left - 1.0).abs() < 1e-3, "frames {output:?}");
        assert!(output[7].left > output[6].left);
    }

    #[test]
    fn short_callback_limits_the_mixed_frame_count() {
        let mut playback = ResampledPlayback::new();
        // `begin_resample` ignores the pre-fill's short-mix report, like the
        // engine (the sentinel stays at "no silent frame").
        let mut source = RampSource::new(6);
        playback.begin_resample(&mut |out| source.fill(out));
        assert_eq!(playback.get_internal_buffer_end(), -1);

        // Enough output frames to force a refill, which now reports silence.
        let mut output = vec![AudioFrame::default(); 256];
        let mixed = playback.mix(&mut output, 1.0, 44100.0, 1.0, 44100.0, &mut |out| {
            source.fill(out)
        });
        assert!(mixed < 256, "mixed {mixed} frames");
        assert_eq!(playback.get_internal_buffer_end(), 0);
        assert_eq!(playback.get_mix_offset() % FP_LEN, 0);
    }

    #[test]
    fn buffer_shifts_keep_the_offset_bounded() {
        let mut playback = ResampledPlayback::new();
        let mut source = RampSource::new(4096);
        playback.begin_resample(&mut |out| source.fill(out));

        let mut output = vec![AudioFrame::default(); 1000];
        playback.mix(&mut output, 1.0, 44100.0, 1.0, 44100.0, &mut |out| {
            source.fill(out)
        });
        // The offset always stays inside one internal buffer.
        assert!((playback.get_mix_offset() >> FP_BITS) < INTERNAL_BUFFER_LEN as u64);
        assert!(output.iter().all(|frame| frame.left.is_finite()));

        // Playback speed scales the increment too.
        let mut fast = ResampledPlayback::new();
        let mut fast_source = RampSource::new(4096);
        fast.begin_resample(&mut |out| fast_source.fill(out));
        let mut fast_output = vec![AudioFrame::default(); 64];
        fast.mix(&mut fast_output, 1.0, 44100.0, 2.0, 44100.0, &mut |out| {
            fast_source.fill(out)
        });
        assert!(fast_output[20].left > output[20].left);
    }
}
