//! Small `scene/gui/` controls: `TextureRect`, `NinePatchRect`, `ColorRect`
//! and `Separator`.
//!
//! State only; textures are reduced to an optional `Rid` handle, so the
//! minimum-size computations that need the texture dimensions are not ported.
//! Defaults and enum order come from the headers.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExpandMode {
    KeepSize,
    IgnoreSize,
    FitWidth,
    FitWidthProportional,
    FitHeight,
    FitHeightProportional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StretchMode {
    Scale,
    Tile,
    Keep,
    KeepCentered,
    KeepAspect,
    KeepAspectCentered,
    KeepAspectCovered,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureRectData {
    texture: Option<Rid>,
    expand_mode: ExpandMode,
    stretch_mode: StretchMode,
    hflip: bool,
    vflip: bool,
}

impl Default for TextureRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureRectData {
    pub fn new() -> Self {
        Self {
            texture: None,
            expand_mode: ExpandMode::KeepSize,
            stretch_mode: StretchMode::Scale,
            hflip: false,
            vflip: false,
        }
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_expand_mode(&mut self, mode: ExpandMode) {
        self.expand_mode = mode;
    }

    pub fn get_expand_mode(&self) -> ExpandMode {
        self.expand_mode
    }

    pub fn set_stretch_mode(&mut self, mode: StretchMode) {
        self.stretch_mode = mode;
    }

    pub fn get_stretch_mode(&self) -> StretchMode {
        self.stretch_mode
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AxisStretchMode {
    Stretch,
    Tile,
    TileFit,
}

#[derive(Clone, Copy, Debug)]
pub struct NinePatchRectData {
    draw_center: bool,
    /// `[left, top, right, bottom]` patch margins, in pixels.
    margins: [i32; 4],
    region_rect: Rect2,
    texture: Option<Rid>,
    axis_h: AxisStretchMode,
    axis_v: AxisStretchMode,
}

impl Default for NinePatchRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl NinePatchRectData {
    pub fn new() -> Self {
        Self {
            draw_center: true,
            margins: [0; 4],
            region_rect: Rect2::new(crate::math::vector2::Vector2::ZERO, crate::math::vector2::Vector2::ZERO),
            texture: None,
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

    /// Sides are `[left, top, right, bottom]`; 4 and up are rejected.
    pub fn set_patch_margin(&mut self, side: usize, size: i32) -> bool {
        if side >= 4 {
            return false;
        }
        self.margins[side] = size;
        true
    }

    pub fn get_patch_margin(&self, side: usize) -> i32 {
        self.margins.get(side).copied().unwrap_or(0)
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

    pub fn set_h_axis_stretch_mode(&mut self, mode: AxisStretchMode) {
        self.axis_h = mode;
    }

    pub fn get_h_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_h
    }

    pub fn set_v_axis_stretch_mode(&mut self, mode: AxisStretchMode) {
        self.axis_v = mode;
    }

    pub fn get_v_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_v
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ColorRectData {
    color: Color,
}

impl Default for ColorRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorRectData {
    pub fn new() -> Self {
        Self {
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub struct SeparatorData {
    orientation: Orientation,
}

impl SeparatorData {
    /// `Separator` is horizontal, `VSeparator` vertical.
    pub fn new(orientation: Orientation) -> Self {
        Self { orientation }
    }

    pub fn horizontal() -> Self {
        Self::new(Orientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(Orientation::Vertical)
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_rect_defaults_and_enums() {
        let mut t = TextureRectData::new();
        assert!(t.get_texture().is_none());
        assert_eq!(t.get_expand_mode(), ExpandMode::KeepSize);
        assert_eq!(t.get_stretch_mode(), StretchMode::Scale);
        assert!(!t.is_flipped_h());
        assert!(!t.is_flipped_v());

        t.set_flip_h(true);
        t.set_flip_v(true);
        assert!(t.is_flipped_h());
        assert!(t.is_flipped_v());
        t.set_expand_mode(ExpandMode::FitWidthProportional);
        t.set_stretch_mode(StretchMode::KeepAspectCovered);
        assert_eq!(t.get_expand_mode(), ExpandMode::FitWidthProportional);
        assert_eq!(t.get_stretch_mode(), StretchMode::KeepAspectCovered);

        // Declaration order, as bound by VARIANT_ENUM_CAST.
        assert_eq!(ExpandMode::KeepSize as i32, 0);
        assert_eq!(ExpandMode::IgnoreSize as i32, 1);
        assert_eq!(ExpandMode::FitWidth as i32, 2);
        assert_eq!(ExpandMode::FitWidthProportional as i32, 3);
        assert_eq!(ExpandMode::FitHeight as i32, 4);
        assert_eq!(ExpandMode::FitHeightProportional as i32, 5);

        assert_eq!(StretchMode::Scale as i32, 0);
        assert_eq!(StretchMode::Tile as i32, 1);
        assert_eq!(StretchMode::Keep as i32, 2);
        assert_eq!(StretchMode::KeepCentered as i32, 3);
        assert_eq!(StretchMode::KeepAspect as i32, 4);
        assert_eq!(StretchMode::KeepAspectCentered as i32, 5);
        assert_eq!(StretchMode::KeepAspectCovered as i32, 6);
    }

    #[test]
    fn nine_patch_defaults_and_margins() {
        let mut n = NinePatchRectData::new();
        assert!(n.is_draw_center_enabled());
        assert_eq!(n.get_region_rect(), Rect2::new(crate::math::vector2::Vector2::ZERO, crate::math::vector2::Vector2::ZERO));
        assert_eq!(n.get_h_axis_stretch_mode(), AxisStretchMode::Stretch);
        assert_eq!(n.get_v_axis_stretch_mode(), AxisStretchMode::Stretch);
        for side in 0..4 {
            assert_eq!(n.get_patch_margin(side), 0);
        }

        assert!(n.set_patch_margin(0, 8));
        assert!(n.set_patch_margin(3, 12));
        assert_eq!(n.get_patch_margin(0), 8);
        assert_eq!(n.get_patch_margin(3), 12);
        assert_eq!(n.get_patch_margin(1), 0);
        assert!(!n.set_patch_margin(4, 1));
        assert_eq!(n.get_patch_margin(9), 0);

        n.set_draw_center(false);
        n.set_h_axis_stretch_mode(AxisStretchMode::TileFit);
        n.set_v_axis_stretch_mode(AxisStretchMode::Tile);
        n.set_region_rect(Rect2::new(
            crate::math::vector2::Vector2::new(1.0, 2.0),
            crate::math::vector2::Vector2::new(3.0, 4.0),
        ));
        assert!(!n.is_draw_center_enabled());
        assert_eq!(n.get_h_axis_stretch_mode(), AxisStretchMode::TileFit);
        assert_eq!(n.get_v_axis_stretch_mode(), AxisStretchMode::Tile);
        assert_eq!(n.get_region_rect().position, crate::math::vector2::Vector2::new(1.0, 2.0));

        assert_eq!(AxisStretchMode::Stretch as i32, 0);
        assert_eq!(AxisStretchMode::Tile as i32, 1);
        assert_eq!(AxisStretchMode::TileFit as i32, 2);
    }

    #[test]
    fn color_rect_and_separator() {
        let mut c = ColorRectData::new();
        assert_eq!(c.get_color(), Color::rgb(1.0, 1.0, 1.0));
        c.set_color(Color::new(0.1, 0.2, 0.3, 0.4));
        assert_eq!(c.get_color(), Color::new(0.1, 0.2, 0.3, 0.4));

        assert_eq!(SeparatorData::horizontal().get_orientation(), Orientation::Horizontal);
        assert_eq!(SeparatorData::vertical().get_orientation(), Orientation::Vertical);
        assert_eq!(Orientation::Horizontal as i32, 0);
        assert_eq!(Orientation::Vertical as i32, 1);
    }
}
