//! `scene/2d/light_2d.*`: light state (no rendering).
//!
//! Defaults come from the header's in-class initialisers and `Light2D::Light2D`.
//! Setters write the value and, in the engine, forward it to the RenderingServer;
//! here only the state is kept. `set_shadow_filter` rejects the `MAX` sentinel,
//! and `PointLight2D::set_texture_scale` turns an exact zero into `CMP_EPSILON`
//! (the engine comment: "avoid having 0 scale values").
//!
//! Not ported: canvas-light RIDs, textures and the actual shading.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

/// `CMP_EPSILON` from `core/typedefs.h`.
const CMP_EPSILON: f32 = 0.00001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShadowFilter {
    None,
    Pcf5,
    Pcf13,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    Add,
    Sub,
    Mix,
}

#[derive(Clone, Copy, Debug)]
pub struct Light2DData {
    enabled: bool,
    editor_only: bool,
    shadow: bool,
    color: Color,
    shadow_color: Color,
    height: f32,
    energy: f32,
    z_min: i32,
    z_max: i32,
    layer_min: i32,
    layer_max: i32,
    item_mask: i32,
    item_shadow_mask: i32,
    shadow_smooth: f32,
    texture: Option<Rid>,
    texture_offset: Vector2,
    shadow_filter: ShadowFilter,
    blend_mode: BlendMode,
}

