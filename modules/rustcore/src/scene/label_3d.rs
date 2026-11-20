//! `scene/3d/label_3d.*` (state only).
//!
//! Defaults: pixel size 0.005, no draw flags, alpha cut disabled with a 0.5
//! scissor threshold and hash scale 1, alpha antialiasing off with edge 0,
//! centred horizontal/vertical alignment, empty text, no uppercase, autowrap
//! off, width 500, font size 32, outline size 12, white modulate, black opaque
//! outline, render priority 0 with outline priority -1 and zero line spacing.
//!
//! The font/texture/mesh generation, autowrap shaping, per-surface material
//! cache and triangle-mesh generation are not ported.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::scene::base_button::{HorizontalAlignment, VerticalAlignment};
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawFlags {
    Shaded,
    DoubleSided,
    DisableDepthTest,
    FixedSize,
    Max,
}

pub const DRAW_FLAG_MAX: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlphaCutMode {
    Disabled,
    Discard,
    OpaquePrepass,
    Hash,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BillboardMode {
    Disabled,
    Enabled,
    FixedY,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TextureFilter {
    Nearest,
    Linear,
    NearestMipmap,
    LinearMipmap,
    NearestMipmapAnisotropic,
    LinearMipmapAnisotropic,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlphaAntiAliasing {
    Off,
    On,
}

#[derive(Clone, Debug)]
pub struct Label3DData {
    pixel_size: f32,
    offset: Vector2,
    flags: [bool; DRAW_FLAG_MAX],
    alpha_cut: AlphaCutMode,
    alpha_scissor_threshold: f32,
    alpha_hash_scale: f32,
    alpha_antialiasing: AlphaAntiAliasing,
    alpha_antialiasing_edge: f32,
    text: String,
    uppercase: bool,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    autowrap_mode: i32,
    width: f32,
    font_size: i32,
    outline_size: i32,
    line_spacing: f32,
    modulate: Color,
    outline_modulate: Color,
    render_priority: i32,
    outline_render_priority: i32,
    billboard: BillboardMode,
    texture_filter: TextureFilter,
    font: Option<Rid>,
    language: String,
    text_direction: i32,
}

impl Default for Label3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Label3DData {
    pub fn new() -> Self {
        Self {
            pixel_size: 0.005,
            offset: Vector2::ZERO,
            flags: [false; DRAW_FLAG_MAX],
            alpha_cut: AlphaCutMode::Disabled,
            alpha_scissor_threshold: 0.5,
            alpha_hash_scale: 1.0,
            alpha_antialiasing: AlphaAntiAliasing::Off,
            alpha_antialiasing_edge: 0.0,
            text: String::new(),
            uppercase: false,
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
            autowrap_mode: 0,
            width: 500.0,
            font_size: 32,
            outline_size: 12,
            line_spacing: 0.0,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_modulate: Color::new(0.0, 0.0, 0.0, 1.0),
            render_priority: 0,
            outline_render_priority: -1,
            billboard: BillboardMode::Disabled,
            texture_filter: TextureFilter::LinearMipmap,
            font: None,
            language: String::new(),
            text_direction: 0,
        }
    }

    pub fn set_pixel_size(&mut self, amount: f32) {
        self.pixel_size = amount;
    }

    pub fn get_pixel_size(&self) -> f32 {
        self.pixel_size
    }

    pub fn set_offset(&mut self, offset: Vector2) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> Vector2 {
        self.offset
    }

    /// `ERR_FAIL_INDEX(flag, FLAG_MAX)`.
    pub fn set_draw_flag(&mut self, flag: i32, enabled: bool) -> bool {
        if !(0..DRAW_FLAG_MAX as i32).contains(&flag) {
            return false;
        }
        self.flags[flag as usize] = enabled;
        true
    }

    pub fn get_draw_flag(&self, flag: i32) -> bool {
        if !(0..DRAW_FLAG_MAX as i32).contains(&flag) {
            return false;
        }
        self.flags[flag as usize]
    }

    /// `ERR_FAIL_INDEX(mode, ALPHA_CUT_MAX)`.
    pub fn set_alpha_cut_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => AlphaCutMode::Disabled,
            1 => AlphaCutMode::Discard,
            2 => AlphaCutMode::OpaquePrepass,
            3 => AlphaCutMode::Hash,
            _ => return false,
        };
        self.alpha_cut = parsed;
        true
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

    pub fn set_alpha_antialiasing(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => AlphaAntiAliasing::Off,
            1 => AlphaAntiAliasing::On,
            _ => return false,
        };
        self.alpha_antialiasing = parsed;
        true
    }

    pub fn get_alpha_antialiasing(&self) -> AlphaAntiAliasing {
        self.alpha_antialiasing
    }

    pub fn set_alpha_antialiasing_edge(&mut self, edge: f32) {
        self.alpha_antialiasing_edge = edge;
    }

    pub fn get_alpha_antialiasing_edge(&self) -> f32 {
        self.alpha_antialiasing_edge
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_uppercase(&mut self, uppercase: bool) {
        self.uppercase = uppercase;
    }

    pub fn is_uppercase(&self) -> bool {
        self.uppercase
    }

    pub fn set_horizontal_alignment(&mut self, alignment: HorizontalAlignment) {
        self.horizontal_alignment = alignment;
    }

    pub fn get_horizontal_alignment(&self) -> HorizontalAlignment {
        self.horizontal_alignment
    }

    pub fn set_vertical_alignment(&mut self, alignment: VerticalAlignment) {
        self.vertical_alignment = alignment;
    }

    pub fn get_vertical_alignment(&self) -> VerticalAlignment {
        self.vertical_alignment
    }

    pub fn set_autowrap_mode(&mut self, mode: i32) {
        self.autowrap_mode = mode;
    }

    pub fn get_autowrap_mode(&self) -> i32 {
        self.autowrap_mode
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn set_font_size(&mut self, size: i32) {
        self.font_size = size;
    }

    pub fn get_font_size(&self) -> i32 {
        self.font_size
    }

    pub fn set_outline_size(&mut self, size: i32) {
        self.outline_size = size;
    }

    pub fn get_outline_size(&self) -> i32 {
        self.outline_size
    }

    pub fn set_line_spacing(&mut self, spacing: f32) {
        self.line_spacing = spacing;
    }

    pub fn get_line_spacing(&self) -> f32 {
        self.line_spacing
    }

    pub fn set_modulate(&mut self, color: Color) {
        self.modulate = color;
    }

    pub fn get_modulate(&self) -> Color {
        self.modulate
    }

    pub fn set_outline_modulate(&mut self, color: Color) {
        self.outline_modulate = color;
    }

    pub fn get_outline_modulate(&self) -> Color {
        self.outline_modulate
    }

    pub fn set_render_priority(&mut self, priority: i32) {
        self.render_priority = priority;
    }

    pub fn get_render_priority(&self) -> i32 {
        self.render_priority
    }

    pub fn set_outline_render_priority(&mut self, priority: i32) {
        self.outline_render_priority = priority;
    }

    pub fn get_outline_render_priority(&self) -> i32 {
        self.outline_render_priority
    }

    pub fn set_billboard_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => BillboardMode::Disabled,
            1 => BillboardMode::Enabled,
            2 => BillboardMode::FixedY,
            _ => return false,
        };
        self.billboard = parsed;
        true
    }

    pub fn get_billboard_mode(&self) -> BillboardMode {
        self.billboard
    }

    pub fn set_texture_filter(&mut self, filter: i32) -> bool {
        let parsed = match filter {
            0 => TextureFilter::Nearest,
            1 => TextureFilter::Linear,
            2 => TextureFilter::NearestMipmap,
            3 => TextureFilter::LinearMipmap,
            4 => TextureFilter::NearestMipmapAnisotropic,
            5 => TextureFilter::LinearMipmapAnisotropic,
            _ => return false,
        };
        self.texture_filter = parsed;
        true
    }

    pub fn get_texture_filter(&self) -> TextureFilter {
        self.texture_filter
    }

    pub fn set_font(&mut self, font: Option<Rid>) {
        self.font = font;
    }

    pub fn get_font(&self) -> Option<Rid> {
        self.font
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_text_direction(&mut self, direction: i32) {
        self.text_direction = direction;
    }

    pub fn get_text_direction(&self) -> i32 {
        self.text_direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let label = Label3DData::new();
        assert_eq!(label.get_pixel_size(), 0.005);
        assert_eq!(label.get_offset(), Vector2::ZERO);
        for flag in 0..4 {
            assert!(!label.get_draw_flag(flag));
        }
        assert_eq!(label.get_alpha_cut_mode(), AlphaCutMode::Disabled);
        assert_eq!(label.get_alpha_scissor_threshold(), 0.5);
        assert_eq!(label.get_alpha_hash_scale(), 1.0);
        assert_eq!(label.get_alpha_antialiasing(), AlphaAntiAliasing::Off);
        assert_eq!(label.get_alpha_antialiasing_edge(), 0.0);
        assert_eq!(label.get_text(), "");
        assert!(!label.is_uppercase());
        assert_eq!(label.get_horizontal_alignment(), HorizontalAlignment::Center);
        assert_eq!(label.get_vertical_alignment(), VerticalAlignment::Center);
        assert_eq!(label.get_autowrap_mode(), 0);
        assert_eq!(label.get_width(), 500.0);
        assert_eq!(label.get_font_size(), 32);
        assert_eq!(label.get_outline_size(), 12);
        assert_eq!(label.get_line_spacing(), 0.0);
        assert_eq!(label.get_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(label.get_outline_modulate(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(label.get_render_priority(), 0);
        assert_eq!(label.get_outline_render_priority(), -1);
        assert_eq!(label.get_billboard_mode(), BillboardMode::Disabled);
        assert_eq!(label.get_texture_filter(), TextureFilter::LinearMipmap);
        assert!(label.get_font().is_none());
        assert_eq!(label.get_language(), "");
        assert_eq!(label.get_text_direction(), 0);
    }

    #[test]
    fn draw_flags_and_guards() {
        let mut label = Label3DData::new();
        assert!(label.set_draw_flag(0, true));
        assert!(label.set_draw_flag(2, true));
        assert!(!label.set_draw_flag(4, true));
        assert!(!label.set_draw_flag(-1, true));
        assert!(label.get_draw_flag(0));
        assert!(!label.get_draw_flag(1));
        assert!(label.get_draw_flag(2));
        assert!(!label.get_draw_flag(4));
        assert_eq!(DrawFlags::Shaded as i32, 0);
        assert_eq!(DrawFlags::FixedSize as i32, 3);

        assert!(label.set_alpha_cut_mode(3));
        assert!(!label.set_alpha_cut_mode(4));
        assert_eq!(label.get_alpha_cut_mode(), AlphaCutMode::Hash);
        assert!(label.set_alpha_antialiasing(1));
        assert!(!label.set_alpha_antialiasing(2));
        assert_eq!(label.get_alpha_antialiasing(), AlphaAntiAliasing::On);
        assert!(label.set_billboard_mode(2));
        assert!(!label.set_billboard_mode(3));
        assert_eq!(label.get_billboard_mode(), BillboardMode::FixedY);
        assert!(label.set_texture_filter(5));
        assert!(!label.set_texture_filter(6));
        assert_eq!(
            label.get_texture_filter(),
            TextureFilter::LinearMipmapAnisotropic
        );
    }

    #[test]
    fn setters_round_trip() {
        let mut label = Label3DData::new();
        label.set_pixel_size(0.01);
        label.set_offset(Vector2::new(1.0, 2.0));
        label.set_alpha_scissor_threshold(0.25);
        label.set_alpha_hash_scale(2.0);
        label.set_alpha_antialiasing_edge(0.5);
        label.set_text("Hello");
        label.set_uppercase(true);
        label.set_horizontal_alignment(HorizontalAlignment::Right);
        label.set_vertical_alignment(VerticalAlignment::Bottom);
        label.set_autowrap_mode(3);
        label.set_width(200.0);
        label.set_font_size(48);
        label.set_outline_size(4);
        label.set_line_spacing(1.5);
        label.set_modulate(Color::new(0.5, 0.25, 0.75, 1.0));
        label.set_outline_modulate(Color::new(0.1, 0.1, 0.1, 1.0));
        label.set_render_priority(3);
        label.set_outline_render_priority(1);
        label.set_font(Some(Rid::from_u64(9)));
        label.set_language("en");
        label.set_text_direction(2);

        assert_eq!(label.get_pixel_size(), 0.01);
        assert_eq!(label.get_offset(), Vector2::new(1.0, 2.0));
        assert_eq!(label.get_alpha_scissor_threshold(), 0.25);
        assert_eq!(label.get_alpha_hash_scale(), 2.0);
        assert_eq!(label.get_alpha_antialiasing_edge(), 0.5);
        assert_eq!(label.get_text(), "Hello");
        assert!(label.is_uppercase());
        assert_eq!(label.get_horizontal_alignment(), HorizontalAlignment::Right);
        assert_eq!(label.get_vertical_alignment(), VerticalAlignment::Bottom);
        assert_eq!(label.get_autowrap_mode(), 3);
        assert_eq!(label.get_width(), 200.0);
        assert_eq!(label.get_font_size(), 48);
        assert_eq!(label.get_outline_size(), 4);
        assert_eq!(label.get_line_spacing(), 1.5);
        assert_eq!(label.get_modulate(), Color::new(0.5, 0.25, 0.75, 1.0));
        assert_eq!(label.get_outline_modulate(), Color::new(0.1, 0.1, 0.1, 1.0));
        assert_eq!(label.get_render_priority(), 3);
        assert_eq!(label.get_outline_render_priority(), 1);
        assert_eq!(label.get_font(), Some(Rid::from_u64(9)));
        assert_eq!(label.get_language(), "en");
        assert_eq!(label.get_text_direction(), 2);
    }
}
