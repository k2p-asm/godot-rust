//! `scene/3d/sprite_3d.*` (state): `SpriteBase3D` and `Sprite3D`.
//!
//! Ported state: centring (on), offset, flips, modulate, render priority, the
//! billboard axis (`Z`), pixel size (0.01), the five draw flags (all off), the
//! alpha-cut mode (disabled) with scissor/hash/edge values, and for `Sprite3D`
//! the texture handle, region flag/rect, frame index and H/V frame grid with the
//! engine's frame-adjustment rules.
//!
//! Mesh generation, the material and drawing are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawFlags {
    Transparent = 0,
    Shaded = 1,
    DoubleSided = 2,
    DisableDepthTest = 3,
    FixedSize = 4,
    Max = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlphaCutMode {
    Disabled = 0,
    Discard = 1,
    OpaquePrepass = 2,
    Hash = 3,
    Max = 4,
}

#[derive(Clone, Debug)]
pub struct SpriteBase3DData {
    centered: bool,
    offset: Vector2,
    hflip: bool,
    vflip: bool,
    modulate: Color,
    render_priority: i32,
    /// `Vector3::AXIS_Z`.
    axis: i32,
    pixel_size: f32,
    flags: [bool; 5],
    alpha_cut: AlphaCutMode,
    alpha_scissor_threshold: f32,
    alpha_hash_scale: f32,
    alpha_antialiasing_edge: f32,
}

impl Default for SpriteBase3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteBase3DData {
    pub fn new() -> Self {
        Self {
            centered: true,
            offset: Vector2::ZERO,
            hflip: false,
            vflip: false,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            render_priority: 0,
            axis: 2,
            pixel_size: 0.01,
            flags: [false; 5],
            alpha_cut: AlphaCutMode::Disabled,
            alpha_scissor_threshold: 0.5,
            alpha_hash_scale: 1.0,
            alpha_antialiasing_edge: 0.0,
        }
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

    pub fn set_modulate(&mut self, color: Color) {
        self.modulate = color;
    }

    pub fn get_modulate(&self) -> Color {
        self.modulate
    }

    pub fn set_render_priority(&mut self, priority: i32) {
        self.render_priority = priority;
    }

    pub fn get_render_priority(&self) -> i32 {
        self.render_priority
    }

    pub fn set_axis(&mut self, axis: i32) -> bool {
        if !(0..=2).contains(&axis) {
            return false;
        }
        self.axis = axis;
        true
    }

    pub fn get_axis(&self) -> i32 {
        self.axis
    }

    pub fn set_pixel_size(&mut self, size: f32) {
        self.pixel_size = size;
    }

    pub fn get_pixel_size(&self) -> f32 {
        self.pixel_size
    }

    pub fn set_draw_flag(&mut self, flag: i32, enabled: bool) -> bool {
        if !(0..5).contains(&flag) {
            return false;
        }
        self.flags[flag as usize] = enabled;
        true
    }

    pub fn get_draw_flag(&self, flag: i32) -> bool {
        self.flags.get(flag as usize).copied().unwrap_or(false)
    }

    pub fn set_alpha_cut_mode(&mut self, mode: i32) {
        self.alpha_cut = match mode {
            0 => AlphaCutMode::Disabled,
            1 => AlphaCutMode::Discard,
            2 => AlphaCutMode::OpaquePrepass,
            3 => AlphaCutMode::Hash,
            _ => self.alpha_cut,
        };
    }

    pub fn get_alpha_cut_mode(&self) -> AlphaCutMode {
        self.alpha_cut
    }

    pub fn set_alpha_scissor_threshold(&mut self, threshold: f32) {
        self.alpha_scissor_threshold = threshold;
    }

    pub fn get_alpha_scissor_threshold(&self) -> f32 {
        self.alpha_scissor_threshold
    }

    pub fn set_alpha_hash_scale(&mut self, scale: f32) {
        self.alpha_hash_scale = scale;
    }

    pub fn get_alpha_hash_scale(&self) -> f32 {
        self.alpha_hash_scale
    }

    pub fn set_alpha_antialiasing_edge(&mut self, edge: f32) {
        self.alpha_antialiasing_edge = edge;
    }

    pub fn get_alpha_antialiasing_edge(&self) -> f32 {
        self.alpha_antialiasing_edge
    }
}

#[derive(Clone, Debug)]
pub struct Sprite3DData {
    base: SpriteBase3DData,
    texture: Option<Rid>,
    region: bool,
    region_rect: Rect2,
    frame: i32,
    vframes: i32,
    hframes: i32,
}

impl Default for Sprite3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Sprite3DData {
    pub fn new() -> Self {
        Self {
            base: SpriteBase3DData::new(),
            texture: None,
            region: false,
            region_rect: Rect2::default(),
            frame: 0,
            vframes: 1,
            hframes: 1,
        }
    }