impl Default for Light2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Light2DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            editor_only: false,
            shadow: false,
            color: Color::rgb(1.0, 1.0, 1.0),
            shadow_color: Color::ZERO,
            height: 0.0,
            energy: 1.0,
            z_min: -1024,
            z_max: 1024,
            layer_min: 0,
            layer_max: 0,
            item_mask: 1,
            item_shadow_mask: 1,
            shadow_smooth: 0.0,
            texture: None,
            texture_offset: Vector2::ZERO,
            shadow_filter: ShadowFilter::None,
            blend_mode: BlendMode::Add,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_editor_only(&mut self, editor_only: bool) {
        self.editor_only = editor_only;
    }

    pub fn is_editor_only(&self) -> bool {
        self.editor_only
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_energy(&mut self, energy: f32) {
        self.energy = energy;
    }

    pub fn get_energy(&self) -> f32 {
        self.energy
    }

    pub fn set_z_range_min(&mut self, min_z: i32) {
        self.z_min = min_z;
    }

    pub fn get_z_range_min(&self) -> i32 {
        self.z_min
    }

    pub fn set_z_range_max(&mut self, max_z: i32) {
        self.z_max = max_z;
    }

    pub fn get_z_range_max(&self) -> i32 {
        self.z_max
    }

    pub fn set_layer_range_min(&mut self, min_layer: i32) {
        self.layer_min = min_layer;
    }

    pub fn get_layer_range_min(&self) -> i32 {
        self.layer_min
    }

    pub fn set_layer_range_max(&mut self, max_layer: i32) {
        self.layer_max = max_layer;
    }

    pub fn get_layer_range_max(&self) -> i32 {
        self.layer_max
    }

    pub fn set_item_cull_mask(&mut self, mask: i32) {
        self.item_mask = mask;
    }

    pub fn get_item_cull_mask(&self) -> i32 {
        self.item_mask
    }

    pub fn set_item_shadow_cull_mask(&mut self, mask: i32) {
        self.item_shadow_mask = mask;
    }

    pub fn get_item_shadow_cull_mask(&self) -> i32 {
        self.item_shadow_mask
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.shadow = enabled;
    }

    pub fn is_shadow_enabled(&self) -> bool {
        self.shadow
    }

    /// Rejects the `SHADOW_FILTER_MAX` sentinel (`ERR_FAIL_INDEX`).
    pub fn set_shadow_filter(&mut self, filter: ShadowFilter) -> bool {
        if filter == ShadowFilter::Max {
            return false;
        }
        self.shadow_filter = filter;
        true
    }

    pub fn get_shadow_filter(&self) -> ShadowFilter {
        self.shadow_filter
    }

    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    pub fn get_shadow_color(&self) -> Color {
        self.shadow_color
    }

    pub fn set_shadow_smooth(&mut self, amount: f32) {
        self.shadow_smooth = amount;
    }

    pub fn get_shadow_smooth(&self) -> f32 {
        self.shadow_smooth
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_texture_offset(&mut self, offset: Vector2) {
        self.texture_offset = offset;
    }

    pub fn get_texture_offset(&self) -> Vector2 {
        self.texture_offset
    }
}

/// `PointLight2D`: a light with a texture and a scale.
#[derive(Clone, Copy, Debug)]
pub struct PointLight2DData {
    pub light: Light2DData,
    scale: f32,
    texture: Option<Rid>,
    texture_offset: Vector2,
}

impl Default for PointLight2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PointLight2DData {
    pub fn new() -> Self {
        Self {
            light: Light2DData::new(),
            scale: 1.0,
            texture: None,
            texture_offset: Vector2::ZERO,
        }
    }

    /// An exact zero becomes `CMP_EPSILON`; other values are stored as given.
    pub fn set_texture_scale(&mut self, scale: f32) {
        self.scale = if scale == 0.0 { CMP_EPSILON } else { scale };
    }

    pub fn get_texture_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_texture_offset(&mut self, offset: Vector2) {
        self.texture_offset = offset;
    }

    pub fn get_texture_offset(&self) -> Vector2 {
        self.texture_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_defaults_match_the_header() {
        let l = Light2DData::new();
        assert!(l.is_enabled());
        assert!(!l.is_editor_only());
        assert!(!l.is_shadow_enabled());
        assert_eq!(l.get_color(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(l.get_shadow_color(), Color::ZERO);
        assert_eq!(l.get_height(), 0.0);
        assert_eq!(l.get_energy(), 1.0);
        assert_eq!(l.get_z_range_min(), -1024);
        assert_eq!(l.get_z_range_max(), 1024);
        assert_eq!(l.get_layer_range_min(), 0);
        assert_eq!(l.get_layer_range_max(), 0);
        assert_eq!(l.get_item_cull_mask(), 1);
        assert_eq!(l.get_item_shadow_cull_mask(), 1);
        assert_eq!(l.get_shadow_smooth(), 0.0);
        assert_eq!(l.get_shadow_filter(), ShadowFilter::None);
        assert_eq!(l.get_blend_mode(), BlendMode::Add);
        assert_eq!(l.get_texture_offset(), Vector2::ZERO);
        assert!(l.get_texture().is_none());
    }

    #[test]
    fn setters_round_trip() {
        let mut l = Light2DData::new();
        l.set_enabled(false);
        l.set_editor_only(true);
        l.set_color(Color::rgb(0.5, 0.25, 0.125));
        l.set_height(32.0);
        l.set_energy(2.5);
        l.set_z_range_min(-10);
        l.set_z_range_max(20);
        l.set_layer_range_min(1);
        l.set_layer_range_max(3);
        l.set_item_cull_mask(0xF);
        l.set_item_shadow_cull_mask(0x3);
        l.set_shadow_enabled(true);
        l.set_shadow_color(Color::new(1.0, 0.0, 0.0, 0.5));
        l.set_shadow_smooth(0.75);
        l.set_blend_mode(BlendMode::Mix);

        assert!(!l.is_enabled());
        assert!(l.is_editor_only());
        assert_eq!(l.get_color(), Color::rgb(0.5, 0.25, 0.125));
        assert_eq!(l.get_height(), 32.0);
        assert_eq!(l.get_energy(), 2.5);
        assert_eq!(l.get_z_range_min(), -10);
        assert_eq!(l.get_z_range_max(), 20);
        assert_eq!(l.get_layer_range_min(), 1);
        assert_eq!(l.get_layer_range_max(), 3);
        assert_eq!(l.get_item_cull_mask(), 0xF);
        assert_eq!(l.get_item_shadow_cull_mask(), 0x3);
        assert!(l.is_shadow_enabled());
        assert_eq!(l.get_shadow_color(), Color::new(1.0, 0.0, 0.0, 0.5));
        assert_eq!(l.get_shadow_smooth(), 0.75);
        assert_eq!(l.get_blend_mode(), BlendMode::Mix);
    }

    #[test]
    fn shadow_filter_rejects_the_max_sentinel() {
        let mut l = Light2DData::new();
        assert!(!l.set_shadow_filter(ShadowFilter::Max));
        assert_eq!(l.get_shadow_filter(), ShadowFilter::None);
        assert!(l.set_shadow_filter(ShadowFilter::Pcf13));
        assert_eq!(l.get_shadow_filter(), ShadowFilter::Pcf13);
    }

    #[test]
    fn enum_values_follow_declaration_order() {
        assert_eq!(ShadowFilter::None as i32, 0);
        assert_eq!(ShadowFilter::Pcf5 as i32, 1);
        assert_eq!(ShadowFilter::Pcf13 as i32, 2);
        assert_eq!(ShadowFilter::Max as i32, 3);

        assert_eq!(BlendMode::Add as i32, 0);
        assert_eq!(BlendMode::Sub as i32, 1);
        assert_eq!(BlendMode::Mix as i32, 2);
    }

    #[test]
    fn point_light_scale_never_becomes_exactly_zero() {
        let mut p = PointLight2DData::new();
        assert_eq!(p.get_texture_scale(), 1.0);
        assert_eq!(p.light.get_energy(), 1.0);

        p.set_texture_scale(0.0);
        assert_eq!(p.get_texture_scale(), CMP_EPSILON);
        p.set_texture_scale(-2.0);
        assert_eq!(p.get_texture_scale(), -2.0);
        p.set_texture_scale(4.0);
        assert_eq!(p.get_texture_scale(), 4.0);

        p.set_texture_offset(Vector2::new(3.0, -1.0));
        assert_eq!(p.get_texture_offset(), Vector2::new(3.0, -1.0));
    }
}
