//! `scene/main/canvas_layer.*`: the data half of `CanvasLayer`.
//!
//! A canvas layer owns a `layer` index, a `Transform2D`, and the
//! offset/rotation/scale triple that rebuilds it. Like `Node2D`, the engine
//! keeps both representations and decomposes lazily after `set_transform`.
//!
//! Not ported: the viewport it renders into, the follow-viewport notification
//! path, and the `RenderingServer` calls. `get_final_transform` takes the
//! viewport's canvas transform as an argument instead of fetching it.

use crate::math::transform_2d::Transform2D;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct CanvasLayerData {
    layer: i32,
    transform: Transform2D,
    offset: Vector2,
    rotation: f32,
    scale: Vector2,
    locrotscale_dirty: bool,
    visible: bool,
    follow_viewport: bool,
    follow_viewport_scale: f32,
}

impl Default for CanvasLayerData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasLayerData {
    pub fn new() -> Self {
        Self {
            layer: 1,
            transform: Transform2D::IDENTITY,
            offset: Vector2::ZERO,
            rotation: 0.0,
            scale: Vector2::ONE,
            locrotscale_dirty: false,
            visible: true,
            follow_viewport: false,
            follow_viewport_scale: 1.0,
        }
    }

    fn update_xform(&mut self) {
        self.transform.set_rotation_and_scale(self.rotation, self.scale);
        self.transform.set_origin(self.offset);
    }

    fn update_locrotscale(&mut self) {
        self.offset = self.transform.columns[2];
        self.rotation = self.transform.get_rotation();
        self.scale = self.transform.get_scale();
        self.locrotscale_dirty = false;
    }

    fn ensure_values(&mut self) {
        if self.locrotscale_dirty {
            self.update_locrotscale();
        }
    }

    pub fn set_layer(&mut self, layer: i32) {
        self.layer = layer;
    }

    pub fn get_layer(&self) -> i32 {
        self.layer
    }

    pub fn set_transform(&mut self, transform: Transform2D) {
        self.transform = transform;
        self.locrotscale_dirty = true;
    }

    pub fn get_transform(&self) -> Transform2D {
        self.transform
    }

    /// `_update_locrotscale` is lazy in the engine; the getters expose it.
    pub fn set_offset(&mut self, offset: Vector2) {
        self.ensure_values();
        self.offset = offset;
        self.update_xform();
    }

    pub fn get_offset(&mut self) -> Vector2 {
        self.ensure_values();
        self.offset
    }

    pub fn set_rotation(&mut self, radians: f32) {
        self.ensure_values();
        self.rotation = radians;
        self.update_xform();
    }

    pub fn get_rotation(&mut self) -> f32 {
        self.ensure_values();
        self.rotation
    }

    pub fn set_scale(&mut self, scale: Vector2) {
        self.ensure_values();
        self.scale = scale;
        self.update_xform();
    }

    pub fn get_scale(&mut self) -> Vector2 {
        self.ensure_values();
        self.scale
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_follow_viewport(&mut self, enable: bool) {
        self.follow_viewport = enable;
    }

    pub fn is_following_viewport(&self) -> bool {
        self.follow_viewport
    }

    pub fn set_follow_viewport_scale(&mut self, ratio: f32) {
        self.follow_viewport_scale = ratio;
    }

    pub fn get_follow_viewport_scale(&self) -> f32 {
        self.follow_viewport_scale
    }

    /// `get_final_transform`: when following, the viewport transform is applied
    /// first, scaled by `follow_viewport_scale`, then the layer transform.
    pub fn get_final_transform(&self, viewport_canvas_transform: Option<&Transform2D>) -> Transform2D {
        if self.follow_viewport {
            let mut follow = Transform2D::IDENTITY;
            follow.set_scale(Vector2::new(self.follow_viewport_scale, self.follow_viewport_scale));
            if let Some(vp) = viewport_canvas_transform {
                follow = *vp * follow;
            }
            follow * self.transform
        } else {
            self.transform
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_engine() {
        let c = CanvasLayerData::new();
        assert_eq!(c.get_layer(), 1);
        assert_eq!(c.get_transform(), Transform2D::IDENTITY);
        assert!(c.is_visible());
        assert!(!c.is_following_viewport());
        assert!((c.get_follow_viewport_scale() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn offset_rotation_scale_rebuild_the_transform() {
        let mut c = CanvasLayerData::new();
        c.set_offset(Vector2::new(10.0, -5.0));
        c.set_rotation(std::f32::consts::FRAC_PI_2);
        c.set_scale(Vector2::new(2.0, 3.0));

        let t = c.get_transform();
        // columns[0] = (cos * sx, sin * sx)
        assert!((t.columns[0] - Vector2::new(0.0, 2.0)).length() < 1e-5);
        // columns[1] = (-sin * sy, cos * sy)
        assert!((t.columns[1] - Vector2::new(-3.0, 0.0)).length() < 1e-5);
        assert_eq!(t.columns[2], Vector2::new(10.0, -5.0));
    }

    #[test]
    fn set_transform_decomposes_lazily() {
        let mut c = CanvasLayerData::new();
        let mut t = Transform2D::IDENTITY;
        t.set_rotation_and_scale(0.6, Vector2::new(4.0, 2.0));
        t.set_origin(Vector2::new(-3.0, 1.0));
        c.set_transform(t);

        assert!((c.get_rotation() - 0.6).abs() < 1e-5);
        assert!((c.get_scale() - Vector2::new(4.0, 2.0)).length() < 1e-5);
        assert_eq!(c.get_offset(), Vector2::new(-3.0, 1.0));
        // Rebuilding from the decomposed values reproduces the matrix.
        let rebuilt = c.get_transform();
        for i in 0..3 {
            assert!((rebuilt.columns[i] - t.columns[i]).length() < 1e-5);
        }
    }

    #[test]
    fn follow_viewport_composes_the_canvas_transform() {
        let mut c = CanvasLayerData::new();
        c.set_offset(Vector2::new(5.0, 0.0));

        // Not following: the layer transform is final.
        assert_eq!(c.get_final_transform(None), c.get_transform());

        c.set_follow_viewport(true);
        c.set_follow_viewport_scale(2.0);

        let mut vp = Transform2D::IDENTITY;
        vp.set_origin(Vector2::new(100.0, 0.0));
        let final_t = c.get_final_transform(Some(&vp));

        // follow = vp * scale(2); final = follow * transform.
        // The layer offset (5,0) is scaled to (10,0) and then translated.
        let origin = final_t.columns[2];
        assert!((origin.x - 110.0).abs() < 1e-5, "{origin:?}");
        assert!(origin.y.abs() < 1e-5, "{origin:?}");
    }

    #[test]
    fn layer_and_visibility_are_plain_state() {
        let mut c = CanvasLayerData::new();
        c.set_layer(-3);
        c.set_visible(false);
        assert_eq!(c.get_layer(), -3);
        assert!(!c.is_visible());
    }
}
