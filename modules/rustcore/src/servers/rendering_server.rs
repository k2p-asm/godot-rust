//! `servers/rendering_server.*`: the canvas-item half as stored state.
//!
//! The engine's `RenderingServer` is an abstract interface backed by Vulkan,
//! D3D12, Metal or the software renderer. This module keeps the part that is
//! *not* backend-specific: the per-item state that `canvas_item_*` calls write,
//! stored in a [`crate::servers::rid::RidOwner`] so handles stay safe across
//! frees.
//!
//! Not ported: everything that reaches a GPU (textures, meshes, materials,
//! draw commands), and the singleton/threading model.

use crate::math::color::Color;
use crate::math::transform_2d::Transform2D;
use crate::math::vector2i::Vector2i;
use crate::servers::rid::{Rid, RidOwner};

#[derive(Clone, Copy, Debug)]
pub struct CanvasItemState {
    pub transform: Transform2D,
    pub modulate: Color,
    pub self_modulate: Color,
    pub z_index: i32,
    pub visible: bool,
}

impl Default for CanvasItemState {
    fn default() -> Self {
        Self {
            transform: Transform2D::IDENTITY,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            self_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            z_index: 0,
            visible: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ViewportState {
    pub canvas_transform: Transform2D,
    pub size: Vector2i,
    pub transparent_background: bool,
    pub active: bool,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            canvas_transform: Transform2D::IDENTITY,
            size: Vector2i::ZERO,
            transparent_background: false,
            active: true,
        }
    }
}

#[derive(Default)]
pub struct RenderingServerData {
    canvas_items: RidOwner<CanvasItemState>,
    viewports: RidOwner<ViewportState>,
}

impl RenderingServerData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn canvas_item_create(&mut self) -> Rid {
        self.canvas_items.alloc(CanvasItemState::default())
    }

    pub fn canvas_item_free(&mut self, item: Rid) -> bool {
        self.canvas_items.free(item)
    }

    pub fn canvas_item_get(&self, item: Rid) -> Option<&CanvasItemState> {
        self.canvas_items.get(item)
    }

