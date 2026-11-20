//! `scene/main/canvas_item.*`: the data half of `CanvasItem`.
//!
//! Stored here: visibility (including the cached parent visibility the engine
//! keeps for `is_visible_in_tree`), `modulate`/`self_modulate`, `z_index` with
//! its `z_as_relative` composition, `y_sort_enabled`, `show_behind_parent`,
//! the clip-children mode, `light_mask` and `visibility_layer`.
//!
//! Not ported: the actual canvas (`RenderingServer` calls), material and
//! texture-filter state, drawing commands, and signal emission. Those need the
//! object layer; the numbers below are what the renderer consumes and what the
//! engine's own getters return.

use crate::math::color::Color;

/// `RenderingServerEnums::CanvasItemZMin` / `Max`.
pub const CANVAS_ITEM_Z_MIN: i32 = -4096;
pub const CANVAS_ITEM_Z_MAX: i32 = 4096;

/// `CanvasItem::ClipChildrenMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipChildrenMode {
    Disabled,
    Only,
    AndDraw,
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasItemData {
    visible: bool,
    /// Cached `parent_visible_in_tree`, refreshed by `propagate_visibility`.
    parent_visible_in_tree: bool,
    modulate: Color,
    self_modulate: Color,
    z_index: i32,
    z_as_relative: bool,
    y_sort_enabled: bool,
    show_behind_parent: bool,
    clip_children: ClipChildrenMode,
    light_mask: i32,
    visibility_layer: i32,
}

