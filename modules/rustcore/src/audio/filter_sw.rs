//! `servers/audio/audio_filter_sw.*`: the biquad filter used by the audio
//! effects.
//!
//! Ported from `audio_filter_sw.cpp`: the eight modes (`BANDPASS`, `HIGHPASS`,
//! `LOWPASS`, `NOTCH`, `PEAK`, `BANDLIMIT`, `LOWSHELF`, `HIGHSHELF`), the
//! coefficient construction (cutoff clamped to the sample rate, resonance/gain
//! floors, multi-stage Q/gain scaling, the band-limit and shelf variants), the
//! frequency response, and the `Processor` state machine (history, coefficient
//! interpolation per buffer, blocked processing with a stride).
//!
//! The effect resources that own filters are ported separately; the object
//! binding is not.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coeffs {
    pub a1: f64,
    pub a2: f64,
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FilterMode {
    Bandpass = 0,
    Highpass = 1,
    Lowpass = 2,
    Notch = 3,
    Peak = 4,
    Bandlimit = 5,
    Lowshelf = 6,
    Highshelf = 7,
}

#[derive(Clone, Debug)]
pub struct AudioFilterSw {
    cutoff: f32,
    resonance: f32,
    gain: f32,
    sampling_rate: f32,
    stages: i32,
    mode: FilterMode,
}

impl Default for AudioFilterSw {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioFilterSw {
    /// Header defaults: 5 kHz cutoff, 0.5 resonance, unit gain, 44.1 kHz, one
    /// stage, low-pass.
    pub fn new() -> Self {
        Self {
            cutoff: 5000.0,
            resonance: 0.5,
            gain: 1.0,
            sampling_rate: 44100.0,
            stages: 1,
            mode: FilterMode::Lowpass,
        }
    }

    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> FilterMode {
        self.mode
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
    }

    pub fn get_cutoff(&self) -> f32 {
        self.cutoff
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }

