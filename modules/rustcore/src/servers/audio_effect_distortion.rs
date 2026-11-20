//! `servers/audio/effects/audio_effect_distortion.*`: parameter state.
//!
//! Mode enum in declaration order and the constructor defaults from
//! `AudioEffectDistortion::AudioEffectDistortion()`. Every setter assigns
//! directly. The waveshaping DSP is not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DistortionMode {
    Clip,
    Atan,
    Lofi,
    Overdrive,
    Waveshape,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioEffectDistortion {
    mode: DistortionMode,
    pre_gain: f32,
    post_gain: f32,
    keep_hf_hz: f32,
    drive: f32,
}

impl Default for AudioEffectDistortion {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEffectDistortion {
    pub fn new() -> Self {
        Self {
            mode: DistortionMode::Clip,
            pre_gain: 0.0,
            post_gain: 0.0,
            keep_hf_hz: 16000.0,
            drive: 0.0,
        }
    }

    pub fn set_mode(&mut self, mode: DistortionMode) {
        self.mode = mode;
    }

    pub fn get_mode(&self) -> DistortionMode {
        self.mode
    }

    pub fn set_pre_gain(&mut self, pre_gain: f32) {
        self.pre_gain = pre_gain;
    }

    pub fn get_pre_gain(&self) -> f32 {
        self.pre_gain
    }

    pub fn set_post_gain(&mut self, post_gain: f32) {
        self.post_gain = post_gain;
    }

    pub fn get_post_gain(&self) -> f32 {
        self.post_gain
    }

    pub fn set_keep_hf_hz(&mut self, keep_hf_hz: f32) {
        self.keep_hf_hz = keep_hf_hz;
    }

    pub fn get_keep_hf_hz(&self) -> f32 {
        self.keep_hf_hz
    }

    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive;
    }

    pub fn get_drive(&self) -> f32 {
        self.drive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_constructor() {
        let d = AudioEffectDistortion::new();
        assert_eq!(d.get_mode(), DistortionMode::Clip);
        assert_eq!(d.get_pre_gain(), 0.0);
        assert_eq!(d.get_post_gain(), 0.0);
        assert_eq!(d.get_keep_hf_hz(), 16000.0);
        assert_eq!(d.get_drive(), 0.0);
    }

    #[test]
    fn mode_values_follow_declaration_order() {
        assert_eq!(DistortionMode::Clip as i32, 0);
        assert_eq!(DistortionMode::Atan as i32, 1);
        assert_eq!(DistortionMode::Lofi as i32, 2);
        assert_eq!(DistortionMode::Overdrive as i32, 3);
        assert_eq!(DistortionMode::Waveshape as i32, 4);
    }

    #[test]
    fn setters_assign_without_clamping() {
        let mut d = AudioEffectDistortion::new();
        d.set_mode(DistortionMode::Overdrive);
        d.set_pre_gain(24.0);
        d.set_post_gain(-12.0);
        d.set_keep_hf_hz(1.0);
        d.set_drive(5.0);

        assert_eq!(d.get_mode(), DistortionMode::Overdrive);
        assert_eq!(d.get_pre_gain(), 24.0);
        assert_eq!(d.get_post_gain(), -12.0);
        assert_eq!(d.get_keep_hf_hz(), 1.0);
        assert_eq!(d.get_drive(), 5.0);
    }
}