impl Default for CanvasItemData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItemData {
    pub fn new() -> Self {
        Self {
            visible: true,
            parent_visible_in_tree: false,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            self_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            z_index: 0,
            z_as_relative: true,
            y_sort_enabled: false,
            show_behind_parent: false,
            clip_children: ClipChildrenMode::Disabled,
            light_mask: 1,
            visibility_layer: 1,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// `_propagate_visibility_changed`: the tree pushes the parent's flag down.
    pub fn propagate_visibility(&mut self, parent_visible_in_tree: bool) {
        self.parent_visible_in_tree = parent_visible_in_tree;
    }

    /// `is_visible_in_tree`: own flag AND the cached parent flag.
    pub fn is_visible_in_tree(&self) -> bool {
        self.visible && self.parent_visible_in_tree
    }

    pub fn set_modulate(&mut self, color: Color) {
        self.modulate = color;
    }

    pub fn get_modulate(&self) -> Color {
        self.modulate
    }

    pub fn set_self_modulate(&mut self, color: Color) {
        self.self_modulate = color;
    }

    pub fn get_self_modulate(&self) -> Color {
        self.self_modulate
    }

    /// `get_modulate_in_tree`: this item's modulate multiplied by every
    /// ancestor's, nearest first (the engine walks up the tree).
    pub fn get_modulate_in_tree(&self, ancestors: &[Color]) -> Color {
        let mut final_modulate = self.modulate;
        for m in ancestors {
            final_modulate = Color::new(
                final_modulate.r * m.r,
                final_modulate.g * m.g,
                final_modulate.b * m.b,
                final_modulate.a * m.a,
            );
        }
        final_modulate
    }

    /// `set_z_index`: rejects values outside the renderer's range and keeps the
    /// previous one, like the engine's `ERR_FAIL_COND_MSG`.
    pub fn set_z_index(&mut self, z: i32) -> bool {
        if !(CANVAS_ITEM_Z_MIN..=CANVAS_ITEM_Z_MAX).contains(&z) {
            return false;
        }
        self.z_index = z;
        true
    }

    pub fn get_z_index(&self) -> i32 {
        self.z_index
    }

    pub fn set_z_as_relative(&mut self, enabled: bool) {
        self.z_as_relative = enabled;
    }

    pub fn is_z_relative(&self) -> bool {
        self.z_as_relative
    }

    /// `get_effective_z_index`: relative z adds the parent's effective z.
    pub fn get_effective_z_index(&self, parent_effective_z: i32) -> i32 {
        if self.z_as_relative {
            self.z_index + parent_effective_z
        } else {
            self.z_index
        }
    }

    pub fn set_y_sort_enabled(&mut self, enabled: bool) {
        self.y_sort_enabled = enabled;
    }

    pub fn is_y_sort_enabled(&self) -> bool {
        self.y_sort_enabled
    }

    pub fn set_draw_behind_parent(&mut self, enabled: bool) {
        self.show_behind_parent = enabled;
    }

    pub fn is_draw_behind_parent_enabled(&self) -> bool {
        self.show_behind_parent
    }

    pub fn set_clip_children_mode(&mut self, mode: ClipChildrenMode) {
        self.clip_children = mode;
    }

    pub fn get_clip_children_mode(&self) -> ClipChildrenMode {
        self.clip_children
    }

    pub fn set_light_mask(&mut self, mask: i32) {
        self.light_mask = mask;
    }

    pub fn get_light_mask(&self) -> i32 {
        self.light_mask
    }

    pub fn set_visibility_layer(&mut self, layer: i32) {
        self.visibility_layer = layer;
    }

    pub fn get_visibility_layer(&self) -> i32 {
        self.visibility_layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::new(r, g, b, a)
    }

    #[test]
    fn defaults_match_the_engine() {
        let c = CanvasItemData::new();
        assert!(c.is_visible());
        // The engine starts with parent_visible_in_tree = false until the tree
        // propagates it, so a fresh item is not yet visible in the tree.
        assert!(!c.is_visible_in_tree());
        assert_eq!(c.get_modulate(), color(1.0, 1.0, 1.0, 1.0));
        assert_eq!(c.get_z_index(), 0);
        assert!(c.is_z_relative());
        assert!(!c.is_y_sort_enabled());
        assert!(!c.is_draw_behind_parent_enabled());
        assert_eq!(c.get_clip_children_mode(), ClipChildrenMode::Disabled);
        assert_eq!(c.get_light_mask(), 1);
        assert_eq!(c.get_visibility_layer(), 1);
    }

    #[test]
    fn visibility_needs_both_flags() {
        let mut c = CanvasItemData::new();
        c.propagate_visibility(true);
        assert!(c.is_visible_in_tree());

        c.set_visible(false);
        assert!(!c.is_visible_in_tree());
        c.set_visible(true);

        c.propagate_visibility(false);
        assert!(!c.is_visible_in_tree());
    }

    #[test]
    fn z_index_rejects_out_of_range_values() {
        let mut c = CanvasItemData::new();
        assert!(c.set_z_index(4096));
        assert_eq!(c.get_z_index(), 4096);
        // Out of range: rejected, previous value kept.
        assert!(!c.set_z_index(4097));
        assert_eq!(c.get_z_index(), 4096);
        assert!(!c.set_z_index(-4097));
        assert_eq!(c.get_z_index(), 4096);
        assert!(c.set_z_index(-4096));
    }

    #[test]
    fn effective_z_composes_or_overrides() {
        let mut c = CanvasItemData::new();
        c.set_z_index(3);
        assert!(c.is_z_relative());
        assert_eq!(c.get_effective_z_index(10), 13);

        c.set_z_as_relative(false);
        assert_eq!(c.get_effective_z_index(10), 3);
        assert_eq!(c.get_effective_z_index(-10), 3);
    }

    #[test]
    fn modulate_in_tree_multiplies_ancestors() {
        let mut c = CanvasItemData::new();
        c.set_modulate(color(0.5, 1.0, 0.25, 0.8));

        // Nearest parent, then grandparent.
        let got = c.get_modulate_in_tree(&[color(1.0, 0.5, 1.0, 0.5), color(0.5, 1.0, 1.0, 1.0)]);
        assert!((got.r - 0.25).abs() < 1e-6, "{got:?}");
        assert!((got.g - 0.5).abs() < 1e-6, "{got:?}");
        assert!((got.b - 0.25).abs() < 1e-6, "{got:?}");
        assert!((got.a - 0.4).abs() < 1e-6, "{got:?}");

        // No ancestors: the item's own modulate.
        assert_eq!(c.get_modulate_in_tree(&[]), color(0.5, 1.0, 0.25, 0.8));
    }

    #[test]
    fn self_modulate_is_independent() {
        let mut c = CanvasItemData::new();
        c.set_modulate(color(0.5, 0.5, 0.5, 1.0));
        c.set_self_modulate(color(1.0, 0.0, 0.0, 1.0));
        assert_eq!(c.get_modulate(), color(0.5, 0.5, 0.5, 1.0));
        assert_eq!(c.get_self_modulate(), color(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn toggles_are_stored() {
        let mut c = CanvasItemData::new();
        c.set_y_sort_enabled(true);
        c.set_draw_behind_parent(true);
        c.set_clip_children_mode(ClipChildrenMode::AndDraw);
        c.set_light_mask(0b1011);
        c.set_visibility_layer(0b100);

        assert!(c.is_y_sort_enabled());
        assert!(c.is_draw_behind_parent_enabled());
        assert_eq!(c.get_clip_children_mode(), ClipChildrenMode::AndDraw);
        assert_eq!(c.get_light_mask(), 0b1011);
        assert_eq!(c.get_visibility_layer(), 0b100);
    }
}