    pub fn get_resonance(&self) -> f32 {
        self.resonance
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    pub fn set_sampling_rate(&mut self, rate: f32) {
        self.sampling_rate = rate;
    }

    pub fn get_sampling_rate(&self) -> f32 {
        self.sampling_rate
    }

    pub fn set_stages(&mut self, stages: i32) {
        self.stages = stages;
    }

    pub fn get_stages(&self) -> i32 {
        self.stages
    }

    /// `prepare_coefficients`.
    pub fn prepare_coefficients(&self, coeffs: &mut Coeffs) {
        let sr_limit = (self.sampling_rate / 2.0) + 512.0;
        let mut final_cutoff = if self.cutoff > sr_limit {
            sr_limit
        } else {
            self.cutoff
        } as f64;
        if final_cutoff < 1.0 {
            final_cutoff = 1.0;
        }

        let mut omega = std::f64::consts::TAU * final_cutoff / self.sampling_rate as f64;
        let sin_v = omega.sin();
        let cos_v = omega.cos();

        let mut q = self.resonance as f64;
        if q <= 0.0 {
            q = 0.0001;
        }
        if self.mode == FilterMode::Bandpass {
            q *= 2.0;
        } else if self.mode == FilterMode::Peak {
            q *= 3.0;
        }

        let mut tmpgain = self.gain as f64;
        if tmpgain < 0.001 {
            tmpgain = 0.001;
        }
        if self.stages > 1 {
            if q > 1.0 {
                q = q.powf(1.0 / self.stages as f64);
            }
            tmpgain = tmpgain.powf(1.0 / (self.stages + 1) as f64);
        }

        let mut alpha = sin_v / (2.0 * q);
        let mut a0 = 1.0 + alpha;

        match self.mode {
            FilterMode::Lowpass => {
                coeffs.b0 = (1.0 - cos_v) / 2.0;
                coeffs.b1 = 1.0 - cos_v;
                coeffs.b2 = (1.0 - cos_v) / 2.0;
                coeffs.a1 = -2.0 * cos_v;
                coeffs.a2 = 1.0 - alpha;
            }
            FilterMode::Highpass => {
                coeffs.b0 = (1.0 + cos_v) / 2.0;
                coeffs.b1 = -(1.0 + cos_v);
                coeffs.b2 = (1.0 + cos_v) / 2.0;
                coeffs.a1 = -2.0 * cos_v;
                coeffs.a2 = 1.0 - alpha;
            }
            FilterMode::Bandpass => {
                coeffs.b0 = alpha * (q + 1.0).sqrt();
                coeffs.b1 = 0.0;
                coeffs.b2 = -alpha * (q + 1.0).sqrt();
                coeffs.a1 = -2.0 * cos_v;
                coeffs.a2 = 1.0 - alpha;
            }
            FilterMode::Notch => {
                coeffs.b0 = 1.0;
                coeffs.b1 = -2.0 * cos_v;
                coeffs.b2 = 1.0;
                coeffs.a1 = -2.0 * cos_v;
                coeffs.a2 = 1.0 - alpha;
            }
            FilterMode::Peak => {
                coeffs.b0 = 1.0 + alpha * tmpgain;
                coeffs.b1 = -2.0 * cos_v;
                coeffs.b2 = 1.0 - alpha * tmpgain;
                coeffs.a1 = -2.0 * cos_v;
                coeffs.a2 = 1.0 - alpha / tmpgain;
            }
            FilterMode::Bandlimit => {
                let hicutoff = self.resonance as f64;
                let centercutoff = (self.cutoff as f64 + self.resonance as f64) / 2.0;
                let bandwidth =
                    (centercutoff.ln() - hicutoff.ln()) / 2.0f64.ln();
                omega = std::f64::consts::TAU * centercutoff / self.sampling_rate as f64;
                alpha = omega.sin()
                    * ((2.0f64.ln() / 2.0 * bandwidth * omega) / omega.sin()).sinh();
                a0 = 1.0 + alpha;

                coeffs.b0 = alpha;
                coeffs.b1 = 0.0;
                coeffs.b2 = -alpha;
                coeffs.a1 = -2.0 * omega.cos();
                coeffs.a2 = 1.0 - alpha;
            }
            FilterMode::Lowshelf => {
                let mut tmpq = q.sqrt();
                if tmpq <= 0.0 {
                    tmpq = 0.001;
                }
                let beta = tmpgain.sqrt() / tmpq;

                a0 = (tmpgain + 1.0) + (tmpgain - 1.0) * cos_v + beta * sin_v;
                coeffs.b0 =
                    tmpgain * ((tmpgain + 1.0) - (tmpgain - 1.0) * cos_v + beta * sin_v);
                coeffs.b1 = 2.0 * tmpgain * ((tmpgain - 1.0) - (tmpgain + 1.0) * cos_v);
                coeffs.b2 =
                    tmpgain * ((tmpgain + 1.0) - (tmpgain - 1.0) * cos_v - beta * sin_v);
                coeffs.a1 = -2.0 * ((tmpgain - 1.0) + (tmpgain + 1.0) * cos_v);
                coeffs.a2 = (tmpgain + 1.0) + (tmpgain - 1.0) * cos_v - beta * sin_v;
            }
            FilterMode::Highshelf => {
                let mut tmpq = q.sqrt();
                if tmpq <= 0.0 {
                    tmpq = 0.001;
                }
                let beta = tmpgain.sqrt() / tmpq;

                a0 = (tmpgain + 1.0) - (tmpgain - 1.0) * cos_v + beta * sin_v;
                coeffs.b0 =
                    tmpgain * ((tmpgain + 1.0) + (tmpgain - 1.0) * cos_v + beta * sin_v);
                coeffs.b1 = -2.0 * tmpgain * ((tmpgain - 1.0) + (tmpgain + 1.0) * cos_v);
                coeffs.b2 =
                    tmpgain * ((tmpgain + 1.0) + (tmpgain - 1.0) * cos_v - beta * sin_v);
                coeffs.a1 = 2.0 * ((tmpgain - 1.0) - (tmpgain + 1.0) * cos_v);
                coeffs.a2 = (tmpgain + 1.0) - (tmpgain - 1.0) * cos_v - beta * sin_v;
            }
        }

        coeffs.b0 /= a0;
        coeffs.b1 /= a0;
        coeffs.b2 /= a0;
        coeffs.a1 /= -a0;
        coeffs.a2 /= -a0;
    }

    /// `get_response`: the squared magnitude response at `freq`.
    pub fn get_response(&self, freq_in: f32, coeffs: &Coeffs) -> f32 {
        let freq = freq_in as f64 / self.sampling_rate as f64 * std::f64::consts::TAU;

        let mut cx = coeffs.b0;
        let mut cy = 0.0f64;
        cx += freq.cos() * coeffs.b1;
        cy -= freq.sin() * coeffs.b1;
        cx += (2.0 * freq).cos() * coeffs.b2;
        cy -= (2.0 * freq).sin() * coeffs.b2;

        let mut h = cx * cx + cy * cy;
        cx = 1.0;
        cy = 0.0;
        cx -= freq.cos() * coeffs.a1;
        cy += freq.sin() * coeffs.a1;
        cx -= (2.0 * freq).cos() * coeffs.a2;
        cy += (2.0 * freq).sin() * coeffs.a2;

        h /= cx * cx + cy * cy;
        h as f32
    }
}

#[derive(Clone, Debug, Default)]
pub struct FilterProcessor {
    filter: Option<AudioFilterSw>,
    coeffs: Coeffs,
    incr_coeffs: Coeffs,
    ha1: f32,
    ha2: f32,
    hb1: f32,
    hb2: f32,
}

impl FilterProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// `set_filter` (history is cleared by default).
    pub fn set_filter(&mut self, filter: Option<AudioFilterSw>, clear_history: bool) {
        if clear_history {
            self.ha1 = 0.0;
            self.ha2 = 0.0;
            self.hb1 = 0.0;
            self.hb2 = 0.0;
        }
        self.filter = filter;
    }

