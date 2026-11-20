//! `scene/resources/font.h` (state): `FontFile`, `FontVariation` and `SystemFont`.
//!
//! Ported configuration and stored data:
//!
//! - `FontFile`: source bytes, antialiasing (gray), mipmaps off, embedded bitmaps
//!   disabled, MSDF off (`16`/`48` defaults), fixed size 0 with scaling disabled,
//!   autohinter off, colour-glyph modulation off, system fallback on, light
//!   hinting, automatic subpixel positioning, rounding remainders kept,
//!   oversampling override 0, language/script support overrides and OpenType
//!   feature overrides;
//! - `FontVariation`: base font handle, variation OpenType coordinates, embolden,
//!   face index, variation transform, feature overrides, per-spacing-type extra
//!   spacing, baseline offset, palette index and custom colours;
//! - `SystemFont`: names, italic (off), weight 400, stretch 100, the same
//!   antialiasing/hinting/subpixel/MSDF defaults as `FontFile`.
//!
//! Glyph shape/raster and the TextServer calls are not ported.

use crate::math::color::Color;
use crate::math::transform_2d::Transform2D;
use crate::servers::rid::Rid;

pub const FONT_ANTIALIASING_NONE: i32 = 0;
pub const FONT_ANTIALIASING_GRAY: i32 = 1;
pub const FONT_ANTIALIASING_LCD: i32 = 2;

pub const HINTING_NONE: i32 = 0;
pub const HINTING_LIGHT: i32 = 1;
pub const HINTING_NORMAL: i32 = 2;

pub const SUBPIXEL_POSITIONING_DISABLED: i32 = 0;
pub const SUBPIXEL_POSITIONING_AUTO: i32 = 1;
pub const SUBPIXEL_POSITIONING_ONE_HALF: i32 = 2;
pub const SUBPIXEL_POSITIONING_ONE_QUARTER: i32 = 3;

pub const FIXED_SIZE_SCALE_DISABLE: i32 = 0;

/// `SpacingType`: glyph, space, top, bottom.
pub const SPACING_GLYPH: usize = 0;
pub const SPACING_SPACE: usize = 1;
pub const SPACING_TOP: usize = 2;
pub const SPACING_BOTTOM: usize = 3;

#[derive(Clone, Debug)]
pub struct FontFileData {
    data: Vec<u8>,
    antialiasing: i32,
    mipmaps: bool,
    disable_embedded_bitmaps: bool,
    msdf: bool,
    msdf_pixel_range: i32,
    msdf_size: i32,
    fixed_size: i32,
    fixed_size_scale_mode: i32,
    force_autohinter: bool,
    modulate_color_glyphs: bool,
    allow_system_fallback: bool,
    hinting: i32,
    subpixel_positioning: i32,
    keep_rounding_remainders: bool,
    oversampling_override: f64,
    language_support_overrides: Vec<(String, bool)>,
    script_support_overrides: Vec<(String, bool)>,
    feature_overrides: Vec<(String, i64)>,
}

impl Default for FontFileData {
    fn default() -> Self {
        Self::new()
    }
}

impl FontFileData {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            antialiasing: FONT_ANTIALIASING_GRAY,
            mipmaps: false,
            disable_embedded_bitmaps: true,
            msdf: false,
            msdf_pixel_range: 16,
            msdf_size: 48,
            fixed_size: 0,
            fixed_size_scale_mode: FIXED_SIZE_SCALE_DISABLE,
            force_autohinter: false,
            modulate_color_glyphs: false,
            allow_system_fallback: true,
            hinting: HINTING_LIGHT,
            subpixel_positioning: SUBPIXEL_POSITIONING_AUTO,
            keep_rounding_remainders: true,
            oversampling_override: 0.0,
            language_support_overrides: Vec::new(),
            script_support_overrides: Vec::new(),
            feature_overrides: Vec::new(),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn set_antialiasing(&mut self, mode: i32) {
        self.antialiasing = mode;
    }

    pub fn get_antialiasing(&self) -> i32 {
        self.antialiasing
    }

    pub fn set_generate_mipmaps(&mut self, enabled: bool) {
        self.mipmaps = enabled;
    }

