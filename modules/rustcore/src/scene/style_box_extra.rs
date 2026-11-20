//! `scene/resources/style_box_line.*` and `style_box_texture.*`: two style boxes.
//!
//! Ported state and margins:
//!
//! - `StyleBoxLine`: colour (opaque black default), thickness 1, vertical flag,
//!   grow begin/end 1.0, and `get_style_margin` (half the thickness on the two
//!   sides the line spans).
//! - `StyleBoxTexture`: texture handle, the four texture/expand margins, region
//!   rect, centre drawing (on), white modulate and the per-axis stretch modes
//!   (stretch by default). `get_style_margin` returns the texture margin.
//!
//! Drawing is not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::servers::rid::Rid;

/// `Side`: left, top, right, bottom.
pub const SIDE_LEFT: usize = 0;
pub const SIDE_TOP: usize = 1;
pub const SIDE_RIGHT: usize = 2;
pub const SIDE_BOTTOM: usize = 3;

#[derive(Clone, Debug)]
pub struct StyleBoxLineData {
    color: Color,
    thickness: i32,
    vertical: bool,
    grow_begin: f32,
    grow_end: f32,
}

impl Default for StyleBoxLineData {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBoxLineData {
    pub fn new() -> Self {
        Self {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            thickness: 1,
            vertical: false,
            grow_begin: 1.0,
            grow_end: 1.0,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_thickness(&mut self, thickness: i32) {
        self.thickness = thickness;
    }

    pub fn get_thickness(&self) -> i32 {
        self.thickness
    }

    pub fn set_vertical(&mut self, vertical: bool) {
        self.vertical = vertical;
    }

    pub fn is_vertical(&self) -> bool {
        self.vertical
    }

    pub fn set_grow_begin(&mut self, grow: f32) {
        self.grow_begin = grow;
    }

    pub fn get_grow_begin(&self) -> f32 {
        self.grow_begin
    }

    pub fn set_grow_end(&mut self, grow: f32) {
        self.grow_end = grow;
    }

    pub fn get_grow_end(&self) -> f32 {
        self.grow_end
    }

    /// `get_style_margin`: half the thickness where the line spans, 0 elsewhere.
    pub fn get_style_margin(&self, side: usize) -> f32 {
        if side > 3 {
            return 0.0;
        }
        if self.vertical {
            if side == SIDE_LEFT || side == SIDE_RIGHT {
                return self.thickness as f32 / 2.0;
            }
        } else if side == SIDE_TOP || side == SIDE_BOTTOM {
            return self.thickness as f32 / 2.0;
        }
        0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AxisStretchMode {
    Stretch = 0,
    Tile = 1,
    TileFit = 2,
}

#[derive(Clone, Debug)]
pub struct StyleBoxTextureData {
    texture: Option<Rid>,
    texture_margin: [f32; 4],
    expand_margin: [f32; 4],
    region_rect: Rect2,
    draw_center: bool,
    modulate: Color,
    axis_h: AxisStretchMode,
    axis_v: AxisStretchMode,
}

impl Default for StyleBoxTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleBoxTextureData {
    pub fn new() -> Self {
        Self {
            texture: None,
            texture_margin: [0.0; 4],
            expand_margin: [0.0; 4],
            region_rect: Rect2::default(),
            draw_center: true,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            axis_h: AxisStretchMode::Stretch,
            axis_v: AxisStretchMode::Stretch,
        }
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    /// `set_texture_margin`: out-of-range sides are ignored.
    pub fn set_texture_margin(&mut self, side: usize, size: f32) {
        if let Some(slot) = self.texture_margin.get_mut(side) {
            *slot = size;
        }
    }

    pub fn set_texture_margin_all(&mut self, size: f32) {
        self.texture_margin = [size; 4];
    }

    pub fn set_texture_margin_individual(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        self.texture_margin = [left, top, right, bottom];
    }

    pub fn get_texture_margin(&self, side: usize) -> f32 {
        self.texture_margin.get(side).copied().unwrap_or(0.0)
    }

    pub fn set_expand_margin(&mut self, side: usize, size: f32) {
        if let Some(slot) = self.expand_margin.get_mut(side) {
            *slot = size;
        }
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

    pub fn set_region_rect(&mut self, rect: Rect2) {
        self.region_rect = rect;
    }

    pub fn get_region_rect(&self) -> Rect2 {
        self.region_rect
    }

    pub fn set_draw_center(&mut self, enabled: bool) {
        self.draw_center = enabled;
    }

    pub fn is_draw_center_enabled(&self) -> bool {
        self.draw_center
    }

    pub fn set_h_axis_stretch_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => AxisStretchMode::Stretch,
            1 => AxisStretchMode::Tile,
            2 => AxisStretchMode::TileFit,
            _ => return false,
        };
        self.axis_h = parsed;
        true
    }

    pub fn get_h_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_h
    }

    pub fn set_v_axis_stretch_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => AxisStretchMode::Stretch,
            1 => AxisStretchMode::Tile,
            2 => AxisStretchMode::TileFit,
            _ => return false,
        };
        self.axis_v = parsed;
        true
    }

    pub fn get_v_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_v
    }

    pub fn set_modulate(&mut self, color: Color) {
        self.modulate = color;
    }

    pub fn get_modulate(&self) -> Color {
        self.modulate
    }

    /// `get_style_margin`: the texture margin of the requested side.
    pub fn get_style_margin(&self, side: usize) -> f32 {
        self.get_texture_margin(side)
    }