    pub fn get_filter(&self) -> Option<&AudioFilterSw> {
        self.filter.as_ref()
    }

    pub fn get_coeffs(&self) -> &Coeffs {
        &self.coeffs
    }

    /// `update_coeffs`: with `interp_buffer_len`, the coefficients ramp from the
    /// previous values across the buffer.
    pub fn update_coeffs(&mut self, interp_buffer_len: usize) {
        let Some(filter) = self.filter.as_ref() else {
            return;
        };
        if interp_buffer_len > 0 {
            let old = self.coeffs;
            filter.prepare_coefficients(&mut self.coeffs);
            let divisor = interp_buffer_len as f64;
            self.incr_coeffs.a1 = (self.coeffs.a1 - old.a1) / divisor;
            self.incr_coeffs.a2 = (self.coeffs.a2 - old.a2) / divisor;
            self.incr_coeffs.b0 = (self.coeffs.b0 - old.b0) / divisor;
            self.incr_coeffs.b1 = (self.coeffs.b1 - old.b1) / divisor;
            self.incr_coeffs.b2 = (self.coeffs.b2 - old.b2) / divisor;
            self.coeffs = old;
        } else {
            filter.prepare_coefficients(&mut self.coeffs);
        }
    }

    /// Public single-sample entry point (the engine's `process_one` is inline
    /// and used by the effect chain).
    #[inline]
    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.process_one(sample)
    }

    /// `process_one`.
    #[inline]
    fn process_one(&mut self, sample: f32) -> f32 {
        let pre = sample;
        let out = sample * self.coeffs.b0 as f32
            + self.hb1 * self.coeffs.b1 as f32
            + self.hb2 * self.coeffs.b2 as f32
            + self.ha1 * self.coeffs.a1 as f32
            + self.ha2 * self.coeffs.a2 as f32;
        self.ha2 = self.ha1;
        self.hb2 = self.hb1;
        self.hb1 = pre;
        self.ha1 = out;
        out
    }

