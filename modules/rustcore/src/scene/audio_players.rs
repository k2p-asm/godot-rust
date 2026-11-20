//! `scene/2d/audio_listener_2d.*`, `scene/2d/audio_stream_player_2d.*` and
//! `scene/3d/audio_stream_player_3d.*` (state only).
//!
//! `AudioListener2D` stores its `current` flag with the engine's
//! `make_current`/`clear_current`/`is_current` API.
//!
//! Both stream players share the defaults `stream = null`, volume 0 dB, pitch
//! scale 1, not playing, no autoplay, not paused, bus `Master`, max polyphony 1
//! and panning strength 1. The 2D player adds max distance 2000 and
//! attenuation 1; the 3D player adds unit size 10, max dB 3, zero max distance,
//! `ATTENUATION_INVERSE_DISTANCE`, Doppler tracking disabled, the emission cone
//! (disabled, 45 degrees, -12 dB), the attenuation filter (5000 Hz, -24 dB),
//! zero linear attenuation and the default playback type.
//!
//! AudioServer/stream playback is not ported; `play`/`stop` only update the
//! playing flag.

use crate::math::color::Color;
use crate::servers::rid::Rid;

// -- enums ------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PlaybackType {
    Default,
    Sample,
    Stream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AttenuationModel {
    InverseDistance,
    InverseSquareDistance,
    Logarithmic,
    Disabled,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DopplerTracking {
    Disabled,
    IdleStep,
    PhysicsStep,
}

// -- AudioListener2D --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AudioListener2DData {
    current: bool,
}

impl Default for AudioListener2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioListener2DData {
    pub fn new() -> Self {
        Self { current: false }
    }

    pub fn make_current(&mut self) {
        self.current = true;
    }

    pub fn clear_current(&mut self) {
        self.current = false;
    }

    pub fn is_current(&self) -> bool {
        self.current
    }
}

// -- AudioStreamPlayer2D ----------------------------------------------------

#[derive(Clone, Debug)]
pub struct AudioStreamPlayer2DData {
    stream: Option<Rid>,
    volume_db: f32,
    pitch_scale: f32,
    playing: bool,
    autoplay: bool,
    stream_paused: bool,
    bus: String,
    max_polyphony: i32,
    max_distance: f32,
    attenuation: f32,
    area_mask: u32,
    panning_strength: f32,
    playback_type: PlaybackType,
    force_update_panning: bool,
}

impl Default for AudioStreamPlayer2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamPlayer2DData {
    pub fn new() -> Self {
        Self {
            stream: None,
            volume_db: 0.0,
            pitch_scale: 1.0,
            playing: false,
            autoplay: false,
            stream_paused: false,
            bus: "Master".to_string(),
            max_polyphony: 1,
            max_distance: 2000.0,
            attenuation: 1.0,
            area_mask: 1,
            panning_strength: 1.0,
            playback_type: PlaybackType::Default,
            force_update_panning: false,
        }
    }

    pub fn set_stream(&mut self, stream: Option<Rid>) {
        self.stream = stream;
    }

    pub fn get_stream(&self) -> Option<Rid> {
        self.stream
    }

    pub fn set_volume_db(&mut self, volume: f32) {
        self.volume_db = volume;
    }

    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }

    /// `set_volume_linear`: the engine converts to dB with
    /// `linear_to_db`, and treats non-positive volumes as silence.
    pub fn set_volume_linear(&mut self, volume: f32) {
        self.volume_db = crate::math::funcs::linear_to_db(volume);
    }

    pub fn get_volume_linear(&self) -> f32 {
        crate::math::funcs::db_to_linear(self.volume_db)
    }

    pub fn set_pitch_scale(&mut self, pitch: f32) {
        self.pitch_scale = pitch;
    }

    pub fn get_pitch_scale(&self) -> f32 {
        self.pitch_scale
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn set_bus(&mut self, bus: &str) {
        self.bus = bus.to_string();
    }

    pub fn get_bus(&self) -> &str {
        &self.bus
    }

    pub fn set_autoplay(&mut self, enabled: bool) {
        self.autoplay = enabled;
    }

    pub fn is_autoplay_enabled(&self) -> bool {
        self.autoplay
    }

    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance;
    }

    pub fn get_max_distance(&self) -> f32 {
        self.max_distance
    }

    pub fn set_attenuation(&mut self, attenuation: f32) {
        self.attenuation = attenuation;
    }

    pub fn get_attenuation(&self) -> f32 {
        self.attenuation
    }

    pub fn set_area_mask(&mut self, mask: u32) {
        self.area_mask = mask;
    }

    pub fn get_area_mask(&self) -> u32 {
        self.area_mask
    }

    pub fn set_stream_paused(&mut self, paused: bool) {
        self.stream_paused = paused;
    }

    pub fn get_stream_paused(&self) -> bool {
        self.stream_paused
    }

    pub fn set_max_polyphony(&mut self, polyphony: i32) {
        self.max_polyphony = polyphony;
    }

    pub fn get_max_polyphony(&self) -> i32 {
        self.max_polyphony
    }

    pub fn set_panning_strength(&mut self, strength: f32) {
        self.panning_strength = strength;
    }

    pub fn get_panning_strength(&self) -> f32 {
        self.panning_strength
    }

    /// `ERR_FAIL_INDEX(mode, PLAYBACK_TYPE_MAX)`; only the three named modes pass.
    pub fn set_playback_type(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => PlaybackType::Default,
            1 => PlaybackType::Sample,
            2 => PlaybackType::Stream,
            _ => return false,
        };
        self.playback_type = parsed;
        true
    }

    pub fn get_playback_type(&self) -> PlaybackType {
        self.playback_type
    }

    pub fn set_force_update_panning(&mut self, enabled: bool) {
        self.force_update_panning = enabled;
    }

    pub fn is_force_update_panning(&self) -> bool {
        self.force_update_panning
    }
}