    pub fn get_generate_mipmaps(&self) -> bool {
        self.mipmaps
    }

    pub fn set_disable_embedded_bitmaps(&mut self, disabled: bool) {
        self.disable_embedded_bitmaps = disabled;
    }

    pub fn get_disable_embedded_bitmaps(&self) -> bool {
        self.disable_embedded_bitmaps
    }

    pub fn set_multichannel_signed_distance_field(&mut self, enabled: bool) {
        self.msdf = enabled;
    }

    pub fn is_multichannel_signed_distance_field(&self) -> bool {
        self.msdf
    }

    pub fn set_msdf_pixel_range(&mut self, range: i32) {
        self.msdf_pixel_range = range;
    }

    pub fn get_msdf_pixel_range(&self) -> i32 {
        self.msdf_pixel_range
    }

    pub fn set_msdf_size(&mut self, size: i32) {
        self.msdf_size = size;
    }

    pub fn get_msdf_size(&self) -> i32 {
        self.msdf_size
    }

    pub fn set_fixed_size(&mut self, size: i32) {
        self.fixed_size = size;
    }

    pub fn get_fixed_size(&self) -> i32 {
        self.fixed_size
    }

    pub fn set_fixed_size_scale_mode(&mut self, mode: i32) {
        self.fixed_size_scale_mode = mode;
    }

    pub fn get_fixed_size_scale_mode(&self) -> i32 {
        self.fixed_size_scale_mode
    }

    pub fn set_force_autohinter(&mut self, enabled: bool) {
        self.force_autohinter = enabled;
    }

    pub fn is_force_autohinter(&self) -> bool {
        self.force_autohinter
    }

    pub fn set_modulate_color_glyphs(&mut self, enabled: bool) {
        self.modulate_color_glyphs = enabled;
    }

    pub fn is_modulate_color_glyphs(&self) -> bool {
        self.modulate_color_glyphs
    }

    pub fn set_allow_system_fallback(&mut self, enabled: bool) {
        self.allow_system_fallback = enabled;
    }

    pub fn is_allow_system_fallback(&self) -> bool {
        self.allow_system_fallback
    }

    pub fn set_hinting(&mut self, mode: i32) {
        self.hinting = mode;
    }

    pub fn get_hinting(&self) -> i32 {
        self.hinting
    }

    pub fn set_subpixel_positioning(&mut self, mode: i32) {
        self.subpixel_positioning = mode;
    }

    pub fn get_subpixel_positioning(&self) -> i32 {
        self.subpixel_positioning
    }

    pub fn set_keep_rounding_remainders(&mut self, enabled: bool) {
        self.keep_rounding_remainders = enabled;
    }

    pub fn get_keep_rounding_remainders(&self) -> bool {
        self.keep_rounding_remainders
    }

    pub fn set_oversampling(&mut self, oversampling: f64) {
        self.oversampling_override = oversampling;
    }

    pub fn get_oversampling(&self) -> f64 {
        self.oversampling_override
    }

    pub fn set_language_support_override(&mut self, language: &str, supported: bool) {
        self.language_support_overrides
            .retain(|(known, _)| known != language);
        self.language_support_overrides
            .push((language.to_string(), supported));
    }

    pub fn get_language_support_override(&self, language: &str) -> Option<bool> {
        self.language_support_overrides
            .iter()
            .find(|(known, _)| known == language)
            .map(|(_, value)| *value)
    }

    pub fn remove_language_support_override(&mut self, language: &str) {
        self.language_support_overrides
            .retain(|(known, _)| known != language);
    }

    pub fn set_script_support_override(&mut self, script: &str, supported: bool) {
        self.script_support_overrides
            .retain(|(known, _)| known != script);
        self.script_support_overrides
            .push((script.to_string(), supported));
    }

    pub fn get_script_support_override(&self, script: &str) -> Option<bool> {
        self.script_support_overrides
            .iter()
            .find(|(known, _)| known == script)
            .map(|(_, value)| *value)
    }

    pub fn remove_script_support_override(&mut self, script: &str) {
        self.script_support_overrides
            .retain(|(known, _)| known != script);
    }

    pub fn set_opentype_feature_override(&mut self, tag: &str, value: i64) {
        self.feature_overrides.retain(|(known, _)| known != tag);
        self.feature_overrides.push((tag.to_string(), value));
    }