    pub fn canvas_item_set_transform(&mut self, item: Rid, transform: Transform2D) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_modulate(&mut self, item: Rid, color: Color) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.modulate = color;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_self_modulate(&mut self, item: Rid, color: Color) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.self_modulate = color;
                true
            }
            None => false,
        }
    }

    /// The server stores the raw value; the ±4096 validation lives in
    /// `CanvasItem::set_z_index`.
    pub fn canvas_item_set_z_index(&mut self, item: Rid, z_index: i32) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.z_index = z_index;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_visible(&mut self, item: Rid, visible: bool) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.visible = visible;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_count(&self) -> usize {
        self.canvas_items.count()
    }

    pub fn viewport_create(&mut self) -> Rid {
        self.viewports.alloc(ViewportState::default())
    }

    pub fn viewport_free(&mut self, viewport: Rid) -> bool {
        self.viewports.free(viewport)
    }

    pub fn viewport_get(&self, viewport: Rid) -> Option<&ViewportState> {
        self.viewports.get(viewport)
    }

    pub fn viewport_set_canvas_transform(&mut self, viewport: Rid, transform: Transform2D) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.canvas_transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_size(&mut self, viewport: Rid, size: Vector2i) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.size = size;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_transparent_background(&mut self, viewport: Rid, enabled: bool) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.transparent_background = enabled;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_active(&mut self, viewport: Rid, active: bool) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.active = active;
                true
            }
            None => false,
        }
    }

    pub fn viewport_count(&self) -> usize {
        self.viewports.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_starts_with_identity_and_white() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        let s = rs.canvas_item_get(item).expect("created item exists");
        assert_eq!(s.transform, Transform2D::IDENTITY);
        assert_eq!(s.modulate, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(s.z_index, 0);
        assert!(s.visible);
        assert_eq!(rs.canvas_item_count(), 1);
    }

    #[test]
    fn setters_write_the_state() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();

        let mut t = Transform2D::IDENTITY;
        t.columns[2] = crate::math::vector2::Vector2::new(3.0, 4.0);
        assert!(rs.canvas_item_set_transform(item, t));
        assert!(rs.canvas_item_set_modulate(item, Color::new(0.5, 0.5, 0.5, 1.0)));
        assert!(rs.canvas_item_set_self_modulate(item, Color::new(1.0, 0.0, 0.0, 1.0)));
        assert!(rs.canvas_item_set_z_index(item, -12));
        assert!(rs.canvas_item_set_visible(item, false));

        let s = rs.canvas_item_get(item).unwrap();
        assert_eq!(s.transform.columns[2], crate::math::vector2::Vector2::new(3.0, 4.0));
        assert_eq!(s.modulate, Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(s.self_modulate, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(s.z_index, -12);
        assert!(!s.visible);
    }

    #[test]
    fn freed_items_reject_writes() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        assert!(rs.canvas_item_free(item));
        assert!(rs.canvas_item_get(item).is_none());

        assert!(!rs.canvas_item_set_transform(item, Transform2D::IDENTITY));
        assert!(!rs.canvas_item_set_visible(item, true));
        assert!(!rs.canvas_item_set_z_index(item, 1));
        assert_eq!(rs.canvas_item_count(), 0);
    }

    #[test]
    fn server_stores_z_outside_the_scene_range() {
        // The scene layer validates; the server does not.
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        assert!(rs.canvas_item_set_z_index(item, 999_999));
        assert_eq!(rs.canvas_item_get(item).unwrap().z_index, 999_999);
    }

    #[test]
    fn viewport_defaults_and_setters() {
        let mut rs = RenderingServerData::new();
        let vp = rs.viewport_create();
        let v = rs.viewport_get(vp).unwrap();
        assert_eq!(v.canvas_transform, Transform2D::IDENTITY);
        assert_eq!(v.size, Vector2i::ZERO);
        assert!(!v.transparent_background);
        assert!(v.active);

        let mut t = Transform2D::IDENTITY;
        t.columns[2] = crate::math::vector2::Vector2::new(5.0, 6.0);
        assert!(rs.viewport_set_canvas_transform(vp, t));
        assert!(rs.viewport_set_size(vp, Vector2i::new(1280, 720)));
        assert!(rs.viewport_set_transparent_background(vp, true));
        assert!(rs.viewport_set_active(vp, false));

        let v = rs.viewport_get(vp).unwrap();
        assert_eq!(v.canvas_transform.columns[2], crate::math::vector2::Vector2::new(5.0, 6.0));
        assert_eq!(v.size, Vector2i::new(1280, 720));
        assert!(v.transparent_background);
        assert!(!v.active);
    }

    #[test]
    fn freed_viewports_reject_writes() {
        let mut rs = RenderingServerData::new();
        let vp = rs.viewport_create();
        assert_eq!(rs.viewport_count(), 1);
        assert!(rs.viewport_free(vp));

        assert!(!rs.viewport_set_size(vp, Vector2i::ONE));
        assert!(!rs.viewport_set_canvas_transform(vp, Transform2D::IDENTITY));
        assert!(!rs.viewport_set_active(vp, true));
        assert!(rs.viewport_get(vp).is_none());
        assert_eq!(rs.viewport_count(), 0);
        // The canvas-item owner is independent.
        assert_eq!(rs.canvas_item_count(), 0);
    }

    #[test]
    fn viewport_handles_do_not_revive_either() {
        let mut rs = RenderingServerData::new();
        let a = rs.viewport_create();
        assert!(rs.viewport_free(a));
        let b = rs.viewport_create();
        assert!(!rs.viewport_set_active(a, false));
        assert!(rs.viewport_get(b).unwrap().active);
    }
}