// -- AudioStreamPlayer3D ----------------------------------------------------

#[derive(Clone, Debug)]
pub struct AudioStreamPlayer3DData {
    stream: Option<Rid>,
    volume_db: f32,
    unit_size: f32,
    max_db: f32,
    pitch_scale: f32,
    playing: bool,
    autoplay: bool,
    stream_paused: bool,
    bus: String,
    max_polyphony: i32,
    max_distance: f32,
    attenuation_model: AttenuationModel,
    doppler_tracking: DopplerTracking,
    emission_angle_enabled: bool,
    emission_angle: f32,
    emission_angle_filter_attenuation_db: f32,
    attenuation_filter_cutoff_hz: f32,
    attenuation_filter_db: f32,
    linear_attenuation: f32,
    area_mask: u32,
    panning_strength: f32,
    playback_type: PlaybackType,
    force_update_panning: bool,
    debug_color: Color,
}

impl Default for AudioStreamPlayer3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStreamPlayer3DData {
    pub fn new() -> Self {
        Self {
            stream: None,
            volume_db: 0.0,
            unit_size: 10.0,
            max_db: 3.0,
            pitch_scale: 1.0,
            playing: false,
            autoplay: false,
            stream_paused: false,
            bus: "Master".to_string(),
            max_polyphony: 1,
            max_distance: 0.0,
            attenuation_model: AttenuationModel::InverseDistance,
            doppler_tracking: DopplerTracking::Disabled,
            emission_angle_enabled: false,
            emission_angle: 45.0,
            emission_angle_filter_attenuation_db: -12.0,
            attenuation_filter_cutoff_hz: 5000.0,
            attenuation_filter_db: -24.0,
            linear_attenuation: 0.0,
            area_mask: 1,
            panning_strength: 1.0,
            playback_type: PlaybackType::Default,
            force_update_panning: false,
            debug_color: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn set_stream(&mut self, stream: Option<Rid>) {
        self.stream = stream;
    }

    pub fn get_stream(&self) -> Option<Rid> {
        self.stream
    }

    pub fn set_volume_db(&mut self, volume: f32) {
        self.volume_db = volume;
    }

    pub fn get_volume_db(&self) -> f32 {
        self.volume_db
    }

    pub fn set_volume_linear(&mut self, volume: f32) {
        self.volume_db = crate::math::funcs::linear_to_db(volume);
    }

    pub fn set_unit_size(&mut self, size: f32) {
        self.unit_size = size;
    }

    pub fn get_unit_size(&self) -> f32 {
        self.unit_size
    }

    pub fn set_max_db(&mut self, max_db: f32) {
        self.max_db = max_db;
    }

    pub fn get_max_db(&self) -> f32 {
        self.max_db
    }

    pub fn set_pitch_scale(&mut self, pitch: f32) {
        self.pitch_scale = pitch;
    }

    pub fn get_pitch_scale(&self) -> f32 {
        self.pitch_scale
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn set_bus(&mut self, bus: &str) {
        self.bus = bus.to_string();
    }

    pub fn get_bus(&self) -> &str {
        &self.bus
    }

    pub fn set_max_polyphony(&mut self, polyphony: i32) {
        self.max_polyphony = polyphony;
    }

    pub fn get_max_polyphony(&self) -> i32 {
        self.max_polyphony
    }

    pub fn set_autoplay(&mut self, enabled: bool) {
        self.autoplay = enabled;
    }

    pub fn is_autoplay_enabled(&self) -> bool {
        self.autoplay
    }

    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance;
    }

    pub fn get_max_distance(&self) -> f32 {
        self.max_distance
    }

    pub fn set_area_mask(&mut self, mask: u32) {
        self.area_mask = mask;
    }

    pub fn get_area_mask(&self) -> u32 {
        self.area_mask
    }

    pub fn set_emission_angle_enabled(&mut self, enabled: bool) {
        self.emission_angle_enabled = enabled;
    }

    pub fn is_emission_angle_enabled(&self) -> bool {
        self.emission_angle_enabled
    }

    pub fn set_emission_angle(&mut self, angle: f32) {
        self.emission_angle = angle;
    }

    pub fn get_emission_angle(&self) -> f32 {
        self.emission_angle
    }

    pub fn set_emission_angle_filter_attenuation_db(&mut self, attenuation: f32) {
        self.emission_angle_filter_attenuation_db = attenuation;
    }

    pub fn get_emission_angle_filter_attenuation_db(&self) -> f32 {
        self.emission_angle_filter_attenuation_db
    }

    pub fn set_attenuation_filter_cutoff_hz(&mut self, hz: f32) {
        self.attenuation_filter_cutoff_hz = hz;
    }

    pub fn get_attenuation_filter_cutoff_hz(&self) -> f32 {
        self.attenuation_filter_cutoff_hz
    }

    pub fn set_attenuation_filter_db(&mut self, db: f32) {
        self.attenuation_filter_db = db;
    }

    pub fn get_attenuation_filter_db(&self) -> f32 {
        self.attenuation_filter_db
    }

    pub fn set_linear_attenuation(&mut self, attenuation: f32) {
        self.linear_attenuation = attenuation;
    }

    pub fn get_linear_attenuation(&self) -> f32 {
        self.linear_attenuation
    }

    /// `ERR_FAIL_INDEX(model, ATTENUATION_MAX)`.
    pub fn set_attenuation_model(&mut self, model: i32) -> bool {
        let parsed = match model {
            0 => AttenuationModel::InverseDistance,
            1 => AttenuationModel::InverseSquareDistance,
            2 => AttenuationModel::Logarithmic,
            3 => AttenuationModel::Disabled,
            _ => return false,
        };
        self.attenuation_model = parsed;
        true
    }

    pub fn get_attenuation_model(&self) -> AttenuationModel {
        self.attenuation_model
    }

    /// `ERR_FAIL_INDEX(tracking, 3)`.
    pub fn set_doppler_tracking(&mut self, tracking: i32) -> bool {
        let parsed = match tracking {
            0 => DopplerTracking::Disabled,
            1 => DopplerTracking::IdleStep,
            2 => DopplerTracking::PhysicsStep,
            _ => return false,
        };
        self.doppler_tracking = parsed;
        true
    }

    pub fn get_doppler_tracking(&self) -> DopplerTracking {
        self.doppler_tracking
    }

    pub fn set_stream_paused(&mut self, paused: bool) {
        self.stream_paused = paused;
    }

    pub fn get_stream_paused(&self) -> bool {
        self.stream_paused
    }

    pub fn set_panning_strength(&mut self, strength: f32) {
        self.panning_strength = strength;
    }

    pub fn get_panning_strength(&self) -> f32 {
        self.panning_strength
    }

    pub fn set_playback_type(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => PlaybackType::Default,
            1 => PlaybackType::Sample,
            2 => PlaybackType::Stream,
            _ => return false,
        };
        self.playback_type = parsed;
        true
    }

    pub fn get_playback_type(&self) -> PlaybackType {
        self.playback_type
    }

    pub fn set_force_update_panning(&mut self, enabled: bool) {
        self.force_update_panning = enabled;
    }

    pub fn is_force_update_panning(&self) -> bool {
        self.force_update_panning
    }

    pub fn set_debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }

