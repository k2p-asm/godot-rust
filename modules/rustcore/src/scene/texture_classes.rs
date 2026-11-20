//! `scene/resources/atlas_texture.*` and `animated_texture.*`: texture state.
//!
//! Textures are `Rid` handles here. `AtlasTexture` keeps the atlas, the
//! (floored) region, the margin and the filter-clip flag; its size is the
//! rounded region size. `AnimatedTexture` keeps up to 256 frames with
//! per-frame durations (default 1 s), 1 frame by default, current frame 0,
//! pause/one-shot off and speed scale 1; the engine's validation is reproduced
//! (frames `1..=256`, current frame inside the count, speed in `-1000..1000`).
//!
//! Playback/proxy updates and image access are not ported.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, Default)]
pub struct AtlasTextureData {
    atlas: Option<Rid>,
    region: Rect2,
    rounded_region: Rect2,
    margin: Rect2,
    filter_clip: bool,
}

impl AtlasTextureData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_atlas(&mut self, atlas: Option<Rid>) {
        self.atlas = atlas;
    }

    pub fn get_atlas(&self) -> Option<Rid> {
        self.atlas
    }

    pub fn set_region(&mut self, region: Rect2) {
        self.region = region;
        self.rounded_region = Rect2::new(
            region.position,
            Vector2::new(region.size.x.floor(), region.size.y.floor()),
        );
    }

    pub fn get_region(&self) -> Rect2 {
        self.region
    }

    pub fn set_margin(&mut self, margin: Rect2) {
        self.margin = margin;
    }

    pub fn get_margin(&self) -> Rect2 {
        self.margin
    }

    pub fn set_filter_clip(&mut self, enable: bool) {
        self.filter_clip = enable;
    }

    pub fn has_filter_clip(&self) -> bool {
        self.filter_clip
    }

    /// `get_width`: the rounded region width.
    pub fn get_width(&self) -> i32 {
        self.rounded_region.size.x as i32
    }

    /// `get_height`: the rounded region height.
    pub fn get_height(&self) -> i32 {
        self.rounded_region.size.y as i32
    }
}

/// `AnimatedTexture::MAX_FRAMES`.
pub const MAX_FRAMES: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct AnimatedFrame {
    pub texture: Option<Rid>,
    pub duration: f32,
}