    pub fn base(&self) -> &SpriteBase3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpriteBase3DData {
        &mut self.base
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_region_enabled(&mut self, enabled: bool) {
        self.region = enabled;
    }

    pub fn is_region_enabled(&self) -> bool {
        self.region
    }

    pub fn set_region_rect(&mut self, rect: Rect2) {
        self.region_rect = rect;
    }

    pub fn get_region_rect(&self) -> Rect2 {
        self.region_rect
    }

    /// `set_frame`: out-of-range indices are refused.
    pub fn set_frame(&mut self, frame: i32) -> bool {
        if frame < 0 || frame >= self.vframes * self.hframes {
            return false;
        }
        self.frame = frame;
        true
    }

    pub fn get_frame(&self) -> i32 {
        self.frame
    }

    /// `set_frame_coords`.
    pub fn set_frame_coords(&mut self, coords: Vector2i) -> bool {
        if coords.x < 0 || coords.x >= self.hframes || coords.y < 0 || coords.y >= self.vframes {
            return false;
        }
        self.set_frame(coords.y * self.hframes + coords.x)
    }

    /// `get_frame_coords`.
    pub fn get_frame_coords(&self) -> Vector2i {
        Vector2i::new(self.frame % self.hframes, self.frame / self.hframes)
    }

    /// `set_vframes`: values below 1 are refused; a frame past the grid resets.
    pub fn set_vframes(&mut self, amount: i32) -> bool {
        if amount < 1 {
            return false;
        }
        if amount == self.vframes {
            return true;
        }
        self.vframes = amount;
        if self.frame >= self.vframes * self.hframes {
            self.frame = 0;
        }
        true
    }

    pub fn get_vframes(&self) -> i32 {
        self.vframes
    }

    /// `set_hframes`: values below 1 are refused; the frame keeps its column
    /// when possible and is remapped to the new row stride.
    pub fn set_hframes(&mut self, amount: i32) -> bool {
        if amount < 1 {
            return false;
        }
        if amount == self.hframes {
            return true;
        }
        if self.vframes > 1 {
            let original_column = self.frame % self.hframes;
            if original_column >= amount {
                self.frame = 0;
            } else {
                let original_row = self.frame / self.hframes;
                self.frame = original_row * amount + original_column;
            }
        }
        self.hframes = amount;
        if self.frame >= self.vframes * self.hframes {
            self.frame = 0;
        }
        true
    }

    pub fn get_hframes(&self) -> i32 {
        self.hframes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_defaults_and_setters() {
        let mut sprite = SpriteBase3DData::new();
        assert!(sprite.is_centered());
        assert_eq!(sprite.get_offset(), Vector2::ZERO);
        assert!(!sprite.is_flipped_h());
        assert!(!sprite.is_flipped_v());
        assert_eq!(sprite.get_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(sprite.get_render_priority(), 0);
        assert_eq!(sprite.get_axis(), 2);
        assert_eq!(sprite.get_pixel_size(), 0.01);
        for flag in 0..5 {
            assert!(!sprite.get_draw_flag(flag));
        }
        assert_eq!(sprite.get_alpha_cut_mode(), AlphaCutMode::Disabled);
        assert_eq!(sprite.get_alpha_scissor_threshold(), 0.5);
        assert_eq!(sprite.get_alpha_hash_scale(), 1.0);
        assert_eq!(sprite.get_alpha_antialiasing_edge(), 0.0);

        sprite.set_centered(false);
        sprite.set_offset(Vector2::new(1.0, 2.0));
        sprite.set_flip_h(true);
        sprite.set_flip_v(true);
        sprite.set_modulate(Color::new(0.5, 0.25, 0.125, 1.0));
        sprite.set_render_priority(4);
        assert!(sprite.set_axis(0));
        assert!(!sprite.set_axis(3));
        sprite.set_pixel_size(0.05);
        assert!(sprite.set_draw_flag(1, true));
        assert!(!sprite.set_draw_flag(5, true));
        sprite.set_alpha_cut_mode(3);
        sprite.set_alpha_scissor_threshold(0.7);
        sprite.set_alpha_hash_scale(0.5);
        sprite.set_alpha_antialiasing_edge(0.3);

        assert!(!sprite.is_centered());
        assert_eq!(sprite.get_offset(), Vector2::new(1.0, 2.0));
        assert!(sprite.is_flipped_h());
        assert!(sprite.is_flipped_v());
        assert_eq!(sprite.get_modulate(), Color::new(0.5, 0.25, 0.125, 1.0));
        assert_eq!(sprite.get_render_priority(), 4);
        assert_eq!(sprite.get_axis(), 0);
        assert_eq!(sprite.get_pixel_size(), 0.05);
        assert!(sprite.get_draw_flag(1));
        assert!(!sprite.get_draw_flag(0));
        assert_eq!(sprite.get_alpha_cut_mode(), AlphaCutMode::Hash);
        assert_eq!(sprite.get_alpha_scissor_threshold(), 0.7);
        assert_eq!(sprite.get_alpha_hash_scale(), 0.5);
        assert_eq!(sprite.get_alpha_antialiasing_edge(), 0.3);
        assert_eq!(DrawFlags::FixedSize as i32, 4);
        assert_eq!(AlphaCutMode::OpaquePrepass as i32, 2);
    }

    #[test]
    fn sprite3d_defaults_and_texture() {
        let mut sprite = Sprite3DData::new();
        assert!(sprite.get_texture().is_none());
        assert!(!sprite.is_region_enabled());
        assert_eq!(sprite.get_region_rect(), Rect2::default());
        assert_eq!(sprite.get_frame(), 0);
        assert_eq!(sprite.get_hframes(), 1);
        assert_eq!(sprite.get_vframes(), 1);
        assert_eq!(sprite.get_frame_coords(), Vector2i::ZERO);
        assert!(sprite.base().is_centered());

        sprite.set_texture(Some(Rid::from_u64(7)));
        sprite.set_region_enabled(true);
        sprite.set_region_rect(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));
        assert_eq!(sprite.get_texture(), Some(Rid::from_u64(7)));
        assert!(sprite.is_region_enabled());
        assert_eq!(
            sprite.get_region_rect(),
            Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0))
        );
    }