    pub fn get_opentype_feature_override(&self, tag: &str) -> Option<i64> {
        self.feature_overrides
            .iter()
            .find(|(known, _)| known == tag)
            .map(|(_, value)| *value)
    }

    pub fn get_opentype_feature_overrides(&self) -> &[(String, i64)] {
        &self.feature_overrides
    }
}

#[derive(Clone, Debug)]
pub struct FontVariationData {
    base_font: Option<Rid>,
    variation_opentype: Vec<(String, f32)>,
    embolden: f32,
    face_index: i32,
    transform: Transform2D,
    opentype_features: Vec<(String, i64)>,
    extra_spacing: [i64; 4],
    baseline_offset: f32,
    palette_index: i64,
    custom_colors: Vec<Color>,
}

impl Default for FontVariationData {
    fn default() -> Self {
        Self::new()
    }
}

impl FontVariationData {
    pub fn new() -> Self {
        Self {
            base_font: None,
            variation_opentype: Vec::new(),
            embolden: 0.0,
            face_index: 0,
            transform: Transform2D::default(),
            opentype_features: Vec::new(),
            extra_spacing: [0; 4],
            baseline_offset: 0.0,
            palette_index: 0,
            custom_colors: Vec::new(),
        }
    }

    pub fn set_base_font(&mut self, font: Option<Rid>) {
        self.base_font = font;
    }

    pub fn get_base_font(&self) -> Option<Rid> {
        self.base_font
    }

    pub fn set_variation_opentype(&mut self, coords: Vec<(String, f32)>) {
        self.variation_opentype = coords;
    }

    pub fn get_variation_opentype(&self) -> &[(String, f32)] {
        &self.variation_opentype
    }

    pub fn set_variation_embolden(&mut self, embolden: f32) {
        self.embolden = embolden;
    }

    pub fn get_variation_embolden(&self) -> f32 {
        self.embolden
    }

    pub fn set_variation_transform(&mut self, transform: Transform2D) {
        self.transform = transform;
    }

    pub fn get_variation_transform(&self) -> Transform2D {
        self.transform
    }

    pub fn set_variation_face_index(&mut self, index: i32) {
        self.face_index = index;
    }

    pub fn get_variation_face_index(&self) -> i32 {
        self.face_index
    }

    pub fn set_opentype_features(&mut self, features: Vec<(String, i64)>) {
        self.opentype_features = features;
    }

    pub fn get_opentype_features(&self) -> &[(String, i64)] {
        &self.opentype_features
    }

    pub fn set_spacing(&mut self, spacing: usize, value: i64) {
        if let Some(slot) = self.extra_spacing.get_mut(spacing) {
            *slot = value;
        }
    }

    pub fn get_spacing(&self, spacing: usize) -> i64 {
        self.extra_spacing.get(spacing).copied().unwrap_or(0)
    }

    pub fn set_baseline_offset(&mut self, offset: f32) {
        self.baseline_offset = offset;
    }

    pub fn get_baseline_offset(&self) -> f32 {
        self.baseline_offset
    }

    pub fn set_palette_index(&mut self, index: i64) {
        self.palette_index = index;
    }

    pub fn get_palette_index(&self) -> i64 {
        self.palette_index
    }

    pub fn set_custom_colors(&mut self, colors: Vec<Color>) {
        self.custom_colors = colors;
    }

    pub fn get_custom_colors(&self) -> &[Color] {
        &self.custom_colors
    }
}

#[derive(Clone, Debug)]
pub struct SystemFontData {
    names: Vec<String>,
    italic: bool,
    weight: i32,
    stretch: i32,
    antialiasing: i32,
    mipmaps: bool,
    disable_embedded_bitmaps: bool,
    force_autohinter: bool,
    modulate_color_glyphs: bool,
    allow_system_fallback: bool,
    hinting: i32,
    subpixel_positioning: i32,
    keep_rounding_remainders: bool,
    oversampling_override: f64,
    msdf: bool,
    msdf_pixel_range: i32,
    msdf_size: i32,
    base_font: Option<Rid>,
}

