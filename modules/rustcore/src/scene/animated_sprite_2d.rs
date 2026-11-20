//! `scene/2d/animated_sprite_2d.*`: frame playback state.
//!
//! Defaults from the header: no frames, empty autoplay, not playing, the
//! `default` animation name, frame 0, speed scales 1, centred with no offset and
//! no flips.
//!
//! Ported exactly:
//!
//! - `set_frame_and_progress` clamps a negative frame to 0 and a frame past the
//!   animation's last to that last frame, then recomputes
//!   `frame_speed_scale = 1 / frame_duration`;
//! - `play` rejects a missing frames resource or animation, ignores an empty
//!   animation, sets the custom speed, and resets the frame according to the
//!   direction (from-end/backwards or from-start/forwards) with the engine's
//!   "already at the boundary" conditions;
//! - `pause` keeps the frame, `stop` resets the custom scale and the frame;
//! - `get_playing_speed` is 0 unless playing.
//!
//! Signals, processing and drawing are not ported.

use crate::math::vector2::Vector2;
use crate::scene::sprite_frames::SpriteFrames;
use crate::string::string_name::StringName;

#[derive(Clone, Debug)]
pub struct AnimatedSprite2DData {
    frames: Option<SpriteFrames>,
    autoplay: String,
    playing: bool,
    animation: StringName,
    frame: i32,
    speed_scale: f32,
    custom_speed_scale: f32,
    frame_speed_scale: f32,
    frame_progress: f32,
    centered: bool,
    offset: Vector2,
    hflip: bool,
    vflip: bool,
}

impl Default for AnimatedSprite2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimatedSprite2DData {
    pub fn new() -> Self {
        Self {
            frames: None,
            autoplay: String::new(),
            playing: false,
            animation: StringName::new("default"),
            frame: 0,
            speed_scale: 1.0,
            custom_speed_scale: 1.0,
            frame_speed_scale: 1.0,
            frame_progress: 0.0,
            centered: true,
            offset: Vector2::ZERO,
            hflip: false,
            vflip: false,
        }
    }

    /// Stores the frames and repairs the animation name like the engine: an
    /// empty resource clears everything, otherwise a missing animation falls
    /// back to the alphabetically first one. (The engine's identity early-out
    /// only suppresses signals, so it is omitted here.)
    pub fn set_sprite_frames(&mut self, frames: Option<SpriteFrames>) {
        self.frames = frames;
        if let Some(frames) = &self.frames {
            let names = frames.get_animation_names();
            if names.is_empty() {
                self.animation = StringName::new("");
                self.autoplay.clear();
            } else if !frames.has_animation(&self.animation.as_string()) {
                self.animation = names[0];
            }
        }
    }

    pub fn get_sprite_frames(&self) -> Option<&SpriteFrames> {
        self.frames.as_ref()
    }

    pub fn set_autoplay(&mut self, name: &str) {
        self.autoplay = name.to_string();
    }

