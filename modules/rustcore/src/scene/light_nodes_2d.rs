//! `scene/2d/light_2d.*`, `scene/2d/light_occluder_2d.*` and
//! `scene/2d/back_buffer_copy.*` (state only).
//!
//! `Light2D` defaults: enabled, not editor-only, shadows off, white colour,
//! transparent shadow colour, height 0, energy 1, z range -1024..1024, layer
//! range 0..0, item and shadow cull masks 1, no shadow smoothing, no texture
//! offset, scale 1, max distance 10000, `SHADOW_FILTER_NONE` and `BLEND_MODE_ADD`.
//!
//! `OccluderPolygon2D` stores the polygon, the `closed` flag (on) and the cull
//! mode (disabled); its item rect is the polygon's bounds with a dirty cache.
//! `LightOccluder2D` keeps the polygon handle, the light mask (1) and the SDF
//! collision flag (off).
//!
//! `BackBufferCopy` starts with `(-100,-100,200,200)` and `COPY_MODE_RECT`.
//!
//! Rendering-server calls, notifications and drawing are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

// -- Light2D ----------------------------------------------------------------

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

#[derive(Clone, Debug)]
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
    texture_scale: f32,
    max_distance: f32,
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
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.0),
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
            texture_scale: 1.0,
            max_distance: 10000.0,
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

    pub fn set_z_range_min(&mut self, min: i32) {
        self.z_min = min;
    }

    pub fn get_z_range_min(&self) -> i32 {
        self.z_min
    }

    pub fn set_z_range_max(&mut self, max: i32) {
        self.z_max = max;
    }

    pub fn get_z_range_max(&self) -> i32 {
        self.z_max
    }

    pub fn set_layer_range_min(&mut self, min: i32) {
        self.layer_min = min;
    }

    pub fn get_layer_range_min(&self) -> i32 {
        self.layer_min
    }

    pub fn set_layer_range_max(&mut self, max: i32) {
        self.layer_max = max;
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

    /// `ERR_FAIL_INDEX(filter, SHADOW_FILTER_MAX)`.
    pub fn set_shadow_filter(&mut self, filter: i32) -> bool {
        let parsed = match filter {
            0 => ShadowFilter::None,
            1 => ShadowFilter::Pcf5,
            2 => ShadowFilter::Pcf13,
            _ => return false,
        };
        self.shadow_filter = parsed;
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

    /// `ERR_FAIL_INDEX(mode, BLEND_MODE_MIX + 1)`.
    pub fn set_blend_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => BlendMode::Add,
            1 => BlendMode::Sub,
            2 => BlendMode::Mix,
            _ => return false,
        };
        self.blend_mode = parsed;
        true
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

    pub fn set_texture_scale(&mut self, scale: f32) {
        self.texture_scale = scale;
    }

    pub fn get_texture_scale(&self) -> f32 {
        self.texture_scale
    }

    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance;
    }

    pub fn get_max_distance(&self) -> f32 {
        self.max_distance
    }
}

// -- OccluderPolygon2D ------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CullMode {
    Disabled,
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Debug)]
pub struct OccluderPolygon2DData {
    polygon: Vec<Vector2>,
    closed: bool,
    cull: CullMode,
    item_rect: Rect2,
    rect_cache_dirty: bool,
}

impl Default for OccluderPolygon2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl OccluderPolygon2DData {
    pub fn new() -> Self {
        Self {
            polygon: Vec::new(),
            closed: true,
            cull: CullMode::Disabled,
            item_rect: Rect2::default(),
            rect_cache_dirty: true,
        }
    }

    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
        self.rect_cache_dirty = true;
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
        self.rect_cache_dirty = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// The engine's enum takes any value; this port reports invalid ones.
    pub fn set_cull_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => CullMode::Disabled,
            1 => CullMode::Clockwise,
            2 => CullMode::CounterClockwise,
            _ => return false,
        };
        self.cull = parsed;
        true
    }

    pub fn get_cull_mode(&self) -> CullMode {
        self.cull
    }

    /// `_edit_get_rect`: the polygon's bounds, cached until it changes.
    pub fn get_item_rect(&mut self) -> Rect2 {
        if self.rect_cache_dirty {
            let mut rect = Rect2::default();
            for (index, point) in self.polygon.iter().enumerate() {
                if index == 0 {
                    rect = Rect2::new(*point, Vector2::ZERO);
                } else {
                    rect.expand_to(*point);
                }
            }
            self.item_rect = rect;
            self.rect_cache_dirty = false;
        }
        self.item_rect
    }
}