    pub fn get_debug_color(&self) -> Color {
        self.debug_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_2d_current() {
        let mut listener = AudioListener2DData::new();
        assert!(!listener.is_current());
        listener.make_current();
        assert!(listener.is_current());
        listener.clear_current();
        assert!(!listener.is_current());
    }

    #[test]
    fn player_2d_defaults_and_setters() {
        let mut player = AudioStreamPlayer2DData::new();
        assert!(player.get_stream().is_none());
        assert_eq!(player.get_volume_db(), 0.0);
        assert_eq!(player.get_pitch_scale(), 1.0);
        assert!(!player.is_playing());
        assert!(!player.is_autoplay_enabled());
        assert!(!player.get_stream_paused());
        assert_eq!(player.get_bus(), "Master");
        assert_eq!(player.get_max_polyphony(), 1);
        assert_eq!(player.get_max_distance(), 2000.0);
        assert_eq!(player.get_attenuation(), 1.0);
        assert_eq!(player.get_area_mask(), 1);
        assert_eq!(player.get_panning_strength(), 1.0);
        assert_eq!(player.get_playback_type(), PlaybackType::Default);
        assert!(!player.is_force_update_panning());

        player.set_stream(Some(Rid::from_u64(3)));
        player.set_volume_db(-6.0);
        player.set_pitch_scale(1.5);
        player.play();
        player.set_autoplay(true);
        player.set_stream_paused(true);
        player.set_bus("SFX");
        player.set_max_polyphony(4);
        player.set_max_distance(100.0);
        player.set_attenuation(0.5);
        player.set_area_mask(7);
        player.set_panning_strength(0.25);
        assert!(player.set_playback_type(2));
        assert!(!player.set_playback_type(3));
        player.set_force_update_panning(true);

        assert_eq!(player.get_stream(), Some(Rid::from_u64(3)));
        assert_eq!(player.get_volume_db(), -6.0);
        assert_eq!(player.get_pitch_scale(), 1.5);
        assert!(player.is_playing());
        assert!(player.is_autoplay_enabled());
        assert!(player.get_stream_paused());
        assert_eq!(player.get_bus(), "SFX");
        assert_eq!(player.get_max_polyphony(), 4);
        assert_eq!(player.get_max_distance(), 100.0);
        assert_eq!(player.get_attenuation(), 0.5);
        assert_eq!(player.get_area_mask(), 7);
        assert_eq!(player.get_panning_strength(), 0.25);
        assert_eq!(player.get_playback_type(), PlaybackType::Stream);
        assert!(player.is_force_update_panning());

        player.stop();
        assert!(!player.is_playing());

        // Linear volume round-trips through dB.
        player.set_volume_linear(0.5);
        assert!((player.get_volume_linear() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn player_3d_defaults() {
        let mut player = AudioStreamPlayer3DData::new();
        assert!(player.get_stream().is_none());
        assert_eq!(player.get_volume_db(), 0.0);
        assert_eq!(player.get_unit_size(), 10.0);
        assert_eq!(player.get_max_db(), 3.0);
        assert_eq!(player.get_pitch_scale(), 1.0);
        assert!(!player.is_playing());
        assert!(!player.is_autoplay_enabled());
        assert_eq!(player.get_bus(), "Master");
        assert_eq!(player.get_max_polyphony(), 1);
        assert_eq!(player.get_max_distance(), 0.0);
        assert_eq!(player.get_attenuation_model(), AttenuationModel::InverseDistance);
        assert_eq!(player.get_doppler_tracking(), DopplerTracking::Disabled);
        assert!(!player.is_emission_angle_enabled());
        assert_eq!(player.get_emission_angle(), 45.0);
        assert_eq!(player.get_emission_angle_filter_attenuation_db(), -12.0);
        assert_eq!(player.get_attenuation_filter_cutoff_hz(), 5000.0);
        assert_eq!(player.get_attenuation_filter_db(), -24.0);
        assert_eq!(player.get_linear_attenuation(), 0.0);
        assert_eq!(player.get_area_mask(), 1);
        assert_eq!(player.get_panning_strength(), 1.0);
        assert_eq!(player.get_playback_type(), PlaybackType::Default);
        assert_eq!(player.get_debug_color(), Color::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn player_3d_setters_and_guards() {
        let mut player = AudioStreamPlayer3DData::new();
        player.set_unit_size(4.0);
        player.set_max_db(6.0);
        player.set_max_distance(30.0);
        assert!(player.set_attenuation_model(3));
        assert!(!player.set_attenuation_model(4));
        assert!(player.set_doppler_tracking(2));
        assert!(!player.set_doppler_tracking(3));
        player.set_emission_angle_enabled(true);
        player.set_emission_angle(30.0);
        player.set_emission_angle_filter_attenuation_db(-6.0);
        player.set_attenuation_filter_cutoff_hz(900.0);
        player.set_attenuation_filter_db(-10.0);
        player.set_linear_attenuation(0.5);
        player.set_area_mask(3);
        player.set_debug_color(Color::new(1.0, 0.0, 0.0, 1.0));

        assert_eq!(player.get_unit_size(), 4.0);
        assert_eq!(player.get_max_db(), 6.0);
        assert_eq!(player.get_max_distance(), 30.0);
        assert_eq!(player.get_attenuation_model(), AttenuationModel::Disabled);
        assert_eq!(player.get_doppler_tracking(), DopplerTracking::PhysicsStep);
        assert!(player.is_emission_angle_enabled());
        assert_eq!(player.get_emission_angle(), 30.0);
        assert_eq!(player.get_emission_angle_filter_attenuation_db(), -6.0);
        assert_eq!(player.get_attenuation_filter_cutoff_hz(), 900.0);
        assert_eq!(player.get_attenuation_filter_db(), -10.0);
        assert_eq!(player.get_linear_attenuation(), 0.5);
        assert_eq!(player.get_area_mask(), 3);
        assert_eq!(player.get_debug_color(), Color::new(1.0, 0.0, 0.0, 1.0));
    }
}