    pub fn get_autoplay(&self) -> &str {
        &self.autoplay
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// 0 unless playing, otherwise `speed_scale * custom_speed_scale`.
    pub fn get_playing_speed(&self) -> f32 {
        if !self.playing {
            return 0.0;
        }
        self.speed_scale * self.custom_speed_scale
    }

    fn signbit(value: f32) -> bool {
        value.to_bits() & 0x8000_0000 != 0
    }

    fn end_frame(&self) -> i32 {
        match &self.frames {
            Some(frames) if frames.has_animation(&self.animation.as_string()) => {
                (frames.get_frame_count(&self.animation.as_string()) - 1).max(0)
            }
            _ => 0,
        }
    }

    fn frame_duration(&self) -> f64 {
        if let Some(frames) = &self.frames {
            if frames.has_animation(&self.animation.as_string()) {
                return frames.get_frame_duration(&self.animation.as_string(), self.frame.max(0) as usize)
                    as f64;
            }
        }
        1.0
    }

    fn calc_frame_speed_scale(&mut self) {
        self.frame_speed_scale = 1.0 / self.frame_duration() as f32;
    }

    pub fn get_frame_duration(&self) -> f64 {
        self.frame_duration()
    }

    pub fn get_frame_speed_scale(&self) -> f32 {
        self.frame_speed_scale
    }

    /// `set_frame_and_progress`; returns whether the frame changed.
    pub fn set_frame_and_progress(&mut self, frame: i32, progress: f32) -> bool {
        if self.frames.is_none() {
            return false;
        }
        let has_animation = self
            .frames
            .as_ref()
            .map(|f| f.has_animation(&self.animation.as_string()))
            .unwrap_or(false);
        let end_frame = self.end_frame();
        let changed = self.frame != frame;

        if frame < 0 {
            self.frame = 0;
        } else if has_animation && frame > end_frame {
            self.frame = end_frame;
        } else {
            self.frame = frame;
        }

        self.calc_frame_speed_scale();
        self.frame_progress = progress;
        changed
    }

    /// `set_frame`: progress starts at 1 when playing backwards.
    pub fn set_frame(&mut self, frame: i32) -> bool {
        let progress = if Self::signbit(self.get_playing_speed()) {
            1.0
        } else {
            0.0
        };
        self.set_frame_and_progress(frame, progress)
    }

    pub fn get_frame(&self) -> i32 {
        self.frame
    }

    pub fn set_frame_progress(&mut self, progress: f32) {
        self.frame_progress = progress;
    }

    pub fn get_frame_progress(&self) -> f32 {
        self.frame_progress
    }

    /// `play`; fails on a missing frames resource or animation, and returns
    /// false when the animation has no frames.
    pub fn play(&mut self, name: &str, custom_scale: f32, from_end: bool) -> bool {
        let requested = if name.is_empty() {
            self.animation.as_string()
        } else {
            name.to_string()
        };

        let Some(frames) = &self.frames else {
            return false;
        };
        if !frames.has_animation(&requested) {
            return false;
        }
        if frames.get_frame_count(&requested) == 0 {
            return false;
        }

        self.playing = true;
        self.custom_speed_scale = custom_scale;

        if requested != self.animation.as_string() {
            self.animation = StringName::new(&requested);
            let end_frame = self.end_frame();
            if from_end {
                self.set_frame_and_progress(end_frame, 1.0);
            } else {
                self.set_frame_and_progress(0, 0.0);
            }
        } else {
            let end_frame = self.end_frame();
            let is_backward = Self::signbit(self.speed_scale * self.custom_speed_scale);
            if from_end && is_backward && self.frame == 0 && self.frame_progress <= 0.0 {
                self.set_frame_and_progress(end_frame, 1.0);
            } else if !from_end && !is_backward && self.frame == end_frame && self.frame_progress >= 1.0
            {
                self.set_frame_and_progress(0, 0.0);
            }
        }
        true
    }

    pub fn play_backwards(&mut self, name: &str) -> bool {
        self.play(name, -1.0, true)
    }

    /// `pause`: keeps the current frame and custom speed.
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// `stop`: also resets the custom speed and the frame.
    pub fn stop(&mut self) {
        self.playing = false;
        self.custom_speed_scale = 1.0;
        self.set_frame_and_progress(0, 0.0);
    }

    /// `set_animation`: false when the name is missing (the engine keeps the
    /// string only until it validates).
    pub fn set_animation(&mut self, name: &str) -> bool {
        if self.animation.as_string() == name {
            return true;
        }
        let Some(frames) = &self.frames else {
            return false;
        };
        if !frames.has_animation(name) {
            return false;
        }
        self.animation = StringName::new(name);
        if frames.get_frame_count(name) == 0 {
            self.playing = false;
            return true;
        }
        if Self::signbit(self.get_playing_speed()) {
            let end = (frames.get_frame_count(name) - 1).max(0);
            self.set_frame_and_progress(end, 1.0);
        } else {
            self.set_frame_and_progress(0, 0.0);
        }
        true
    }

    pub fn get_animation(&self) -> StringName {
        self.animation
    }

    pub fn set_speed_scale(&mut self, scale: f32) {
        self.speed_scale = scale;
    }

    pub fn get_speed_scale(&self) -> f32 {
        self.speed_scale
    }

    pub fn set_centered(&mut self, centered: bool) {
        self.centered = centered;
    }

    pub fn is_centered(&self) -> bool {
        self.centered
    }

    pub fn set_offset(&mut self, offset: Vector2) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> Vector2 {
        self.offset
    }

    pub fn set_flip_h(&mut self, flip: bool) {
        self.hflip = flip;
    }

    pub fn is_flipped_h(&self) -> bool {
        self.hflip
    }

    pub fn set_flip_v(&mut self, flip: bool) {
        self.vflip = flip;
    }

    pub fn is_flipped_v(&self) -> bool {
        self.vflip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::sprite_frames::Frame;

    fn frames() -> SpriteFrames {
        // `SpriteFrames::new` creates `default` with one frame; add a second
        // animation with three frames and a 0.5 s middle frame.
        let mut f = SpriteFrames::new();
        f.add_animation("walk");
        for _ in 0..3 {
            f.add_frame("walk", Frame::new(None, 0.25), -1);
        }
        f.set_frame("walk", 1, Frame::new(None, 0.5));
        f
    }

    #[test]
    fn defaults() {
        let s = AnimatedSprite2DData::new();
        assert!(s.get_sprite_frames().is_none());
        assert_eq!(s.get_autoplay(), "");
        assert!(!s.is_playing());
        assert_eq!(s.get_animation().as_string(), "default");
        assert_eq!(s.get_frame(), 0);
        assert_eq!(s.get_speed_scale(), 1.0);
        assert_eq!(s.get_playing_speed(), 0.0);
        assert_eq!(s.get_frame_progress(), 0.0);
        assert!(s.is_centered());
        assert_eq!(s.get_offset(), Vector2::ZERO);
        assert!(!s.is_flipped_h());
        assert!(!s.is_flipped_v());
        // Without frames, a frame change is refused.
        let mut s = s;
        assert!(!s.set_frame(2));
    }

    #[test]
    fn assigning_frames_repairs_the_animation_name() {
        let mut s = AnimatedSprite2DData::new();
        s.set_sprite_frames(Some(frames()));
        // `default` exists, so it stays selected.
        assert_eq!(s.get_animation().as_string(), "default");

        let mut empty = SpriteFrames::new();
        empty.remove_animation("default");
        let mut e = AnimatedSprite2DData::new();
        e.set_autoplay("run");
        e.set_sprite_frames(Some(empty));
        assert_eq!(e.get_animation().as_string(), "");
        assert_eq!(e.get_autoplay(), "", "autoplay is cleared with no animations");

        // A frames resource without the current animation picks the first
        // alphabetical one.
        let mut only = SpriteFrames::new();
        only.remove_animation("default");
        only.add_animation("walk");
        only.add_animation("idle");
        let mut o = AnimatedSprite2DData::new();
        o.set_sprite_frames(Some(only));
        assert_eq!(o.get_animation().as_string(), "idle");
    }

    #[test]
    fn frame_clamping_and_duration() {
        let mut s = AnimatedSprite2DData::new();
        s.set_sprite_frames(Some(frames()));
        s.set_animation("walk");

        assert!(s.set_frame(1));
        assert_eq!(s.get_frame(), 1);
        // Frame 1 lasts 0.5 s, so the speed scale is 2.
        assert!((s.get_frame_speed_scale() - 2.0).abs() < 1e-5);
        assert!((s.get_frame_duration() - 0.5).abs() < 1e-6);

        // Out of range clamps to the last frame (2).
        s.set_frame(99);
        assert_eq!(s.get_frame(), 2);
        // Negative clamps to 0.
        s.set_frame(-5);
        assert_eq!(s.get_frame(), 0);

        s.set_frame_progress(0.25);
        assert_eq!(s.get_frame_progress(), 0.25);
        // Setting the same frame reports no change.
        assert!(!s.set_frame_and_progress(0, 0.5));
        assert!(s.set_frame_and_progress(1, 0.5));
    }

    #[test]
    fn play_resets_the_frame_for_the_direction() {
        let mut s = AnimatedSprite2DData::new();
        s.set_sprite_frames(Some(frames()));

        assert!(s.play("walk", 1.0, false));
        assert!(s.is_playing());
        assert_eq!(s.get_animation().as_string(), "walk");
        assert_eq!(s.get_frame(), 0);
        assert_eq!(s.get_playing_speed(), 1.0);

        // `from_end` only applies when the animation changes (or when already
        // playing backwards); switching away and back shows it.
        s.set_animation("default");
        assert!(s.play("walk", 1.0, true));
        assert_eq!(s.get_frame(), 2);
        assert_eq!(s.get_frame_progress(), 1.0);

        // Same animation, forwards, from_end: the engine leaves the frame alone.
        s.set_frame_and_progress(0, 0.0);
        assert!(s.play("walk", 1.0, true));
        assert_eq!(s.get_frame(), 0);

        // Backwards playback reports a negative speed.
        assert!(s.play_backwards("walk"));
        assert_eq!(s.get_playing_speed(), -1.0);

        // Missing animation or empty frames are refused.
        assert!(!s.play("missing", 1.0, false));
        let mut null = AnimatedSprite2DData::new();
        assert!(!null.play("", 1.0, false));
    }

    #[test]
    fn pause_and_stop() {
        let mut s = AnimatedSprite2DData::new();
        s.set_sprite_frames(Some(frames()));
        s.play("walk", 2.0, false);
        s.set_frame_and_progress(1, 0.5);

        s.pause();
        assert!(!s.is_playing());
        assert_eq!(s.get_frame(), 1, "pause keeps the frame");
        assert_eq!(s.get_playing_speed(), 0.0);

        s.play("walk", 2.0, false);
        s.stop();
        assert!(!s.is_playing());
        assert_eq!(s.get_frame(), 0, "stop resets the frame");
        assert_eq!(s.get_frame_progress(), 0.0);

        // After stop the custom speed is back to 1.
        assert!(s.play("walk", 1.0, false));
        assert_eq!(s.get_playing_speed(), 1.0);
    }

    #[test]
    fn animation_validation_and_flags() {
        let mut s = AnimatedSprite2DData::new();
        s.set_sprite_frames(Some(frames()));

        assert!(!s.set_animation("missing"));
        assert_eq!(s.get_animation().as_string(), "default");
        assert!(s.set_animation("walk"));
        assert_eq!(s.get_animation().as_string(), "walk");
        // Setting the current animation again is accepted.
        assert!(s.set_animation("walk"));

        s.set_speed_scale(2.0);
        s.set_centered(false);
        s.set_offset(Vector2::new(3.0, -4.0));
        s.set_flip_h(true);
        s.set_flip_v(true);
        assert_eq!(s.get_speed_scale(), 2.0);
        assert!(!s.is_centered());
        assert_eq!(s.get_offset(), Vector2::new(3.0, -4.0));
        assert!(s.is_flipped_h());
        assert!(s.is_flipped_v());
    }
}
