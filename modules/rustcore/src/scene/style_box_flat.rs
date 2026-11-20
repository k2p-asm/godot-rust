//! `scene/resources/style_box_flat.*`: the flat style box.
//!
//! Ported state: the background/shadow/border colours, the four border widths,
//! expand margins and corner radii, centre drawing, border blending, skew,
//! anti-aliasing (with its size), corner detail, shadow size/offset, the
//! border-width style margins and `get_draw_rect` (expand margins plus the
//! shadow rect merge).
//!
//! Drawing is not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

pub const SIDE_LEFT: usize = 0;
pub const SIDE_TOP: usize = 1;
pub const SIDE_RIGHT: usize = 2;
pub const SIDE_BOTTOM: usize = 3;

/// `Corner`: top-left, top-right, bottom-right, bottom-left.
pub const CORNER_TOP_LEFT: usize = 0;
pub const CORNER_TOP_RIGHT: usize = 1;
pub const CORNER_BOTTOM_RIGHT: usize = 2;
pub const CORNER_BOTTOM_LEFT: usize = 3;

#[derive(Clone, Debug)]
pub struct StyleBoxFlatData {
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
            bg_color: Color::new(0.6, 0.6, 0.6, 1.0),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.6),
            border_color: Color::new(0.8, 0.8, 0.8, 1.0),
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

    pub fn set_border_width_all(&mut self, size: i32) {
        self.border_width = [size as f32; 4];
    }

    pub fn set_border_width(&mut self, side: usize, width: i32) {
        if let Some(slot) = self.border_width.get_mut(side) {
            *slot = width as f32;
        }
    }

    pub fn get_border_width(&self, side: usize) -> f32 {
        self.border_width.get(side).copied().unwrap_or(0.0)
    }

    /// `get_border_width_min`: the smallest of the four sides.
    pub fn get_border_width_min(&self) -> f32 {
        self.border_width
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min)
    }

    pub fn set_border_blend(&mut self, blend: bool) {
        self.blend_border = blend;
    }

    pub fn get_border_blend(&self) -> bool {
        self.blend_border
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

    pub fn set_corner_radius(&mut self, corner: usize, radius: i32) {
        if let Some(slot) = self.corner_radius.get_mut(corner) {
            *slot = radius as f32;
        }
    }

    pub fn get_corner_radius(&self, corner: usize) -> f32 {
        self.corner_radius.get(corner).copied().unwrap_or(0.0)
    }

    pub fn set_corner_detail(&mut self, detail: i32) {
        self.corner_detail = detail;
    }

    pub fn get_corner_detail(&self) -> i32 {
        self.corner_detail
    }

    pub fn set_expand_margin(&mut self, side: usize, size: f32) {
        if let Some(slot) = self.expand_margin.get_mut(side) {
            *slot = size;
        }
    }

    pub fn set_expand_margin_all(&mut self, size: f32) {
        self.expand_margin = [size; 4];
    }

    pub fn set_expand_margin_individual(
        &mut self,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    ) {
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

    pub fn set_anti_aliased(&mut self, enabled: bool) {
        self.anti_aliased = enabled;
    }

    pub fn is_anti_aliased(&self) -> bool {
        self.anti_aliased
    }

    pub fn set_aa_size(&mut self, size: f32) {
        self.aa_size = size;
    }

    pub fn get_aa_size(&self) -> f32 {
        self.aa_size
    }

    /// `get_style_margin`: the border width of the side.
    pub fn get_style_margin(&self, side: usize) -> f32 {
        self.get_border_width(side)
    }

    /// `get_draw_rect`: expand each side, then merge the shadow rect when the
    /// shadow size is positive.
    pub fn get_draw_rect(&self, rect: Rect2) -> Rect2 {
        let mut draw = Rect2::new(
            Vector2::new(
                rect.position.x - self.expand_margin[SIDE_LEFT],
                rect.position.y - self.expand_margin[SIDE_TOP],
            ),
            Vector2::new(
                rect.size.x + self.expand_margin[SIDE_LEFT] + self.expand_margin[SIDE_RIGHT],
                rect.size.y + self.expand_margin[SIDE_TOP] + self.expand_margin[SIDE_BOTTOM],
            ),
        );
        if self.shadow_size > 0 {
            let grow = self.shadow_size as f32;
            let shadow = Rect2::new(
                Vector2::new(draw.position.x - grow, draw.position.y - grow) + self.shadow_offset,
                Vector2::new(draw.size.x + grow * 2.0, draw.size.y + grow * 2.0),
            );
            let min = Vector2::new(
                draw.position.x.min(shadow.position.x),
                draw.position.y.min(shadow.position.y),
            );
            let max = Vector2::new(
                (draw.position.x + draw.size.x).max(shadow.position.x + shadow.size.x),
                (draw.position.y + draw.size.y).max(shadow.position.y + shadow.size.y),
            );
            draw = Rect2::new(min, max - min);
        }
        draw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let style = StyleBoxFlatData::new();
        assert_eq!(style.get_bg_color(), Color::new(0.6, 0.6, 0.6, 1.0));
        assert_eq!(style.get_border_color(), Color::new(0.8, 0.8, 0.8, 1.0));
        assert_eq!(style.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.6));
        assert_eq!(style.get_border_width(SIDE_LEFT), 0.0);
        assert_eq!(style.get_expand_margin(SIDE_TOP), 0.0);
        assert_eq!(style.get_corner_radius(CORNER_TOP_LEFT), 0.0);
        assert!(style.is_draw_center_enabled());
        assert!(!style.get_border_blend());
        assert_eq!(style.get_skew(), Vector2::ZERO);
        assert!(style.is_anti_aliased());
        assert_eq!(style.get_aa_size(), 1.0);
        assert_eq!(style.get_corner_detail(), 8);
        assert_eq!(style.get_shadow_size(), 0);
        assert_eq!(style.get_shadow_offset(), Vector2::ZERO);
        assert_eq!(style.get_style_margin(SIDE_LEFT), 0.0);
    }

    #[test]
    fn border_widths_and_margins() {
        let mut style = StyleBoxFlatData::new();
        style.set_border_width_all(3);
        assert_eq!(style.get_border_width(SIDE_BOTTOM), 3.0);
        assert_eq!(style.get_border_width_min(), 3.0);
        assert_eq!(style.get_style_margin(SIDE_RIGHT), 3.0);

        style.set_border_width(SIDE_TOP, 7);
        assert_eq!(style.get_border_width(SIDE_TOP), 7.0);
        assert_eq!(style.get_border_width_min(), 3.0);
        style.set_border_width(9, 1); // ignored
        assert_eq!(style.get_border_width(9), 0.0);
        style.set_border_blend(true);
        assert!(style.get_border_blend());
        style.set_border_color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(style.get_border_color(), Color::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn corners_and_expand_margins() {
        let mut style = StyleBoxFlatData::new();
        style.set_corner_radius_all(4);
        assert_eq!(style.get_corner_radius(CORNER_BOTTOM_LEFT), 4.0);
        style.set_corner_radius(CORNER_TOP_RIGHT, 9);
        assert_eq!(style.get_corner_radius(CORNER_TOP_RIGHT), 9.0);
        style.set_corner_radius_individual(1, 2, 3, 4);
        assert_eq!(style.get_corner_radius(CORNER_TOP_LEFT), 1.0);
        assert_eq!(style.get_corner_radius(CORNER_TOP_RIGHT), 2.0);
        assert_eq!(style.get_corner_radius(CORNER_BOTTOM_RIGHT), 3.0);
        assert_eq!(style.get_corner_radius(CORNER_BOTTOM_LEFT), 4.0);
        style.set_corner_detail(16);
        assert_eq!(style.get_corner_detail(), 16);

        style.set_expand_margin_all(2.0);
        assert_eq!(style.get_expand_margin(SIDE_LEFT), 2.0);
        style.set_expand_margin(SIDE_TOP, 5.0);
        assert_eq!(style.get_expand_margin(SIDE_TOP), 5.0);
        style.set_expand_margin_individual(0.0, 1.0, 2.0, 3.0);
        assert_eq!(style.get_expand_margin(SIDE_RIGHT), 2.0);
        assert_eq!(style.get_expand_margin(SIDE_BOTTOM), 3.0);
    }

    #[test]
    fn draw_rect_without_and_with_shadow() {
        let mut style = StyleBoxFlatData::new();
        style.set_expand_margin_individual(1.0, 2.0, 3.0, 4.0);
        let rect = Rect2::new(Vector2::new(10.0, 10.0), Vector2::new(20.0, 20.0));
        let draw = style.get_draw_rect(rect);
        assert_eq!(draw.position, Vector2::new(9.0, 8.0));
        assert_eq!(draw.size, Vector2::new(24.0, 26.0));

        // A shadow grows the rect and is offset.
        style.set_shadow_size(5);
        style.set_shadow_offset(Vector2::new(2.0, 3.0));
        let draw = style.get_draw_rect(rect);
        // Expand rect (9,8,24x26) merged with shadow (6,6,34x36) -> (6,6,34x36).
        assert_eq!(draw.position, Vector2::new(6.0, 6.0));
        assert_eq!(draw.size, Vector2::new(34.0, 36.0));

        // Negative/zero shadow sizes do not merge.
        let mut plain = StyleBoxFlatData::new();
        plain.set_shadow_size(0);
        assert_eq!(
            plain.get_draw_rect(rect),
            Rect2::new(Vector2::new(10.0, 10.0), Vector2::new(20.0, 20.0))
        );
    }

    #[test]
    fn visual_flags() {
        let mut style = StyleBoxFlatData::new();
        style.set_draw_center(false);
        style.set_skew(Vector2::new(0.25, -0.5));
        style.set_shadow_color(Color::new(0.1, 0.2, 0.3, 0.4));
        style.set_shadow_size(8);
        style.set_shadow_offset(Vector2::new(-1.0, 1.0));
        style.set_anti_aliased(false);
        style.set_aa_size(1.5);
        style.set_bg_color(Color::new(0.0, 1.0, 0.0, 1.0));

        assert!(!style.is_draw_center_enabled());
        assert_eq!(style.get_skew(), Vector2::new(0.25, -0.5));
        assert_eq!(style.get_shadow_color(), Color::new(0.1, 0.2, 0.3, 0.4));
        assert_eq!(style.get_shadow_size(), 8);
        assert_eq!(style.get_shadow_offset(), Vector2::new(-1.0, 1.0));
        assert!(!style.is_anti_aliased());
        assert_eq!(style.get_aa_size(), 1.5);
        assert_eq!(style.get_bg_color(), Color::new(0.0, 1.0, 0.0, 1.0));
    }
}
