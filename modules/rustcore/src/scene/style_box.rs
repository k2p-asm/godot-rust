//! `scene/resources/style_box.*` and `style_box_flat.*`: theme box state.
//!
//! `StyleBox` keeps four content margins initialised to `-1` (unset). `get_margin`
//! falls back to the subclass's style margin, which `StyleBoxFlat` defines as the
//! border width on that side. Everything else here is the flat box's property
//! set, with the engine's clamps:
//!
//! - `corner_detail` is clamped to `1..=20`;
//! - `aa_size` is clamped to `0.01..=10`;
//! - side/corner indices are validated (`ERR_FAIL_INDEX`), values are not.
//!
//! Not ported: `draw`, `get_draw_rect`'s shadow/expand geometry, and the
//! `Resource` plumbing.

use crate::math::color::Color;
use crate::math::vector2::Vector2;

/// Sides are `[left, top, right, bottom]`, like `Side`.
pub const SIDE_LEFT: usize = 0;
pub const SIDE_TOP: usize = 1;
pub const SIDE_RIGHT: usize = 2;
pub const SIDE_BOTTOM: usize = 3;

/// Corners are `[top-left, top-right, bottom-right, bottom-left]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Clone, Copy, Debug)]
pub struct StyleBoxData {
    content_margin: [f32; 4],
}

impl Default for StyleBoxData {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBoxData {
    pub fn new() -> Self {
        Self {
            content_margin: [-1.0; 4],
        }
    }

    pub fn set_content_margin(&mut self, side: usize, value: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.content_margin[side] = value;
        true
    }

    pub fn set_content_margin_all(&mut self, value: f32) {
        self.content_margin = [value; 4];
    }

    pub fn set_content_margin_individual(
        &mut self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) {
        self.content_margin = [left, top, right, bottom];
    }

    /// Raw value, `-1` when unset; invalid sides read as 0.
    pub fn get_content_margin(&self, side: usize) -> f32 {
        self.content_margin.get(side).copied().unwrap_or(0.0)
    }

    /// `StyleBoxEmpty`: style margins are zero.
    pub fn get_margin(&self, side: usize) -> f32 {
        if side >= 4 {
            return 0.0;
        }
        if self.content_margin[side] < 0.0 {
            0.0
        } else {
            self.content_margin[side]
        }
    }

    pub fn get_offset(&self) -> Vector2 {
        Vector2::new(self.get_margin(SIDE_LEFT), self.get_margin(SIDE_TOP))
    }