    /// `process_one_interp`: also ramps the coefficients.
    #[inline]
    fn process_one_interp(&mut self, sample: f32) -> f32 {
        let out = self.process_one(sample);
        self.coeffs.b0 += self.incr_coeffs.b0;
        self.coeffs.b1 += self.incr_coeffs.b1;
        self.coeffs.b2 += self.incr_coeffs.b2;
        self.coeffs.a1 += self.incr_coeffs.a1;
        self.coeffs.a2 += self.incr_coeffs.a2;
        out
    }

    /// `process`: in-place over `samples` with a stride.
    pub fn process(&mut self, samples: &mut [f32], stride: usize, interpolate: bool) {
        if self.filter.is_none() || stride == 0 {
            return;
        }
        let mut index = 0usize;
        while index < samples.len() {
            samples[index] = if interpolate {
                self.process_one_interp(samples[index])
            } else {
                self.process_one(samples[index])
            };
            index += stride;
        }
    }

    pub fn clear_history(&mut self) {
        self.ha1 = 0.0;
        self.ha2 = 0.0;
        self.hb1 = 0.0;
        self.hb2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_filter(mode: FilterMode) -> AudioFilterSw {
        let mut filter = AudioFilterSw::new();
        filter.set_mode(mode);
        filter
    }

    #[test]
    fn defaults_and_accessors() {
        let mut filter = AudioFilterSw::new();
        assert_eq!(filter.get_cutoff(), 5000.0);
        assert_eq!(filter.get_resonance(), 0.5);
        assert_eq!(filter.get_gain(), 1.0);
        assert_eq!(filter.get_sampling_rate(), 44100.0);
        assert_eq!(filter.get_stages(), 1);
        assert_eq!(filter.get_mode(), FilterMode::Lowpass);
        filter.set_mode(FilterMode::Bandpass);
        filter.set_cutoff(1000.0);
        filter.set_resonance(2.0);
        filter.set_gain(3.0);
        filter.set_sampling_rate(48000.0);
        filter.set_stages(3);
        assert_eq!(filter.get_mode(), FilterMode::Bandpass);
        assert_eq!(filter.get_cutoff(), 1000.0);
        assert_eq!(filter.get_resonance(), 2.0);
        assert_eq!(filter.get_gain(), 3.0);
        assert_eq!(filter.get_sampling_rate(), 48000.0);
        assert_eq!(filter.get_stages(), 3);
    }

    #[test]
    fn lowpass_response() {
        // A low-pass has unit DC gain and a falling response at Nyquist.
        let filter = make_filter(FilterMode::Lowpass);
        let mut coeffs = Coeffs::default();
        filter.prepare_coefficients(&mut coeffs);
        let dc = filter.get_response(0.0, &coeffs);
        assert!((dc - 1.0).abs() < 0.01, "dc {dc}");
        let high = filter.get_response(20000.0, &coeffs);
        assert!(high < 0.01, "high {high}");

        // A high-pass is the inverse at DC.
        let filter = make_filter(FilterMode::Highpass);
        filter.prepare_coefficients(&mut coeffs);
        assert!(filter.get_response(0.0, &coeffs) < 1e-4);
        assert!(filter.get_response(20000.0, &coeffs) > 0.5);

        // The notch rejects at the cutoff.
        let filter = make_filter(FilterMode::Notch);
        filter.prepare_coefficients(&mut coeffs);
        let at_cutoff = filter.get_response(5000.0, &coeffs);
        assert!(at_cutoff < 0.01);
        assert!((filter.get_response(0.0, &coeffs) - 1.0).abs() < 0.01);
    }

    #[test]
    fn cutoff_is_clamped() {
        let mut filter = make_filter(FilterMode::Lowpass);
        filter.set_cutoff(1e9);
        let mut coeffs = Coeffs::default();
        filter.prepare_coefficients(&mut coeffs);
        // The clamped cutoff stays below Nyquist, so the response remains finite.
        assert!(coeffs.a1.is_finite() && coeffs.b0.is_finite());
        filter.set_cutoff(-100.0);
        filter.prepare_coefficients(&mut coeffs);
        assert!(coeffs.b0.is_finite());
        // Resonance and gain floors keep the coefficients finite.
        filter.set_resonance(0.0);
        filter.set_gain(0.0);
        filter.prepare_coefficients(&mut coeffs);
        assert!(coeffs.a1.is_finite() && coeffs.a2.is_finite());
    }

    #[test]
    fn processor_reaches_dc_gain() {
        let mut processor = FilterProcessor::new();
        assert!(processor.get_filter().is_none());
        processor.set_filter(Some(make_filter(FilterMode::Lowpass)), true);
        assert!(processor.get_filter().is_some());
        processor.update_coeffs(0);

        // Feed a constant; after the transient the output approaches the input
        // (unit DC gain).
        let mut samples = vec![1.0f32; 2048];
        processor.process(&mut samples, 1, false);
        assert!((samples[2047] - 1.0).abs() < 0.01);

        // Clearing history restarts the transient.
        processor.clear_history();
        let mut impulse = vec![0.0f32; 8];
        impulse[0] = 1.0;
        processor.process(&mut impulse, 1, false);
        assert!(impulse[0] != 1.0);
        assert!(impulse[1] != 0.0, "the filter rings after the impulse");
    }

    #[test]
    fn processor_interpolation_and_stride() {
        let mut filter = make_filter(FilterMode::Lowpass);
        filter.set_cutoff(1000.0);
        let mut processor = FilterProcessor::new();
        processor.set_filter(Some(filter), true);
        processor.update_coeffs(0);
        let start = *processor.get_coeffs();

        // Retune and interpolate across 64 samples.
        let mut retuned = make_filter(FilterMode::Lowpass);
        retuned.set_cutoff(8000.0);
        processor.set_filter(Some(retuned), false);
        processor.update_coeffs(64);
        let mut samples = vec![0.5f32; 128];
        processor.process(&mut samples, 1, true);
        let end = *processor.get_coeffs();
        assert!((end.b0 - start.b0).abs() > 1e-9);

        // Stride processing only touches every other sample.
        let mut processor = FilterProcessor::new();
        processor.set_filter(Some(make_filter(FilterMode::Lowpass)), true);
        processor.update_coeffs(0);
        let mut samples = vec![0.0f32, 1.0, 0.0, 1.0];
        processor.process(&mut samples, 2, false);
        assert_eq!(samples[1], 1.0, "odd samples untouched");

        // No filter means no work.
        let mut idle = FilterProcessor::new();
        let mut untouched = vec![1.0f32; 4];
        idle.process(&mut untouched, 1, false);
        assert_eq!(untouched, vec![1.0f32; 4]);
    }

    #[test]
    fn per_channel_processors_share_no_history() {
        // The engine uses one Processor per channel and a stride of 2 over the
        // interleaved buffer.
        let mut left = FilterProcessor::new();
        left.set_filter(Some(make_filter(FilterMode::Lowpass)), true);
        left.update_coeffs(0);
        let mut right = FilterProcessor::new();
        right.set_filter(Some(make_filter(FilterMode::Lowpass)), true);
        right.update_coeffs(0);

        let mut interleaved = Vec::new();
        for _ in 0..1024 {
            interleaved.push(1.0f32);
            interleaved.push(-1.0f32);
        }
        left.process(&mut interleaved, 2, false);
        right.process(&mut interleaved[1..], 2, false);
        assert!((interleaved[2046] - 1.0).abs() < 0.01);
        assert!((interleaved[2047] + 1.0).abs() < 0.01);
    }
}
