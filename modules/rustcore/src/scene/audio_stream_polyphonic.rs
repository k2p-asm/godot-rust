//! `scene/resources/audio_stream_polyphonic.*`: the stream descriptor.
//!
//! `AudioStreamPolyphonic` only exposes the voice count (default 32, validated
//! into `0..=128`) and the audio-server playback type (default). The playback
//! object, internal buffers and `play_stream` are not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PlaybackType {
    Default,
    Stream,
    Sample,
    Max,
}

#[derive(Clone, Copy, Debug)]
pub struct AudioStreamPolyphonicData {
    polyphony: i32,
    playback_type: PlaybackType,
}

impl Default for AudioStreamPolyphonicData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamPolyphonicData {
    pub fn new() -> Self {
        Self {
            polyphony: 32,
            playback_type: PlaybackType::Default,
        }
    }

    /// `ERR_FAIL_COND(p_voices < 0 || p_voices > 128)`.
    pub fn set_polyphony(&mut self, voices: i32) -> bool {
        if !(0..=128).contains(&voices) {
            return false;
        }
        self.polyphony = voices;
        true
    }

    pub fn get_polyphony(&self) -> i32 {
        self.polyphony
    }

    pub fn get_playback_type(&self) -> PlaybackType {
        self.playback_type
    }

    /// Meta stream: it plays other streams, so it is not monophonic itself.
    pub fn is_monophonic(&self) -> bool {
        false
    }

    pub fn is_meta_stream(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = AudioStreamPolyphonicData::new();
        assert_eq!(p.get_polyphony(), 32);
        assert_eq!(p.get_playback_type(), PlaybackType::Default);
        assert!(!p.is_monophonic());
        assert!(p.is_meta_stream());
    }

    #[test]
    fn polyphony_range_and_enum_values() {
        let mut p = AudioStreamPolyphonicData::new();
        assert!(!p.set_polyphony(-1));
        assert!(!p.set_polyphony(129));
        assert_eq!(p.get_polyphony(), 32);

        // Both ends are accepted.
        assert!(p.set_polyphony(0));
        assert_eq!(p.get_polyphony(), 0);
        assert!(p.set_polyphony(128));
        assert_eq!(p.get_polyphony(), 128);

        assert_eq!(PlaybackType::Default as i32, 0);
        assert_eq!(PlaybackType::Stream as i32, 1);
        assert_eq!(PlaybackType::Sample as i32, 2);
        assert_eq!(PlaybackType::Max as i32, 3);
    }
}
