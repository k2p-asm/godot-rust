//! `scene/animation/animation_player.*`: playback timing.
//!
//! Ported: the animation library lookup, current animation, play/stop/seek,
//! `speed_scale`, and the per-step position update with the three loop modes
//! (`None` stops at the end, `Linear` wraps, `PingPong` reverses direction).
//!
//! Not ported: applying tracks to nodes, blending, `AnimationMixer` callbacks,
//! and the `Node` plumbing.

use crate::scene::animation::{Animation, LoopMode};
use crate::string::string_name::StringName;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlaybackResult {
    pub finished: bool,
}

pub struct AnimationPlayerData {
    animations: HashMap<StringName, Animation>,
    current: Option<StringName>,
    position: f64,
    speed_scale: f64,
    playing: bool,
    direction: f64,
}

impl Default for AnimationPlayerData {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationPlayerData {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            current: None,
            position: 0.0,
            speed_scale: 1.0,
            playing: false,
            direction: 1.0,
        }
    }

    pub fn add_animation(&mut self, name: &str, animation: Animation) {
        self.animations.insert(StringName::new(name), animation);
    }

    pub fn has_animation(&self, name: &str) -> bool {
        self.animations.contains_key(&StringName::new(name))
    }

    pub fn remove_animation(&mut self, name: &str) -> bool {
        self.animations.remove(&StringName::new(name)).is_some()
    }

    /// `play(name)`: switching animations restarts at 0; replaying the current
    /// one resumes from its position, like the engine.
    pub fn play(&mut self, name: &str) -> bool {
        let key = StringName::new(name);
        if !self.animations.contains_key(&key) {
            return false;
        }
        if self.current != Some(key) {
            self.current = Some(key);
            self.position = 0.0;
            self.direction = 1.0;
        }
        self.playing = true;
        true
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.position = 0.0;
        self.direction = 1.0;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn get_current_animation(&self) -> Option<StringName> {
        self.current
    }

    pub fn get_current_animation_position(&self) -> f64 {
        self.position
    }

    /// `seek`: clamps into `[0, length]` of the current animation.
    pub fn seek(&mut self, seconds: f64, update: bool) -> bool {
        let Some(name) = self.current else {
            return false;
        };
        let Some(anim) = self.animations.get(&name) else {
            return false;
        };
        let length = anim.get_length();
        self.position = seconds.clamp(0.0, length);
        let _ = update; // applying tracks is not ported
        true
    }

    /// `speed_scale`: negative plays backwards.
    pub fn set_speed_scale(&mut self, scale: f64) {
        self.speed_scale = scale;
    }

    pub fn get_speed_scale(&self) -> f64 {
        self.speed_scale
    }

    /// One process step. Reports `finished` when a non-looping animation
    /// reaches its end (or its start, playing backwards).
    pub fn advance(&mut self, delta: f64) -> PlaybackResult {
        let mut result = PlaybackResult::default();
        if !self.playing {
            return result;
        }
        let Some(name) = self.current else {
            return result;
        };
        let Some(anim) = self.animations.get(&name) else {
            return result;
        };
        let length = anim.get_length();
        if length <= 0.0 {
            self.playing = false;
            result.finished = true;
            return result;
        }

        self.position += delta * self.speed_scale * self.direction;

        match anim.get_loop_mode() {
            LoopMode::None => {
                if self.position >= length {
                    self.position = length;
                    self.playing = false;
                    result.finished = true;
                } else if self.position <= 0.0 {
                    self.position = 0.0;
                    self.playing = false;
                    result.finished = true;
                }
            }
            LoopMode::Linear => {
                self.position = self.position.rem_euclid(length);
            }
            LoopMode::PingPong => {
                // Bounce at both ends by reflecting the overshoot.
                while self.position > length || self.position < 0.0 {
                    if self.position > length {
                        self.position = 2.0 * length - self.position;
                        self.direction = -1.0;
                    } else {
                        self.position = -self.position;
                        self.direction = 1.0;
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anim(length: f64, loop_mode: LoopMode) -> Animation {
        let mut a = Animation::new();
        a.set_length(length);
        a.set_loop_mode(loop_mode);
        a
    }

    fn player(length: f64, loop_mode: LoopMode) -> AnimationPlayerData {
        let mut p = AnimationPlayerData::new();
        p.add_animation("run", anim(length, loop_mode));
        assert!(p.play("run"));
        p
    }

    #[test]
    fn play_requires_an_existing_animation() {
        let mut p = AnimationPlayerData::new();
        assert!(!p.play("missing"));
        assert!(!p.is_playing());
        p.add_animation("idle", anim(1.0, LoopMode::None));
        assert!(p.has_animation("idle"));
        assert!(p.play("idle"));
        assert!(p.is_playing());
        assert_eq!(p.get_current_animation().unwrap().as_string(), "idle");
    }

    #[test]
    fn position_advances_with_speed_scale() {
        let mut p = player(2.0, LoopMode::None);
        p.advance(0.5);
        assert!((p.get_current_animation_position() - 0.5).abs() < 1e-9);
        p.set_speed_scale(2.0);
        p.advance(0.25);
        assert!((p.get_current_animation_position() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn non_looping_animation_finishes_at_the_end() {
        let mut p = player(1.0, LoopMode::None);
        let r = p.advance(1.5);
        assert!(r.finished);
        assert!(!p.is_playing());
        assert!((p.get_current_animation_position() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn linear_loop_wraps() {
        let mut p = player(1.0, LoopMode::Linear);
        let r = p.advance(1.25);
        assert!(!r.finished);
        assert!(p.is_playing());
        assert!((p.get_current_animation_position() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn pingpong_reverses_at_the_ends() {
        let mut p = player(1.0, LoopMode::PingPong);
        p.advance(1.25); // 0.25 and now heading back
        assert!((p.get_current_animation_position() - 0.75).abs() < 1e-9);
        p.advance(0.5); // 0.25 heading back
        assert!((p.get_current_animation_position() - 0.25).abs() < 1e-9);
        p.advance(0.5); // hits 0, direction flips to forward
        assert!((p.get_current_animation_position() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn seek_clamps_and_switching_restarts() {
        let mut p = player(2.0, LoopMode::None);
        assert!(p.seek(5.0, true));
        assert!((p.get_current_animation_position() - 2.0).abs() < 1e-9);
        assert!(p.seek(-1.0, true));
        assert_eq!(p.get_current_animation_position(), 0.0);

        p.advance(1.0);
        p.add_animation("walk", anim(3.0, LoopMode::None));
        assert!(p.play("walk"));
        assert_eq!(p.get_current_animation_position(), 0.0, "new animation restarts");
    }

    #[test]
    fn stop_resets_position() {
        let mut p = player(2.0, LoopMode::None);
        p.advance(1.0);
        p.stop();
        assert!(!p.is_playing());
        assert_eq!(p.get_current_animation_position(), 0.0);
        // Advancing while stopped does nothing.
        let r = p.advance(1.0);
        assert!(!r.finished);
        assert_eq!(p.get_current_animation_position(), 0.0);
    }
}
