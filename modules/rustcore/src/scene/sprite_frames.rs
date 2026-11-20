//! `scene/resources/sprite_frames.*` and `scene/2d/animated_sprite_2d.*`.
//!
//! `SpriteFrames` holds named animations, each with a speed, a loop mode and
//! frames carrying a duration. Textures are reduced to their size here, since
//! the animation logic only needs durations and the frame count.
//!
//! `AnimatedSprite2DData` stores the selected animation/frame and advances the
//! accumulator with the engine's rules: duration is divided by `speed_scale`,
//! and `LOOP_NONE` stops on the last frame, `LOOP_LINEAR` wraps and
//! `LOOP_PINGPONG` reverses.
//!
//! Not ported: drawing and the `Texture2D`/`Resource` plumbing.

use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::string::string_name::StringName;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopMode {
    None,
    Linear,
    PingPong,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub texture_size: Option<Vector2i>,
    pub duration: f32,
}

impl Frame {
    pub fn new(texture_size: Option<Vector2i>, duration: f32) -> Self {
        Self {
            texture_size,
            duration,
        }
    }
}

#[derive(Clone, Debug)]
struct Anim {
    speed: f64,
    loop_mode: LoopMode,
    frames: Vec<Frame>,
}

#[derive(Clone, Debug)]
pub struct SpriteFrames {
    animations: HashMap<StringName, Anim>,
}

impl Default for SpriteFrames {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteFrames {
    /// The engine creates a `default` animation with one frame and speed 5.
    pub fn new() -> Self {
        let mut animations = HashMap::new();
        animations.insert(
            StringName::new("default"),
            Anim {
                speed: 5.0,
                loop_mode: LoopMode::Linear,
                frames: vec![Frame::new(None, 1.0)],
            },
        );
        Self { animations }
    }

    pub fn has_animation(&self, name: &str) -> bool {
        self.animations.contains_key(&StringName::new(name))
    }

    pub fn add_animation(&mut self, name: &str) {
        self.animations.entry(StringName::new(name)).or_insert(Anim {
            speed: 5.0,
            loop_mode: LoopMode::Linear,
            frames: Vec::new(),
        });
    }

    pub fn remove_animation(&mut self, name: &str) -> bool {
        self.animations.remove(&StringName::new(name)).is_some()
    }

    pub fn rename_animation(&mut self, from: &str, to: &str) -> bool {
        let Some(anim) = self.animations.remove(&StringName::new(from)) else {
            return false;
        };
        self.animations.insert(StringName::new(to), anim);
        true
    }

    /// Sorted, for deterministic output.
    pub fn get_animation_names(&self) -> Vec<StringName> {
        let mut names: Vec<StringName> = self.animations.keys().copied().collect();
        names.sort_by_key(|n| n.as_string());
        names
    }

    pub fn set_animation_speed(&mut self, name: &str, speed: f64) -> bool {
        match self.animations.get_mut(&StringName::new(name)) {
            Some(a) => {
                a.speed = speed;
                true
            }
            None => false,
        }
    }

    pub fn get_animation_speed(&self, name: &str) -> Option<f64> {
        self.animations.get(&StringName::new(name)).map(|a| a.speed)
    }

    pub fn set_animation_loop_mode(&mut self, name: &str, mode: LoopMode) -> bool {
        match self.animations.get_mut(&StringName::new(name)) {
            Some(a) => {
                a.loop_mode = mode;
                true
            }
            None => false,
        }
    }

    pub fn get_animation_loop_mode(&self, name: &str) -> Option<LoopMode> {
        self.animations
            .get(&StringName::new(name))
            .map(|a| a.loop_mode)
    }

    /// Inserts at `at_pos`; negative appends.
    pub fn add_frame(&mut self, name: &str, frame: Frame, at_pos: i32) -> bool {
        let Some(a) = self.animations.get_mut(&StringName::new(name)) else {
            return false;
        };
        if at_pos < 0 || at_pos as usize >= a.frames.len() {
            a.frames.push(frame);
        } else {
            a.frames.insert(at_pos as usize, frame);
        }
        true
    }

    pub fn set_frame(&mut self, name: &str, index: usize, frame: Frame) -> bool {
        let Some(a) = self.animations.get_mut(&StringName::new(name)) else {
            return false;
        };
        match a.frames.get_mut(index) {
            Some(f) => {
                *f = frame;
                true
            }
            None => false,
        }
    }

    pub fn get_frame(&self, name: &str, index: usize) -> Option<Frame> {
        self.animations
            .get(&StringName::new(name))
            .and_then(|a| a.frames.get(index).copied())
    }