    pub fn get_minimum_size(&self) -> Vector2 {
        Vector2::new(
            self.get_margin(SIDE_LEFT) + self.get_margin(SIDE_RIGHT),
            self.get_margin(SIDE_TOP) + self.get_margin(SIDE_BOTTOM),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StyleBoxFlatData {
    content_margin: [f32; 4],
    bg_color: Color,
    shadow_color: Color,
    border_color: Color,
    border_width: [f32; 4],
    expand_margin: [f32; 4],
    corner_radius: [f32; 4],
    draw_center: bool,
    blend_border: bool,
    skew: Vector2,
    anti_aliased: bool,
    corner_detail: i32,
    shadow_size: i32,
    shadow_offset: Vector2,
    aa_size: f32,
}

impl Default for StyleBoxFlatData {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBoxFlatData {
    pub fn new() -> Self {
        Self {
            content_margin: [-1.0; 4],
            bg_color: Color::rgb(0.6, 0.6, 0.6),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.6),
            border_color: Color::rgb(0.8, 0.8, 0.8),
            border_width: [0.0; 4],
            expand_margin: [0.0; 4],
            corner_radius: [0.0; 4],
            draw_center: true,
            blend_border: false,
            skew: Vector2::ZERO,
            anti_aliased: true,
            corner_detail: 8,
            shadow_size: 0,
            shadow_offset: Vector2::ZERO,
            aa_size: 1.0,
        }
    }

    // -- content margins (StyleBox base) ------------------------------------

    pub fn set_content_margin(&mut self, side: usize, value: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.content_margin[side] = value;
        true
    }

    pub fn set_content_margin_all(&mut self, value: f32) {
        self.content_margin = [value; 4];
    }

    pub fn set_content_margin_individual(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.content_margin = [left, top, right, bottom];
    }

    pub fn get_content_margin(&self, side: usize) -> f32 {
        self.content_margin.get(side).copied().unwrap_or(0.0)
    }

    /// Unset content margins fall back to the border width (`get_style_margin`).
    pub fn get_margin(&self, side: usize) -> f32 {
        if side >= 4 {
            return 0.0;
        }
        if self.content_margin[side] < 0.0 {
            self.border_width[side]
        } else {
            self.content_margin[side]
        }
    }

    pub fn get_offset(&self) -> Vector2 {
        Vector2::new(self.get_margin(SIDE_LEFT), self.get_margin(SIDE_TOP))
    }

    pub fn get_minimum_size(&self) -> Vector2 {
        Vector2::new(
            self.get_margin(SIDE_LEFT) + self.get_margin(SIDE_RIGHT),
            self.get_margin(SIDE_TOP) + self.get_margin(SIDE_BOTTOM),
        )
    }

    // -- flat properties ----------------------------------------------------

    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
    }

    pub fn get_bg_color(&self) -> Color {
        self.bg_color
    }

    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    pub fn get_border_color(&self) -> Color {
        self.border_color
    }

    pub fn set_border_width_all(&mut self, width: i32) {
        self.border_width = [width as f32; 4];
    }

    pub fn get_border_width_min(&self) -> f32 {
        self.border_width
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
    }

    pub fn set_border_width(&mut self, side: usize, width: i32) -> bool {
        if side >= 4 {
            return false;
        }
        self.border_width[side] = width as f32;
        true
    }

    pub fn get_border_width(&self, side: usize) -> f32 {
        self.border_width.get(side).copied().unwrap_or(0.0)
    }

    pub fn set_border_blend(&mut self, blend: bool) {
        self.blend_border = blend;
    }

    pub fn get_border_blend(&self) -> bool {
        self.blend_border
    }

    pub fn set_corner_radius(&mut self, corner: Corner, radius: i32) -> bool {
        let index = corner as usize;
        if index >= 4 {
            return false;
        }
        self.corner_radius[index] = radius as f32;
        true
    }

    pub fn set_corner_radius_all(&mut self, radius: i32) {
        self.corner_radius = [radius as f32; 4];
    }

    pub fn set_corner_radius_individual(
        &mut self,
        top_left: i32,
        top_right: i32,
        bottom_right: i32,
        bottom_left: i32,
    ) {
        self.corner_radius = [
            top_left as f32,
            top_right as f32,
            bottom_right as f32,
            bottom_left as f32,
        ];
    }

    pub fn get_corner_radius(&self, corner: Corner) -> f32 {
        self.corner_radius[corner as usize]
    }

    /// Clamped to `1..=20` as in the engine.
    pub fn set_corner_detail(&mut self, detail: i32) {
        self.corner_detail = detail.clamp(1, 20);
    }

    pub fn get_corner_detail(&self) -> i32 {
        self.corner_detail
    }

    pub fn set_expand_margin(&mut self, side: usize, size: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.expand_margin[side] = size;
        true
    }

    pub fn set_expand_margin_all(&mut self, size: f32) {
        self.expand_margin = [size; 4];
    }

    pub fn set_expand_margin_individual(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.expand_margin = [left, top, right, bottom];
    }

    pub fn get_expand_margin(&self, side: usize) -> f32 {
        self.expand_margin.get(side).copied().unwrap_or(0.0)
    }

    pub fn set_draw_center(&mut self, enabled: bool) {
        self.draw_center = enabled;
    }

    pub fn is_draw_center_enabled(&self) -> bool {
        self.draw_center
    }

    pub fn set_skew(&mut self, skew: Vector2) {
        self.skew = skew;
    }

    pub fn get_skew(&self) -> Vector2 {
        self.skew
    }

    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    pub fn get_shadow_color(&self) -> Color {
        self.shadow_color
    }

    pub fn set_shadow_size(&mut self, size: i32) {
        self.shadow_size = size;
    }

    pub fn get_shadow_size(&self) -> i32 {
        self.shadow_size
    }

    pub fn set_shadow_offset(&mut self, offset: Vector2) {
        self.shadow_offset = offset;
    }

    pub fn get_shadow_offset(&self) -> Vector2 {
        self.shadow_offset
    }

    pub fn set_anti_aliased(&mut self, anti_aliased: bool) {
        self.anti_aliased = anti_aliased;
    }

    pub fn is_anti_aliased(&self) -> bool {
        self.anti_aliased
    }

    /// Clamped to `0.01..=10`.
    pub fn set_aa_size(&mut self, size: f32) {
        self.aa_size = size.clamp(0.01, 10.0);
    }

    pub fn get_aa_size(&self) -> f32 {
        self.aa_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_style_box_margins() {
        let mut b = StyleBoxData::new();
        for side in 0..4 {
            assert_eq!(b.get_content_margin(side), -1.0);
            assert_eq!(b.get_margin(side), 0.0);
        }
        assert_eq!(b.get_minimum_size(), Vector2::ZERO);
        assert_eq!(b.get_offset(), Vector2::ZERO);

        assert!(b.set_content_margin(SIDE_LEFT, 4.0));
        b.set_content_margin_individual(1.0, 2.0, 3.0, 4.0);
        assert_eq!(b.get_content_margin(SIDE_RIGHT), 3.0);
        assert_eq!(b.get_offset(), Vector2::new(1.0, 2.0));
        assert_eq!(b.get_minimum_size(), Vector2::new(4.0, 6.0));

        b.set_content_margin_all(5.0);
        assert_eq!(b.get_minimum_size(), Vector2::new(10.0, 10.0));

        assert!(!b.set_content_margin(4, 1.0));
        assert_eq!(b.get_content_margin(9), 0.0);
    }

    #[test]
    fn flat_defaults() {
        let f = StyleBoxFlatData::new();
        assert_eq!(f.get_bg_color(), Color::rgb(0.6, 0.6, 0.6));
        assert_eq!(f.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.6));
        assert_eq!(f.get_border_color(), Color::rgb(0.8, 0.8, 0.8));
        assert!(f.is_draw_center_enabled());
        assert!(!f.get_border_blend());
        assert_eq!(f.get_skew(), Vector2::ZERO);
        assert!(f.is_anti_aliased());
        assert_eq!(f.get_corner_detail(), 8);
        assert_eq!(f.get_shadow_size(), 0);
        assert_eq!(f.get_shadow_offset(), Vector2::ZERO);
        assert_eq!(f.get_aa_size(), 1.0);
        for side in 0..4 {
            assert_eq!(f.get_border_width(side), 0.0);
            assert_eq!(f.get_expand_margin(side), 0.0);
        }
    }

    #[test]
    fn flat_margins_fall_back_to_border_width() {
        let mut f = StyleBoxFlatData::new();
        f.set_border_width_all(3);
        assert_eq!(f.get_border_width_min(), 3.0);
        assert_eq!(f.get_margin(SIDE_LEFT), 3.0);
        assert_eq!(f.get_minimum_size(), Vector2::new(6.0, 6.0));
        assert_eq!(f.get_offset(), Vector2::new(3.0, 3.0));

        // A content margin overrides the border for that side only.
        assert!(f.set_content_margin(SIDE_LEFT, 10.0));
        assert_eq!(f.get_margin(SIDE_LEFT), 10.0);
        assert_eq!(f.get_margin(SIDE_RIGHT), 3.0);
        assert_eq!(f.get_minimum_size(), Vector2::new(13.0, 6.0));

        // Setting the content margin back to -1 restores the fallback.
        assert!(f.set_content_margin(SIDE_LEFT, -1.0));
        assert_eq!(f.get_margin(SIDE_LEFT), 3.0);

        // The minimum border is the smallest of the four.
        f.set_border_width(SIDE_TOP, 1);
        assert_eq!(f.get_border_width_min(), 1.0);
        assert!(!f.set_border_width(4, 1));
        assert_eq!(f.get_border_width(9), 0.0);
    }

    #[test]
    fn flat_corner_detail_and_aa_are_clamped() {
        let mut f = StyleBoxFlatData::new();
        f.set_corner_detail(0);
        assert_eq!(f.get_corner_detail(), 1);
        f.set_corner_detail(100);
        assert_eq!(f.get_corner_detail(), 20);
        f.set_corner_detail(12);
        assert_eq!(f.get_corner_detail(), 12);

        f.set_aa_size(0.0);
        assert_eq!(f.get_aa_size(), 0.01);
        f.set_aa_size(99.0);
        assert_eq!(f.get_aa_size(), 10.0);
        f.set_aa_size(2.5);
        assert_eq!(f.get_aa_size(), 2.5);
    }

    #[test]
    fn flat_corners_expand_and_colors() {
        let mut f = StyleBoxFlatData::new();
        assert_eq!(f.get_corner_radius(Corner::TopLeft), 0.0);
        assert!(f.set_corner_radius(Corner::BottomRight, 6));
        assert_eq!(f.get_corner_radius(Corner::BottomRight), 6.0);

        f.set_corner_radius_individual(1, 2, 3, 4);
        assert_eq!(f.get_corner_radius(Corner::TopLeft), 1.0);
        assert_eq!(f.get_corner_radius(Corner::BottomLeft), 4.0);

        f.set_corner_radius_all(7);
        assert_eq!(f.get_corner_radius(Corner::TopRight), 7.0);
        assert_eq!(Corner::BottomLeft as i32, 3);

        f.set_expand_margin_individual(1.0, 2.0, 3.0, 4.0);
        assert_eq!(f.get_expand_margin(SIDE_BOTTOM), 4.0);
        f.set_expand_margin_all(2.0);
        assert_eq!(f.get_expand_margin(SIDE_RIGHT), 2.0);
        assert!(!f.set_expand_margin(4, 1.0));

        f.set_bg_color(Color::new(0.1, 0.2, 0.3, 1.0));
        f.set_border_color(Color::rgb(1.0, 0.0, 0.0));
        f.set_shadow_color(Color::new(1.0, 1.0, 1.0, 0.25));
        assert_eq!(f.get_bg_color(), Color::new(0.1, 0.2, 0.3, 1.0));
        assert_eq!(f.get_border_color(), Color::rgb(1.0, 0.0, 0.0));
        assert_eq!(f.get_shadow_color(), Color::new(1.0, 1.0, 1.0, 0.25));

        f.set_shadow_size(4);
        f.set_shadow_offset(Vector2::new(2.0, -2.0));
        f.set_skew(Vector2::new(0.25, 0.5));
        f.set_draw_center(false);
        f.set_border_blend(true);
        f.set_anti_aliased(false);
        assert_eq!(f.get_shadow_size(), 4);
        assert_eq!(f.get_shadow_offset(), Vector2::new(2.0, -2.0));
        assert_eq!(f.get_skew(), Vector2::new(0.25, 0.5));
        assert!(!f.is_draw_center_enabled());
        assert!(f.get_border_blend());
        assert!(!f.is_anti_aliased());
    }
}