    #[test]
    fn frame_grid_and_coords() {
        let mut sprite = Sprite3DData::new();
        assert!(sprite.set_hframes(4));
        assert!(sprite.set_vframes(2));
        assert_eq!(sprite.get_hframes(), 4);
        assert_eq!(sprite.get_vframes(), 2);
        assert!(sprite.set_frame(5));
        assert_eq!(sprite.get_frame(), 5);
        assert_eq!(sprite.get_frame_coords(), Vector2i::new(1, 1));
        assert!(sprite.set_frame_coords(Vector2i::new(3, 1)));
        assert_eq!(sprite.get_frame(), 7);

        // Range guards.
        assert!(!sprite.set_hframes(0));
        assert!(!sprite.set_vframes(0));
        assert!(!sprite.set_frame(8));
        assert!(!sprite.set_frame(-1));
        assert!(!sprite.set_frame_coords(Vector2i::new(4, 0)));
        assert_eq!(sprite.get_frame(), 7);

        // Growing V frames keeps the frame; shrinking may reset it.
        assert!(sprite.set_vframes(4));
        assert_eq!(sprite.get_frame(), 7);
        assert!(sprite.set_frame(9));
        assert!(sprite.set_vframes(2));
        assert_eq!(sprite.get_frame(), 0, "frame past the new grid resets");
    }

    #[test]
    fn hframes_remaps_the_column() {
        let mut sprite = Sprite3DData::new();
        sprite.set_hframes(4);
        sprite.set_vframes(2);
        sprite.set_frame(6); // column 2, row 1
        assert_eq!(sprite.get_frame_coords(), Vector2i::new(2, 1));

        // A narrower sheet keeps the column and the row stride.
        assert!(sprite.set_hframes(3));
        assert_eq!(sprite.get_frame(), 5, "row 1 * 3 + column 2");
        assert_eq!(sprite.get_frame_coords(), Vector2i::new(2, 1));

        // If the column is dropped the frame resets.
        sprite.set_frame(2);
        assert!(sprite.set_hframes(2));
        assert_eq!(sprite.get_frame(), 0);
    }
}