    pub fn remove_frame(&mut self, name: &str, index: usize) -> bool {
        let Some(a) = self.animations.get_mut(&StringName::new(name)) else {
            return false;
        };
        if index >= a.frames.len() {
            return false;
        }
        a.frames.remove(index);
        true
    }

    pub fn get_frame_count(&self, name: &str) -> i32 {
        self.animations
            .get(&StringName::new(name))
            .map(|a| a.frames.len() as i32)
            .unwrap_or(0)
    }

    /// `get_frame_duration` falls back to 1.0 for a missing animation or an
    /// out-of-range index, like the engine's `ERR_FAIL_COND_V`.
    pub fn get_frame_duration(&self, name: &str, index: usize) -> f32 {
        self.get_frame(name, index).map(|f| f.duration).unwrap_or(1.0)
    }

    pub fn get_animation_duration(&self, name: &str) -> f32 {
        let Some(a) = self.animations.get(&StringName::new(name)) else {
            return 0.0;
        };
        let total: f32 = a.frames.iter().map(|f| f.duration).sum();
        total / a.speed.max(f64::EPSILON) as f32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdvanceResult {
    pub frame_changed: bool,
    pub finished: bool,
}

#[derive(Clone, Debug)]
pub struct AnimatedSprite2DData {
    frames: Option<SpriteFrames>,
    animation: StringName,
    frame: i32,
    speed_scale: f32,
    playing: bool,
    accumulator: f32,
    pingpong_backwards: bool,
    centered: bool,
    offset: Vector2,
    flip_h: bool,
    flip_v: bool,
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
            animation: StringName::new("default"),
            frame: 0,
            speed_scale: 1.0,
            playing: false,
            accumulator: 0.0,
            pingpong_backwards: false,
            centered: true,
            offset: Vector2::ZERO,
            flip_h: false,
            flip_v: false,
        }
    }

    pub fn set_frames(&mut self, frames: Option<SpriteFrames>) {
        self.frames = frames;
        self.frame = 0;
        self.accumulator = 0.0;
        self.playing = false;
    }

    pub fn get_frames(&self) -> Option<&SpriteFrames> {
        self.frames.as_ref()
    }

    pub fn set_animation(&mut self, animation: &str) {
        if self.animation.as_string() == animation {
            return;
        }
        self.animation = StringName::new(animation);
        self.frame = 0;
        self.accumulator = 0.0;
        self.pingpong_backwards = false;
    }

    pub fn get_animation(&self) -> StringName {
        self.animation
    }

    pub fn set_frame(&mut self, frame: i32) {
        self.frame = frame.max(0);
    }

    pub fn get_frame(&self) -> i32 {
        self.frame
    }

    pub fn set_speed_scale(&mut self, scale: f32) {
        self.speed_scale = scale;
    }

    pub fn get_speed_scale(&self) -> f32 {
        self.speed_scale
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.accumulator = 0.0;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
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
        self.flip_h = flip;
    }

    pub fn is_flipped_h(&self) -> bool {
        self.flip_h
    }

    pub fn set_flip_v(&mut self, flip: bool) {
        self.flip_v = flip;
    }

    pub fn is_flipped_v(&self) -> bool {
        self.flip_v
    }

    /// Frame duration in seconds: base duration divided by the animation speed
    /// times `speed_scale`; zero speed never advances.
    fn frame_duration(&self) -> Option<f32> {
        let frames = self.frames.as_ref()?;
        let name = self.animation.as_string();
        let base = frames.get_frame_duration(&name, self.frame.max(0) as usize);
        let speed = frames.get_animation_speed(&name).unwrap_or(5.0) as f32;
        let scale = speed * self.speed_scale;
        if scale == 0.0 {
            None
        } else {
            Some(base / scale)
        }
    }

    fn frame_count(&self) -> i32 {
        self.frames
            .as_ref()
            .map(|f| f.get_frame_count(&self.animation.as_string()))
            .unwrap_or(0)
    }

