//! `scene/gui/video_stream_player.*`: the video player's state.
//!
//! Defaults from the header: no stream, paused off, autoplay off, volume 1.0,
//! speed scale 1.0, expand off, loop off, buffering 500 ms, audio track 0,
//! bus index 0 and bus `Master`.
//!
//! Playback, the texture hand-off, the audio resampler and the mix callbacks are
//! not ported; `play`/`stop` only move the paused flag like the node's state.

use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct VideoStreamPlayerData {
    stream: Option<Rid>,
    paused: bool,
    autoplay: bool,
    volume: f32,
    speed_scale: f32,
    expand: bool,
    looping: bool,
    buffering_ms: i32,
    audio_track: i32,
    bus_index: i32,
    bus: String,
}

impl Default for VideoStreamPlayerData {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoStreamPlayerData {
    pub fn new() -> Self {
        Self {
            stream: None,
            paused: false,
            autoplay: false,
            volume: 1.0,
            speed_scale: 1.0,
            expand: false,
            looping: false,
            buffering_ms: 500,
            audio_track: 0,
            bus_index: 0,
            bus: "Master".to_string(),
        }
    }

    pub fn set_stream(&mut self, stream: Option<Rid>) {
        self.stream = stream;
    }

    pub fn get_stream(&self) -> Option<Rid> {
        self.stream
    }

    /// `play`: playback starts (the engine instantiates a playback object).
    pub fn play(&mut self) {
        self.paused = false;
    }

    /// `stop`: playback ends; the engine clears the playback reference.
    pub fn stop(&mut self) {
        self.paused = false;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_autoplay(&mut self, enabled: bool) {
        self.autoplay = enabled;
    }

    pub fn has_autoplay(&self) -> bool {
        self.autoplay
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn set_speed_scale(&mut self, speed: f32) {
        self.speed_scale = speed;
    }

    pub fn get_speed_scale(&self) -> f32 {
        self.speed_scale
    }

    pub fn set_expand(&mut self, expand: bool) {
        self.expand = expand;
    }

    pub fn has_expand(&self) -> bool {
        self.expand
    }

    pub fn set_loop(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub fn has_loop(&self) -> bool {
        self.looping
    }

    pub fn set_buffering_msec(&mut self, msec: i32) {
        self.buffering_ms = msec;
    }

    pub fn get_buffering_msec(&self) -> i32 {
        self.buffering_ms
    }

    pub fn set_audio_track(&mut self, track: i32) {
        self.audio_track = track;
    }

    pub fn get_audio_track(&self) -> i32 {
        self.audio_track
    }

    pub fn set_bus(&mut self, bus: &str) {
        self.bus = bus.to_string();
    }

    pub fn get_bus(&self) -> &str {
        &self.bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut player = VideoStreamPlayerData::new();
        assert!(player.get_stream().is_none());
        assert!(!player.is_paused());
        assert!(!player.has_autoplay());
        assert_eq!(player.get_volume(), 1.0);
        assert_eq!(player.get_speed_scale(), 1.0);
        assert!(!player.has_expand());
        assert!(!player.has_loop());
        assert_eq!(player.get_buffering_msec(), 500);
        assert_eq!(player.get_audio_track(), 0);
        assert_eq!(player.get_bus(), "Master");

        player.set_stream(Some(Rid::from_u64(3)));
        player.set_volume(0.5);
        player.set_speed_scale(2.0);
        player.set_expand(true);
        player.set_loop(true);
        player.set_buffering_msec(250);
        player.set_audio_track(1);
        player.set_bus("Video");
        player.set_autoplay(true);
        assert_eq!(player.get_stream(), Some(Rid::from_u64(3)));
        assert_eq!(player.get_volume(), 0.5);
        assert_eq!(player.get_speed_scale(), 2.0);
        assert!(player.has_expand());
        assert!(player.has_loop());
        assert_eq!(player.get_buffering_msec(), 250);
        assert_eq!(player.get_audio_track(), 1);
        assert_eq!(player.get_bus(), "Video");
        assert!(player.has_autoplay());

        player.play();
        assert!(!player.is_paused());
        player.set_paused(true);
        assert!(player.is_paused());
        player.stop();
        assert!(!player.is_paused());
    }
}