impl Default for AnimatedFrame {
    fn default() -> Self {
        Self {
            texture: None,
            duration: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimatedTextureData {
    frames: Vec<AnimatedFrame>,
    frame_count: i32,
    current_frame: i32,
    pause: bool,
    one_shot: bool,
    speed_scale: f32,
    time: f32,
}

impl Default for AnimatedTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimatedTextureData {
    pub fn new() -> Self {
        Self {
            frames: vec![AnimatedFrame::default(); MAX_FRAMES],
            frame_count: 1,
            current_frame: 0,
            pause: false,
            one_shot: false,
            speed_scale: 1.0,
            time: 0.0,
        }
    }

    /// Rejects counts outside `1..=MAX_FRAMES`.
    pub fn set_frames(&mut self, frames: i32) -> bool {
        if frames < 1 || frames as usize > MAX_FRAMES {
            return false;
        }
        self.frame_count = frames;
        true
    }

    pub fn get_frames(&self) -> i32 {
        self.frame_count
    }

    /// Rejects frames outside `0..frame_count` and resets the elapsed time.
    pub fn set_current_frame(&mut self, frame: i32) -> bool {
        if frame < 0 || frame >= self.frame_count {
            return false;
        }
        self.current_frame = frame;
        self.time = 0.0;
        true
    }

    pub fn get_current_frame(&self) -> i32 {
        self.current_frame
    }

    pub fn set_pause(&mut self, pause: bool) {
        self.pause = pause;
    }

    pub fn get_pause(&self) -> bool {
        self.pause
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn get_one_shot(&self) -> bool {
        self.one_shot
    }

    /// The index is validated against `MAX_FRAMES`, not the frame count.
    pub fn set_frame_texture(&mut self, frame: usize, texture: Option<Rid>) -> bool {
        if frame >= MAX_FRAMES {
            return false;
        }
        self.frames[frame].texture = texture;
        true
    }

    pub fn get_frame_texture(&self, frame: usize) -> Option<Rid> {
        self.frames.get(frame).and_then(|f| f.texture)
    }

    pub fn set_frame_duration(&mut self, frame: usize, duration: f32) -> bool {
        if frame >= MAX_FRAMES {
            return false;
        }
        self.frames[frame].duration = duration;
        true
    }

    pub fn get_frame_duration(&self, frame: usize) -> f32 {
        self.frames.get(frame).map(|f| f.duration).unwrap_or(0.0)
    }

    /// Rejects `scale < -1000` or `scale >= 1000`.
    pub fn set_speed_scale(&mut self, scale: f32) -> bool {
        if scale < -1000.0 || scale >= 1000.0 {
            return false;
        }
        self.speed_scale = scale;
        true
    }

    pub fn get_speed_scale(&self) -> f32 {
        self.speed_scale
    }

    /// Elapsed time within the current frame, reset by `set_current_frame`.
    pub fn get_time(&self) -> f32 {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_defaults_and_region_rounding() {
        let mut a = AtlasTextureData::new();
        assert!(a.get_atlas().is_none());
        assert_eq!(a.get_region(), Rect2::default());
        assert_eq!(a.get_margin(), Rect2::default());
        assert!(!a.has_filter_clip());

        a.set_atlas(Some(Rid::from_u64(3)));
        assert_eq!(a.get_atlas(), Some(Rid::from_u64(3)));

        a.set_region(Rect2::new(Vector2::new(1.5, 2.25), Vector2::new(10.75, 4.5)));
        // The property keeps the float rect; the size uses the floored rect.
        assert_eq!(a.get_region().size, Vector2::new(10.75, 4.5));
        assert_eq!(a.get_width(), 10);
        assert_eq!(a.get_height(), 4);

        a.set_margin(Rect2::new(Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0)));
        a.set_filter_clip(true);
        assert_eq!(a.get_margin().size, Vector2::new(2.0, 2.0));
        assert!(a.has_filter_clip());
    }

    #[test]
    fn animated_defaults() {
        let a = AnimatedTextureData::new();
        assert_eq!(a.get_frames(), 1);
        assert_eq!(a.get_current_frame(), 0);
        assert!(!a.get_pause());
        assert!(!a.get_one_shot());
        assert_eq!(a.get_speed_scale(), 1.0);
        assert_eq!(a.get_time(), 0.0);
        assert_eq!(a.get_frame_duration(0), 1.0);
        assert!(a.get_frame_texture(0).is_none());
        // All 256 frame slots exist with the default duration.
        assert_eq!(a.get_frame_duration(MAX_FRAMES - 1), 1.0);
    }

    #[test]
    fn animated_validation() {
        let mut a = AnimatedTextureData::new();
        assert!(!a.set_frames(0));
        assert!(!a.set_frames((MAX_FRAMES + 1) as i32));
        assert!(a.set_frames(MAX_FRAMES as i32));
        assert!(a.set_frames(4));
        assert_eq!(a.get_frames(), 4);

        assert!(!a.set_current_frame(-1));
        assert!(!a.set_current_frame(4));
        assert!(a.set_current_frame(3));
        assert_eq!(a.get_current_frame(), 3);

        // The engine allows exactly -1000 but rejects 1000 (`< -1000 || >= 1000`).
        assert!(a.set_speed_scale(-1000.0));
        assert!(!a.set_speed_scale(-1000.5));
        assert!(!a.set_speed_scale(1000.0));
        assert!(a.set_speed_scale(-999.0));
        assert_eq!(a.get_speed_scale(), -999.0);

        // Frame indices are validated against MAX_FRAMES.
        assert!(a.set_frame_texture(200, Some(Rid::from_u64(9))));
        assert_eq!(a.get_frame_texture(200), Some(Rid::from_u64(9)));
        assert!(!a.set_frame_texture(MAX_FRAMES, None));
        assert!(!a.set_frame_duration(MAX_FRAMES, 2.0));
        assert!(a.set_frame_duration(1, 0.25));
        assert_eq!(a.get_frame_duration(1), 0.25);
        assert_eq!(a.get_frame_duration(MAX_FRAMES), 0.0);

        a.set_pause(true);
        a.set_one_shot(true);
        assert!(a.get_pause());
        assert!(a.get_one_shot());
    }
}