    /// Advances the animation. `finished` is set when `LOOP_NONE` reaches the
    /// last frame; looping animations wrap silently.
    pub fn advance(&mut self, delta: f32) -> AdvanceResult {
        let mut result = AdvanceResult::default();
        if !self.playing {
            return result;
        }
        let Some(duration) = self.frame_duration() else {
            return result;
        };
        if duration <= 0.0 {
            return result;
        }

        self.accumulator += delta;
        let count = self.frame_count();
        if count <= 0 {
            return result;
        }
        let loop_mode = self
            .frames
            .as_ref()
            .and_then(|f| f.get_animation_loop_mode(&self.animation.as_string()))
            .unwrap_or(LoopMode::Linear);

        while self.accumulator >= duration {
            self.accumulator -= duration;
            match loop_mode {
                LoopMode::None => {
                    if self.frame + 1 < count {
                        self.frame += 1;
                        result.frame_changed = true;
                    } else {
                        self.playing = false;
                        self.accumulator = 0.0;
                        result.finished = true;
                        break;
                    }
                }
                LoopMode::Linear => {
                    self.frame = (self.frame + 1) % count;
                    result.frame_changed = true;
                }
                LoopMode::PingPong => {
                    if self.pingpong_backwards {
                        self.frame -= 1;
                        if self.frame <= 0 {
                            self.frame = 0;
                            self.pingpong_backwards = false;
                        }
                    } else {
                        self.frame += 1;
                        if self.frame >= count - 1 {
                            self.frame = count - 1;
                            self.pingpong_backwards = true;
                        }
                    }
                    result.frame_changed = true;
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames_with(count: usize, duration: f32, mode: LoopMode) -> SpriteFrames {
        let mut f = SpriteFrames::new();
        f.add_animation("run");
        f.set_animation_loop_mode("run", mode);
        f.set_animation_speed("run", 1.0);
        for _ in 0..count {
            f.add_frame("run", Frame::new(None, duration), -1);
        }
        f
    }

    #[test]
    fn default_animation_and_durations() {
        let f = SpriteFrames::new();
        assert!(f.has_animation("default"));
        assert_eq!(f.get_frame_count("default"), 1);
        assert_eq!(f.get_animation_speed("default"), Some(5.0));
        // Missing animation or index falls back to 1.0.
        assert_eq!(f.get_frame_duration("nope", 0), 1.0);
        assert_eq!(f.get_frame_duration("default", 9), 1.0);
    }

    #[test]
    fn add_remove_rename() {
        let mut f = SpriteFrames::new();
        f.add_animation("walk");
        assert!(f.has_animation("walk"));
        assert!(f.rename_animation("walk", "sprint"));
        assert!(f.has_animation("sprint") && !f.has_animation("walk"));
        assert!(f.remove_animation("sprint"));
        assert!(!f.remove_animation("sprint"));
    }

    #[test]
    fn linear_animation_wraps() {
        let mut a = AnimatedSprite2DData::new();
        a.set_frames(Some(frames_with(3, 0.1, LoopMode::Linear)));
        a.set_animation("run");
        a.play();

        assert_eq!(a.get_frame(), 0);
        a.advance(0.1);
        assert_eq!(a.get_frame(), 1);
        a.advance(0.2);
        assert_eq!(a.get_frame(), 0);
        assert!(a.is_playing());
    }

    #[test]
    fn non_looping_animation_stops_on_the_last_frame() {
        let mut a = AnimatedSprite2DData::new();
        a.set_frames(Some(frames_with(2, 0.1, LoopMode::None)));
        a.set_animation("run");
        a.play();

        a.advance(0.1);
        assert_eq!(a.get_frame(), 1);
        let r = a.advance(0.1);
        assert!(r.finished);
        assert!(!a.is_playing());
        assert_eq!(a.get_frame(), 1);
    }

    #[test]
    fn pingpong_reverses_at_the_ends() {
        let mut a = AnimatedSprite2DData::new();
        a.set_frames(Some(frames_with(3, 0.1, LoopMode::PingPong)));
        a.set_animation("run");
        a.play();

        a.advance(0.1);
        a.advance(0.1);
        assert_eq!(a.get_frame(), 2);
        a.advance(0.1); // reverses
        assert_eq!(a.get_frame(), 1);
        a.advance(0.1);
        assert_eq!(a.get_frame(), 0);
        a.advance(0.1); // forward again
        assert_eq!(a.get_frame(), 1);
    }

    #[test]
    fn speed_scale_changes_the_step_and_zero_freezes() {
        let mut a = AnimatedSprite2DData::new();
        a.set_frames(Some(frames_with(3, 0.1, LoopMode::Linear)));
        a.set_animation("run");
        a.play();
        a.set_speed_scale(2.0);
        // duration = 0.1 / (1.0 * 2.0) = 0.05
        a.advance(0.05);
        assert_eq!(a.get_frame(), 1);

        a.set_speed_scale(0.0);
        let r = a.advance(10.0);
        assert!(!r.frame_changed);
    }

    #[test]
    fn changing_animation_resets_the_frame() {
        let mut a = AnimatedSprite2DData::new();
        let mut f = frames_with(3, 0.1, LoopMode::Linear);
        f.add_animation("idle");
        f.add_frame("idle", Frame::new(None, 0.1), -1);
        a.set_frames(Some(f));
        a.set_animation("run");
        a.play();
        a.advance(0.1);
        assert_eq!(a.get_frame(), 1);

        a.set_animation("idle");
        assert_eq!(a.get_frame(), 0);
    }
}