impl Default for SystemFontData {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemFontData {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            italic: false,
            weight: 400,
            stretch: 100,
            antialiasing: FONT_ANTIALIASING_GRAY,
            mipmaps: false,
            disable_embedded_bitmaps: true,
            force_autohinter: false,
            modulate_color_glyphs: false,
            allow_system_fallback: true,
            hinting: HINTING_LIGHT,
            subpixel_positioning: SUBPIXEL_POSITIONING_AUTO,
            keep_rounding_remainders: true,
            oversampling_override: 0.0,
            msdf: false,
            msdf_pixel_range: 16,
            msdf_size: 48,
            base_font: None,
        }
    }

    pub fn set_font_names(&mut self, names: Vec<String>) {
        self.names = names;
    }

    pub fn get_font_names(&self) -> &[String] {
        &self.names
    }

    pub fn set_font_italic(&mut self, italic: bool) {
        self.italic = italic;
    }

    pub fn get_font_italic(&self) -> bool {
        self.italic
    }

    pub fn set_font_weight(&mut self, weight: i32) {
        self.weight = weight;
    }

    pub fn get_font_weight(&self) -> i32 {
        self.weight
    }

    pub fn set_font_stretch(&mut self, stretch: i32) {
        self.stretch = stretch;
    }

    pub fn get_font_stretch(&self) -> i32 {
        self.stretch
    }

    pub fn set_antialiasing(&mut self, mode: i32) {
        self.antialiasing = mode;
    }

    pub fn get_antialiasing(&self) -> i32 {
        self.antialiasing
    }

    pub fn set_generate_mipmaps(&mut self, enabled: bool) {
        self.mipmaps = enabled;
    }

    pub fn get_generate_mipmaps(&self) -> bool {
        self.mipmaps
    }

    pub fn set_disable_embedded_bitmaps(&mut self, disabled: bool) {
        self.disable_embedded_bitmaps = disabled;
    }

    pub fn get_disable_embedded_bitmaps(&self) -> bool {
        self.disable_embedded_bitmaps
    }

    pub fn set_force_autohinter(&mut self, enabled: bool) {
        self.force_autohinter = enabled;
    }

    pub fn is_force_autohinter(&self) -> bool {
        self.force_autohinter
    }

    pub fn set_modulate_color_glyphs(&mut self, enabled: bool) {
        self.modulate_color_glyphs = enabled;
    }

    pub fn is_modulate_color_glyphs(&self) -> bool {
        self.modulate_color_glyphs
    }

    pub fn set_allow_system_fallback(&mut self, enabled: bool) {
        self.allow_system_fallback = enabled;
    }

    pub fn is_allow_system_fallback(&self) -> bool {
        self.allow_system_fallback
    }

    pub fn set_hinting(&mut self, mode: i32) {
        self.hinting = mode;
    }

    pub fn get_hinting(&self) -> i32 {
        self.hinting
    }

    pub fn set_subpixel_positioning(&mut self, mode: i32) {
        self.subpixel_positioning = mode;
    }

    pub fn get_subpixel_positioning(&self) -> i32 {
        self.subpixel_positioning
    }

    pub fn set_keep_rounding_remainders(&mut self, enabled: bool) {
        self.keep_rounding_remainders = enabled;
    }

    pub fn get_keep_rounding_remainders(&self) -> bool {
        self.keep_rounding_remainders
    }

    pub fn set_oversampling(&mut self, oversampling: f64) {
        self.oversampling_override = oversampling;
    }

    pub fn get_oversampling(&self) -> f64 {
        self.oversampling_override
    }

    pub fn set_multichannel_signed_distance_field(&mut self, enabled: bool) {
        self.msdf = enabled;
    }

    pub fn is_multichannel_signed_distance_field(&self) -> bool {
        self.msdf
    }

    pub fn set_msdf_pixel_range(&mut self, range: i32) {
        self.msdf_pixel_range = range;
    }

    pub fn get_msdf_pixel_range(&self) -> i32 {
        self.msdf_pixel_range
    }

    pub fn set_msdf_size(&mut self, size: i32) {
        self.msdf_size = size;
    }

    pub fn get_msdf_size(&self) -> i32 {
        self.msdf_size
    }

    pub fn get_base_font(&self) -> Option<Rid> {
        self.base_font
    }

    pub fn set_base_font(&mut self, font: Option<Rid>) {
        self.base_font = font;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_file_defaults() {
        let mut file = FontFileData::new();
        assert!(file.get_data().is_empty());
        assert_eq!(file.get_antialiasing(), FONT_ANTIALIASING_GRAY);
        assert!(!file.get_generate_mipmaps());
        assert!(file.get_disable_embedded_bitmaps());
        assert!(!file.is_multichannel_signed_distance_field());
        assert_eq!(file.get_msdf_pixel_range(), 16);
        assert_eq!(file.get_msdf_size(), 48);
        assert_eq!(file.get_fixed_size(), 0);
        assert_eq!(file.get_fixed_size_scale_mode(), FIXED_SIZE_SCALE_DISABLE);
        assert!(!file.is_force_autohinter());
        assert!(!file.is_modulate_color_glyphs());
        assert!(file.is_allow_system_fallback());
        assert_eq!(file.get_hinting(), HINTING_LIGHT);
        assert_eq!(file.get_subpixel_positioning(), SUBPIXEL_POSITIONING_AUTO);
        assert!(file.get_keep_rounding_remainders());
        assert_eq!(file.get_oversampling(), 0.0);
        assert!(file.get_opentype_feature_overrides().is_empty());

        file.set_data(vec![1, 2, 3]);
        file.set_antialiasing(FONT_ANTIALIASING_LCD);
        file.set_generate_mipmaps(true);
        file.set_fixed_size(24);
        file.set_hinting(HINTING_NORMAL);
        file.set_oversampling(2.0);
        assert_eq!(file.get_data(), &[1, 2, 3]);
        assert_eq!(file.get_antialiasing(), FONT_ANTIALIASING_LCD);
        assert!(file.get_generate_mipmaps());
        assert_eq!(file.get_fixed_size(), 24);
        assert_eq!(file.get_hinting(), HINTING_NORMAL);
        assert_eq!(file.get_oversampling(), 2.0);
        assert_eq!(FONT_ANTIALIASING_GRAY, 1);
    }

    #[test]
    fn font_file_overrides() {
        let mut file = FontFileData::new();
        file.set_language_support_override("ja", true);
        file.set_language_support_override("ja", false);
        assert_eq!(file.get_language_support_override("ja"), Some(false));
        assert_eq!(file.get_language_support_override("ko"), None);
        file.remove_language_support_override("ja");
        assert_eq!(file.get_language_support_override("ja"), None);

        file.set_script_support_override("Latn", true);
        assert_eq!(file.get_script_support_override("Latn"), Some(true));
        file.set_script_support_override("Latn", false);
        assert_eq!(file.get_script_support_override("Latn"), Some(false));
        file.remove_script_support_override("Latn");
        assert_eq!(file.get_script_support_override("Latn"), None);

        file.set_opentype_feature_override("kern", 0);
        file.set_opentype_feature_override("kern", 1);
        assert_eq!(file.get_opentype_feature_override("kern"), Some(1));
        assert_eq!(file.get_opentype_feature_overrides().len(), 1);
        assert_eq!(file.get_opentype_feature_override("liga"), None);
    }

    #[test]
    fn font_variation_state() {
        let mut variation = FontVariationData::new();
        assert!(variation.get_base_font().is_none());
        assert_eq!(variation.get_variation_embolden(), 0.0);
        assert_eq!(variation.get_variation_face_index(), 0);
        assert_eq!(variation.get_variation_transform(), Transform2D::default());
        assert_eq!(variation.get_spacing(SPACING_GLYPH), 0);
        assert_eq!(variation.get_baseline_offset(), 0.0);
        assert_eq!(variation.get_palette_index(), 0);
        assert!(variation.get_custom_colors().is_empty());

        variation.set_base_font(Some(Rid::from_u64(2)));
        variation.set_variation_opentype(vec![("wght".to_string(), 700.0)]);
        variation.set_variation_embolden(1.5);
        variation.set_variation_face_index(2);
        variation.set_opentype_features(vec![("kern".to_string(), 1)]);
        variation.set_spacing(SPACING_GLYPH, 3);
        variation.set_spacing(SPACING_SPACE, -1);
        variation.set_spacing(9, 4); // ignored
        variation.set_baseline_offset(0.5);
        variation.set_palette_index(4);
        variation.set_custom_colors(vec![Color::new(1.0, 0.0, 0.0, 1.0)]);

        assert_eq!(variation.get_base_font(), Some(Rid::from_u64(2)));
        assert_eq!(variation.get_variation_opentype()[0].1, 700.0);
        assert_eq!(variation.get_variation_embolden(), 1.5);
        assert_eq!(variation.get_variation_face_index(), 2);
        assert_eq!(variation.get_opentype_features()[0].0, "kern");
        assert_eq!(variation.get_spacing(SPACING_GLYPH), 3);
        assert_eq!(variation.get_spacing(SPACING_SPACE), -1);
        assert_eq!(variation.get_spacing(9), 0);
        assert_eq!(variation.get_baseline_offset(), 0.5);
        assert_eq!(variation.get_palette_index(), 4);
        assert_eq!(variation.get_custom_colors().len(), 1);
        assert_eq!(SPACING_BOTTOM, 3);
    }

    #[test]
    fn system_font_state() {
        let mut font = SystemFontData::new();
        assert!(font.get_font_names().is_empty());
        assert!(!font.get_font_italic());
        assert_eq!(font.get_font_weight(), 400);
        assert_eq!(font.get_font_stretch(), 100);
        assert_eq!(font.get_antialiasing(), FONT_ANTIALIASING_GRAY);
        assert!(!font.get_generate_mipmaps());
        assert!(font.get_disable_embedded_bitmaps());
        assert!(!font.is_force_autohinter());
        assert!(!font.is_modulate_color_glyphs());
        assert!(font.is_allow_system_fallback());
        assert_eq!(font.get_hinting(), HINTING_LIGHT);
        assert_eq!(font.get_subpixel_positioning(), SUBPIXEL_POSITIONING_AUTO);
        assert!(font.get_keep_rounding_remainders());
        assert_eq!(font.get_oversampling(), 0.0);
        assert!(!font.is_multichannel_signed_distance_field());
        assert_eq!(font.get_msdf_pixel_range(), 16);
        assert_eq!(font.get_msdf_size(), 48);
        assert!(font.get_base_font().is_none());

        font.set_font_names(vec!["Arial".to_string(), "Helvetica".to_string()]);
        font.set_font_italic(true);
        font.set_font_weight(700);
        font.set_font_stretch(125);
        font.set_antialiasing(FONT_ANTIALIASING_NONE);
        font.set_generate_mipmaps(true);
        font.set_force_autohinter(true);
        font.set_allow_system_fallback(false);
        font.set_hinting(HINTING_NONE);
        font.set_subpixel_positioning(SUBPIXEL_POSITIONING_ONE_QUARTER);
        font.set_keep_rounding_remainders(false);
        font.set_oversampling(1.5);
        font.set_multichannel_signed_distance_field(true);
        font.set_msdf_pixel_range(8);
        font.set_msdf_size(32);
        font.set_base_font(Some(Rid::from_u64(7)));

        assert_eq!(font.get_font_names(), &["Arial".to_string(), "Helvetica".to_string()]);
        assert!(font.get_font_italic());
        assert_eq!(font.get_font_weight(), 700);
        assert_eq!(font.get_font_stretch(), 125);
        assert_eq!(font.get_antialiasing(), FONT_ANTIALIASING_NONE);
        assert!(font.get_generate_mipmaps());
        assert!(font.is_force_autohinter());
        assert!(!font.is_allow_system_fallback());
        assert_eq!(font.get_hinting(), HINTING_NONE);
        assert_eq!(font.get_subpixel_positioning(), SUBPIXEL_POSITIONING_ONE_QUARTER);
        assert!(!font.get_keep_rounding_remainders());
        assert_eq!(font.get_oversampling(), 1.5);
        assert!(font.is_multichannel_signed_distance_field());
        assert_eq!(font.get_msdf_pixel_range(), 8);
        assert_eq!(font.get_msdf_size(), 32);
        assert_eq!(font.get_base_font(), Some(Rid::from_u64(7)));
    }
}