// -- LightOccluder2D --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct LightOccluder2DData {
    occluder: Option<Rid>,
    mask: i32,
    occluder_polygon: Option<Rid>,
    sdf_collision: bool,
}

impl Default for LightOccluder2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl LightOccluder2DData {
    pub fn new() -> Self {
        Self {
            occluder: None,
            mask: 1,
            occluder_polygon: None,
            sdf_collision: false,
        }
    }

    /// The node's occluder handle (assigned by the server on setup).
    pub fn set_occluder(&mut self, occluder: Option<Rid>) {
        self.occluder = occluder;
    }

    pub fn get_occluder(&self) -> Option<Rid> {
        self.occluder
    }

    pub fn set_occluder_polygon(&mut self, polygon: Option<Rid>) {
        self.occluder_polygon = polygon;
    }

    pub fn get_occluder_polygon(&self) -> Option<Rid> {
        self.occluder_polygon
    }

    pub fn set_occluder_light_mask(&mut self, mask: i32) {
        self.mask = mask;
    }

    pub fn get_occluder_light_mask(&self) -> i32 {
        self.mask
    }

    pub fn set_as_sdf_collision(&mut self, enabled: bool) {
        self.sdf_collision = enabled;
    }

    pub fn is_set_as_sdf_collision(&self) -> bool {
        self.sdf_collision
    }
}

// -- BackBufferCopy ---------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CopyMode {
    Disabled,
    Rect,
    Viewport,
}

#[derive(Clone, Debug)]
pub struct BackBufferCopyData {
    rect: Rect2,
    copy_mode: CopyMode,
}

impl Default for BackBufferCopyData {
    fn default() -> Self {
        Self::new()
    }
}

