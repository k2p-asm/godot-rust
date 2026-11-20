//! `scene/gui/link_button.*`, `scene/gui/nine_patch_rect.*` and
//! `scene/gui/reference_rect.*`: three small widget states.
//!
//! - `LinkButton`: empty text, underline `ALWAYS`, no URI, auto text direction,
//!   default structured-text parser, empty parser arguments, no overrun
//!   trimming and the engine's ellipsis character;
//! - `NinePatchRect`: draw centre on, zero patch margins, empty region, no
//!   texture and stretch axis modes;
//! - `ReferenceRect`: red border, width 1, editor-only on.
//!
//! Drawing, text shaping and input are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::servers::rid::Rid;

// -- LinkButton -------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum UnderlineMode {
    Always = 0,
    OnHover = 1,
    Never = 2,
}

#[derive(Clone, Debug)]
pub struct LinkButtonData {
    text: String,
    underline_mode: UnderlineMode,
    uri: String,
    language: String,
    text_direction: i32,
    structured_text_parser: i32,
    structured_text_args: Vec<String>,
    overrun_behavior: i32,
    ellipsis_char: String,
}

impl Default for LinkButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkButtonData {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            underline_mode: UnderlineMode::Always,
            uri: String::new(),
            language: String::new(),
            text_direction: 0,
            structured_text_parser: 0,
            structured_text_args: Vec::new(),
            overrun_behavior: 0,
            ellipsis_char: "\u{2026}".to_string(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_underline_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => UnderlineMode::Always,
            1 => UnderlineMode::OnHover,
            2 => UnderlineMode::Never,
            _ => return false,
        };
        self.underline_mode = parsed;
        true
    }

    pub fn get_underline_mode(&self) -> UnderlineMode {
        self.underline_mode
    }

    pub fn set_uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
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

    pub fn set_structured_text_parser(&mut self, parser: i32) {
        self.structured_text_parser = parser;
    }

    pub fn get_structured_text_parser(&self) -> i32 {
        self.structured_text_parser
    }

    pub fn set_structured_text_args(&mut self, args: Vec<String>) {
        self.structured_text_args = args;
    }

    pub fn get_structured_text_args(&self) -> &[String] {
        &self.structured_text_args
    }

    pub fn set_text_overrun_behavior(&mut self, behavior: i32) {
        self.overrun_behavior = behavior;
    }

    pub fn get_text_overrun_behavior(&self) -> i32 {
        self.overrun_behavior
    }

    pub fn set_ellipsis_char(&mut self, ellipsis: &str) {
        self.ellipsis_char = ellipsis.to_string();
    }

    pub fn get_ellipsis_char(&self) -> &str {
        &self.ellipsis_char
    }
}

// -- NinePatchRect ----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AxisStretchMode {
    Stretch = 0,
    Tile = 1,
    TileFit = 2,
}

#[derive(Clone, Debug)]
pub struct NinePatchRectData {
    texture: Option<Rid>,
    draw_center: bool,
    patch_margins: [i32; 4],
    region_rect: Rect2,
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
            texture: None,
            draw_center: true,
            patch_margins: [0; 4],
            region_rect: Rect2::default(),
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

    /// `Side`: 0 = left, 1 = top, 2 = right, 3 = bottom.
    pub fn set_patch_margin(&mut self, side: i32, size: i32) -> bool {
        if !(0..4).contains(&side) {
            return false;
        }
        self.patch_margins[side as usize] = size;
        true
    }

    pub fn get_patch_margin(&self, side: i32) -> i32 {
        if !(0..4).contains(&side) {
            return 0;
        }
        self.patch_margins[side as usize]
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
        match mode {
            0 => self.axis_h = AxisStretchMode::Stretch,
            1 => self.axis_h = AxisStretchMode::Tile,
            2 => self.axis_h = AxisStretchMode::TileFit,
            _ => return false,
        }
        true
    }

    pub fn get_h_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_h
    }

    pub fn set_v_axis_stretch_mode(&mut self, mode: i32) -> bool {
        match mode {
            0 => self.axis_v = AxisStretchMode::Stretch,
            1 => self.axis_v = AxisStretchMode::Tile,
            2 => self.axis_v = AxisStretchMode::TileFit,
            _ => return false,
        }
        true
    }

    pub fn get_v_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_v
    }
}

// -- ReferenceRect ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ReferenceRectData {
    border_color: Color,
    border_width: f32,
    editor_only: bool,
}

impl Default for ReferenceRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceRectData {
    pub fn new() -> Self {
        Self {
            border_color: Color::new(1.0, 0.0, 0.0, 1.0),
            border_width: 1.0,
            editor_only: true,
        }
    }

    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    pub fn get_border_color(&self) -> Color {
        self.border_color
    }