    /// `get_draw_rect`: the region plus the expand margins.
    pub fn get_draw_rect(&self, rect: Rect2) -> Rect2 {
        Rect2::new(
            crate::math::vector2::Vector2::new(
                rect.position.x - self.expand_margin[SIDE_LEFT],
                rect.position.y - self.expand_margin[SIDE_TOP],
            ),
            crate::math::vector2::Vector2::new(
                rect.size.x + self.expand_margin[SIDE_LEFT] + self.expand_margin[SIDE_RIGHT],
                rect.size.y + self.expand_margin[SIDE_TOP] + self.expand_margin[SIDE_BOTTOM],
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn line_defaults_and_margins() {
        let mut line = StyleBoxLineData::new();
        assert_eq!(line.get_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(line.get_thickness(), 1);
        assert!(!line.is_vertical());
        assert_eq!(line.get_grow_begin(), 1.0);
        assert_eq!(line.get_grow_end(), 1.0);
        // Horizontal line: top/bottom margins are half the thickness.
        assert_eq!(line.get_style_margin(SIDE_TOP), 0.5);
        assert_eq!(line.get_style_margin(SIDE_BOTTOM), 0.5);
        assert_eq!(line.get_style_margin(SIDE_LEFT), 0.0);
        assert_eq!(line.get_style_margin(SIDE_RIGHT), 0.0);
        assert_eq!(line.get_style_margin(9), 0.0);

        line.set_vertical(true);
        line.set_thickness(6);
        assert_eq!(line.get_style_margin(SIDE_LEFT), 3.0);
        assert_eq!(line.get_style_margin(SIDE_RIGHT), 3.0);
        assert_eq!(line.get_style_margin(SIDE_TOP), 0.0);

        line.set_color(Color::new(1.0, 0.0, 0.0, 0.5));
        line.set_grow_begin(0.25);
        line.set_grow_end(0.75);
        assert_eq!(line.get_color(), Color::new(1.0, 0.0, 0.0, 0.5));
        assert_eq!(line.get_grow_begin(), 0.25);
        assert_eq!(line.get_grow_end(), 0.75);
    }

    #[test]
    fn texture_defaults_and_margins() {
        let mut box_ = StyleBoxTextureData::new();
        assert!(box_.get_texture().is_none());
        assert_eq!(box_.get_texture_margin(SIDE_LEFT), 0.0);
        assert_eq!(box_.get_expand_margin(SIDE_TOP), 0.0);
        assert_eq!(box_.get_region_rect(), Rect2::default());
        assert!(box_.is_draw_center_enabled());
        assert_eq!(box_.get_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(box_.get_h_axis_stretch_mode(), AxisStretchMode::Stretch);
        assert_eq!(box_.get_v_axis_stretch_mode(), AxisStretchMode::Stretch);

        box_.set_texture(Some(Rid::from_u64(4)));
        box_.set_texture_margin(SIDE_LEFT, 8.0);
        box_.set_texture_margin(SIDE_RIGHT, 9.0);
        box_.set_texture_margin(7, 1.0); // ignored
        assert_eq!(box_.get_texture_margin(SIDE_LEFT), 8.0);
        assert_eq!(box_.get_texture_margin(SIDE_RIGHT), 9.0);
        assert_eq!(box_.get_style_margin(SIDE_LEFT), 8.0);

        box_.set_texture_margin_all(3.0);
        assert_eq!(box_.get_texture_margin(SIDE_TOP), 3.0);
        box_.set_texture_margin_individual(1.0, 2.0, 3.0, 5.0);
        assert_eq!(box_.get_texture_margin(SIDE_BOTTOM), 5.0);

        box_.set_expand_margin_all(2.0);
        assert_eq!(box_.get_expand_margin(SIDE_RIGHT), 2.0);
        box_.set_expand_margin_individual(0.0, 1.0, 2.0, 3.0);
        assert_eq!(box_.get_expand_margin(SIDE_TOP), 1.0);
        assert_eq!(box_.get_expand_margin(SIDE_BOTTOM), 3.0);

        box_.set_region_rect(Rect2::new(Vector2::new(4.0, 8.0), Vector2::new(16.0, 32.0)));
        assert_eq!(box_.get_region_rect().size, Vector2::new(16.0, 32.0));
        box_.set_draw_center(false);
        assert!(!box_.is_draw_center_enabled());
        box_.set_modulate(Color::new(1.0, 0.5, 0.25, 1.0));
        assert_eq!(box_.get_modulate(), Color::new(1.0, 0.5, 0.25, 1.0));
    }

    #[test]
    fn texture_stretch_modes_and_draw_rect() {
        let mut box_ = StyleBoxTextureData::new();
        assert!(box_.set_h_axis_stretch_mode(2));
        assert!(box_.set_v_axis_stretch_mode(1));
        assert!(!box_.set_h_axis_stretch_mode(3));
        assert_eq!(box_.get_h_axis_stretch_mode(), AxisStretchMode::TileFit);
        assert_eq!(box_.get_v_axis_stretch_mode(), AxisStretchMode::Tile);
        assert_eq!(AxisStretchMode::Stretch as i32, 0);
        assert_eq!(AxisStretchMode::Tile as i32, 1);
        assert_eq!(AxisStretchMode::TileFit as i32, 2);

        box_.set_expand_margin_individual(1.0, 2.0, 3.0, 4.0);
        let rect = box_.get_draw_rect(Rect2::new(
            Vector2::new(10.0, 10.0),
            Vector2::new(20.0, 20.0),
        ));
        assert_eq!(rect.position, Vector2::new(9.0, 8.0));
        assert_eq!(rect.size, Vector2::new(24.0, 26.0));
    }
}