impl BackBufferCopyData {
    pub fn new() -> Self {
        Self {
            rect: Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0)),
            copy_mode: CopyMode::Rect,
        }
    }

    pub fn set_rect(&mut self, rect: Rect2) {
        self.rect = rect;
    }

    pub fn get_rect(&self) -> Rect2 {
        self.rect
    }

    pub fn set_copy_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => CopyMode::Disabled,
            1 => CopyMode::Rect,
            2 => CopyMode::Viewport,
            _ => return false,
        };
        self.copy_mode = parsed;
        true
    }

    pub fn get_copy_mode(&self) -> CopyMode {
        self.copy_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_2d_defaults() {
        let mut light = Light2DData::new();
        assert!(light.is_enabled());
        assert!(!light.is_editor_only());
        assert!(!light.is_shadow_enabled());
        assert_eq!(light.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(light.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(light.get_height(), 0.0);
        assert_eq!(light.get_energy(), 1.0);
        assert_eq!(light.get_z_range_min(), -1024);
        assert_eq!(light.get_z_range_max(), 1024);
        assert_eq!(light.get_layer_range_min(), 0);
        assert_eq!(light.get_layer_range_max(), 0);
        assert_eq!(light.get_item_cull_mask(), 1);
        assert_eq!(light.get_item_shadow_cull_mask(), 1);
        assert_eq!(light.get_shadow_smooth(), 0.0);
        assert!(light.get_texture().is_none());
        assert_eq!(light.get_texture_offset(), Vector2::ZERO);
        assert_eq!(light.get_texture_scale(), 1.0);
        assert_eq!(light.get_max_distance(), 10000.0);
        assert_eq!(light.get_shadow_filter(), ShadowFilter::None);
        assert_eq!(light.get_blend_mode(), BlendMode::Add);

        assert_eq!(ShadowFilter::None as i32, 0);
        assert_eq!(ShadowFilter::Pcf13 as i32, 2);
        assert_eq!(ShadowFilter::Max as i32, 3);
        assert_eq!(BlendMode::Add as i32, 0);
        assert_eq!(BlendMode::Mix as i32, 2);
    }

    #[test]
    fn light_2d_setters() {
        let mut light = Light2DData::new();
        light.set_enabled(false);
        light.set_editor_only(true);
        light.set_color(Color::new(0.5, 0.25, 0.75, 1.0));
        light.set_height(2.0);
        light.set_energy(3.0);
        light.set_z_range_min(-5);
        light.set_z_range_max(6);
        light.set_layer_range_min(1);
        light.set_layer_range_max(2);
        light.set_item_cull_mask(0b1010);
        light.set_item_shadow_cull_mask(0b0101);
        light.set_shadow_enabled(true);
        assert!(light.set_shadow_filter(2));
        assert!(!light.set_shadow_filter(3));
        light.set_shadow_color(Color::new(0.1, 0.1, 0.1, 1.0));
        light.set_shadow_smooth(0.5);
        assert!(light.set_blend_mode(1));
        assert!(!light.set_blend_mode(3));
        light.set_texture(Some(Rid::from_u64(2)));
        light.set_texture_offset(Vector2::new(1.0, 2.0));
        light.set_texture_scale(0.5);
        light.set_max_distance(100.0);

        assert!(!light.is_enabled());
        assert!(light.is_editor_only());
        assert_eq!(light.get_color(), Color::new(0.5, 0.25, 0.75, 1.0));
        assert_eq!(light.get_height(), 2.0);
        assert_eq!(light.get_energy(), 3.0);
        assert_eq!(light.get_z_range_min(), -5);
        assert_eq!(light.get_z_range_max(), 6);
        assert_eq!(light.get_layer_range_min(), 1);
        assert_eq!(light.get_layer_range_max(), 2);
        assert_eq!(light.get_item_cull_mask(), 0b1010);
        assert_eq!(light.get_item_shadow_cull_mask(), 0b0101);
        assert!(light.is_shadow_enabled());
        assert_eq!(light.get_shadow_filter(), ShadowFilter::Pcf13);
        assert_eq!(light.get_shadow_color(), Color::new(0.1, 0.1, 0.1, 1.0));
        assert_eq!(light.get_shadow_smooth(), 0.5);
        assert_eq!(light.get_blend_mode(), BlendMode::Sub);
        assert_eq!(light.get_texture(), Some(Rid::from_u64(2)));
        assert_eq!(light.get_texture_offset(), Vector2::new(1.0, 2.0));
        assert_eq!(light.get_texture_scale(), 0.5);
        assert_eq!(light.get_max_distance(), 100.0);
    }

    #[test]
    fn occluder_polygon() {
        let mut polygon = OccluderPolygon2DData::new();
        assert!(polygon.get_polygon().is_empty());
        assert!(polygon.is_closed());
        assert_eq!(polygon.get_cull_mode(), CullMode::Disabled);
        assert_eq!(polygon.get_item_rect(), Rect2::default());

        polygon.set_polygon(vec![
            Vector2::new(1.0, 2.0),
            Vector2::new(5.0, 2.0),
            Vector2::new(5.0, 6.0),
        ]);
        assert_eq!(polygon.get_polygon().len(), 3);
        let rect = polygon.get_item_rect();
        assert_eq!(rect.position, Vector2::new(1.0, 2.0));
        assert_eq!(rect.size, Vector2::new(4.0, 4.0));

        // The rect is cached and invalidated by changes.
        polygon.set_closed(false);
        assert!(!polygon.is_closed());
        assert!(polygon.set_cull_mode(2));
        assert_eq!(polygon.get_cull_mode(), CullMode::CounterClockwise);
        assert!(!polygon.set_cull_mode(3));
        assert_eq!(polygon.get_item_rect().size, Vector2::new(4.0, 4.0));

        polygon.set_polygon(Vec::new());
        assert_eq!(polygon.get_item_rect(), Rect2::default());
    }

    #[test]
    fn light_occluder_and_back_buffer() {
        let mut occluder = LightOccluder2DData::new();
        assert!(occluder.get_occluder_polygon().is_none());
        assert_eq!(occluder.get_occluder_light_mask(), 1);
        assert!(!occluder.is_set_as_sdf_collision());
        occluder.set_occluder_polygon(Some(Rid::from_u64(4)));
        occluder.set_occluder_light_mask(7);
        occluder.set_as_sdf_collision(true);
        assert_eq!(occluder.get_occluder_polygon(), Some(Rid::from_u64(4)));
        assert_eq!(occluder.get_occluder_light_mask(), 7);
        assert!(occluder.is_set_as_sdf_collision());

        let mut copy = BackBufferCopyData::new();
        assert_eq!(
            copy.get_rect(),
            Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0))
        );
        assert_eq!(copy.get_copy_mode(), CopyMode::Rect);
        assert!(copy.set_copy_mode(2));
        assert_eq!(copy.get_copy_mode(), CopyMode::Viewport);
        assert!(!copy.set_copy_mode(3));
        copy.set_rect(Rect2::new(Vector2::ZERO, Vector2::ONE));
        assert_eq!(copy.get_rect().size, Vector2::ONE);
        assert_eq!(CopyMode::Disabled as i32, 0);
        assert_eq!(CopyMode::Viewport as i32, 2);
    }
}