    pub fn set_border_width(&mut self, width: f32) {
        self.border_width = width;
    }

    pub fn get_border_width(&self) -> f32 {
        self.border_width
    }

    pub fn set_editor_only(&mut self, enabled: bool) {
        self.editor_only = enabled;
    }

    pub fn get_editor_only(&self) -> bool {
        self.editor_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_button_state() {
        let mut link = LinkButtonData::new();
        assert_eq!(link.get_text(), "");
        assert_eq!(link.get_underline_mode(), UnderlineMode::Always);
        assert_eq!(link.get_uri(), "");
        assert_eq!(link.get_language(), "");
        assert_eq!(link.get_text_direction(), 0);
        assert_eq!(link.get_structured_text_parser(), 0);
        assert!(link.get_structured_text_args().is_empty());
        assert_eq!(link.get_text_overrun_behavior(), 0);
        assert_eq!(link.get_ellipsis_char(), "\u{2026}");

        link.set_text("Docs");
        assert!(link.set_underline_mode(2));
        assert!(!link.set_underline_mode(3));
        link.set_uri("https://example.com");
        link.set_language("en");
        link.set_text_direction(2);
        link.set_structured_text_parser(1);
        link.set_structured_text_args(vec!["a".to_string(), "b".to_string()]);
        link.set_text_overrun_behavior(3);
        link.set_ellipsis_char("...");

        assert_eq!(link.get_text(), "Docs");
        assert_eq!(link.get_underline_mode(), UnderlineMode::Never);
        assert_eq!(link.get_uri(), "https://example.com");
        assert_eq!(link.get_language(), "en");
        assert_eq!(link.get_text_direction(), 2);
        assert_eq!(link.get_structured_text_parser(), 1);
        assert_eq!(link.get_structured_text_args(), &["a".to_string(), "b".to_string()]);
        assert_eq!(link.get_text_overrun_behavior(), 3);
        assert_eq!(link.get_ellipsis_char(), "...");
        assert_eq!(UnderlineMode::OnHover as i32, 1);
    }

    #[test]
    fn nine_patch_state() {
        let mut patch = NinePatchRectData::new();
        assert!(patch.get_texture().is_none());
        assert!(patch.is_draw_center_enabled());
        assert_eq!(patch.get_patch_margin(0), 0);
        assert_eq!(patch.get_region_rect(), Rect2::default());
        assert_eq!(patch.get_h_axis_stretch_mode(), AxisStretchMode::Stretch);
        assert_eq!(patch.get_v_axis_stretch_mode(), AxisStretchMode::Stretch);

        patch.set_texture(Some(Rid::from_u64(2)));
        patch.set_draw_center(false);
        assert!(patch.set_patch_margin(0, 4));
        assert!(patch.set_patch_margin(3, 8));
        assert!(!patch.set_patch_margin(4, 1));
        patch.set_region_rect(Rect2::new(
            crate::math::vector2::Vector2::new(1.0, 2.0),
            crate::math::vector2::Vector2::new(8.0, 16.0),
        ));
        assert!(patch.set_h_axis_stretch_mode(2));
        assert!(patch.set_v_axis_stretch_mode(1));
        assert!(!patch.set_h_axis_stretch_mode(3));

        assert_eq!(patch.get_texture(), Some(Rid::from_u64(2)));
        assert!(!patch.is_draw_center_enabled());
        assert_eq!(patch.get_patch_margin(0), 4);
        assert_eq!(patch.get_patch_margin(3), 8);
        assert_eq!(patch.get_patch_margin(9), 0);
        assert_eq!(patch.get_region_rect().size, crate::math::vector2::Vector2::new(8.0, 16.0));
        assert_eq!(patch.get_h_axis_stretch_mode(), AxisStretchMode::TileFit);
        assert_eq!(patch.get_v_axis_stretch_mode(), AxisStretchMode::Tile);
        assert_eq!(AxisStretchMode::Tile as i32, 1);
    }

    #[test]
    fn reference_rect_state() {
        let mut rect = ReferenceRectData::new();
        assert_eq!(rect.get_border_color(), Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(rect.get_border_width(), 1.0);
        assert!(rect.get_editor_only());

        rect.set_border_color(Color::new(0.0, 1.0, 0.0, 0.5));
        rect.set_border_width(3.5);
        rect.set_editor_only(false);
        assert_eq!(rect.get_border_color(), Color::new(0.0, 1.0, 0.0, 0.5));
        assert_eq!(rect.get_border_width(), 3.5);
        assert!(!rect.get_editor_only());
    }
}
