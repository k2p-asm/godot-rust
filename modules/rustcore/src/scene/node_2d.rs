//! `scene/2d/node_2d.*`: the data half of `Node2D`.
//!
//! `Node2D` is a `CanvasItem` whose transform is exposed as position, rotation,
//! scale and skew. The engine keeps both representations: the setters rebuild
//! `transform` from the four values, and `set_transform` stores the matrix and
//! marks the values dirty so they are decomposed on the next read. That lazy
//! path is reproduced here, including the zero-scale clamp.
//!
//! Not ported: parenting (`get_global_transform` needs the `CanvasItem` tree),
//! notification, and the `RenderingServer` calls. Nothing here touches objects,
//! so it stays POD and testable on its own.

use crate::math::funcs;
use crate::math::transform_2d::Transform2D;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct Node2D {
    transform: Transform2D,
    position: Vector2,
    rotation: f32,
    skew: f32,
    scale: Vector2,
    xform_dirty: bool,
}

impl Default for Node2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Node2D {
    pub fn new() -> Self {
        Self {
            transform: Transform2D::IDENTITY,
            position: Vector2::ZERO,
            rotation: 0.0,
            skew: 0.0,
            scale: Vector2::ONE,
            xform_dirty: false,
        }
    }

    fn update_transform(&mut self) {
        self.transform
            .set_rotation_scale_and_skew(self.rotation, self.scale, self.skew);
        self.transform.columns[2] = self.position;
    }

    fn update_xform_values(&mut self) {
        self.rotation = self.transform.get_rotation();
        self.skew = self.transform.get_skew();
        self.position = self.transform.columns[2];
        self.scale = self.transform.get_scale();
        self.xform_dirty = false;
    }

    fn ensure_values(&mut self) {
        if self.xform_dirty {
            self.update_xform_values();
        }
    }

    pub fn set_position(&mut self, pos: Vector2) {
        self.ensure_values();
        self.position = pos;
        self.update_transform();
    }

    pub fn get_position(&mut self) -> Vector2 {
        self.ensure_values();
        self.position
    }

    pub fn set_rotation(&mut self, radians: f32) {
        self.ensure_values();
        self.rotation = radians;
        self.update_transform();
    }

    pub fn get_rotation(&mut self) -> f32 {
        self.ensure_values();
        self.rotation
    }

    pub fn set_rotation_degrees(&mut self, degrees: f32) {
        self.set_rotation(funcs::deg_to_rad(degrees as f64) as f32);
    }

    pub fn get_rotation_degrees(&mut self) -> f32 {
        funcs::rad_to_deg(self.get_rotation() as f64) as f32
    }

    pub fn set_skew(&mut self, radians: f32) {
        self.ensure_values();
        self.skew = radians;
        self.update_transform();
    }

    pub fn get_skew(&mut self) -> f32 {
        self.ensure_values();
        self.skew
    }

    pub fn set_scale(&mut self, mut scale: Vector2) {
        self.ensure_values();
        // The engine refuses zero scale: it breaks physics and rendering.
        if funcs::is_zero_approx(scale.x) {
            scale.x = funcs::CMP_EPSILON as f32;
        }
        if funcs::is_zero_approx(scale.y) {
            scale.y = funcs::CMP_EPSILON as f32;
        }
        self.scale = scale;
        self.update_transform();
    }

    pub fn get_scale(&mut self) -> Vector2 {
        self.ensure_values();
        self.scale
    }

    /// Stores a matrix directly; position/rotation/scale/skew are recomputed
    /// lazily on the next getter.
    pub fn set_transform(&mut self, transform: Transform2D) {
        self.transform = transform;
        self.xform_dirty = true;
    }

    pub fn get_transform(&self) -> Transform2D {
        self.transform
    }

    pub fn rotate(&mut self, radians: f32) {
        let current = self.get_rotation();
        self.set_rotation(current + radians);
    }

    pub fn translate(&mut self, amount: Vector2) {
        let current = self.get_position();
        self.set_position(current + amount);
    }

    /// `get_relative_transform_to_parent` for a known parent transform.
    pub fn get_transform_relative_to(&self, parent: &Transform2D) -> Transform2D {
        parent.affine_inverse() * self.transform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: Vector2, b: Vector2) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn position_only() {
        let mut n = Node2D::new();
        n.set_position(Vector2::new(3.0, -4.0));
        assert_eq!(n.get_position(), Vector2::new(3.0, -4.0));
        assert!(approx(n.get_transform().columns[2], Vector2::new(3.0, -4.0)));
        // Identity basis.
        assert!(approx(n.get_transform().columns[0], Vector2::RIGHT));
        assert!(approx(n.get_transform().columns[1], Vector2::DOWN));
    }

    #[test]
    fn rotation_turns_the_x_axis() {
        let mut n = Node2D::new();
        n.set_rotation(std::f32::consts::FRAC_PI_2);
        assert!(approx(n.get_transform().columns[0], Vector2::DOWN));
        assert!(approx(n.get_transform().columns[1], Vector2::LEFT));
        assert!((n.get_rotation() - std::f32::consts::FRAC_PI_2).abs() < 1e-6);

        n.set_rotation_degrees(180.0);
        assert!(approx(n.get_transform().columns[0], Vector2::LEFT));
        assert!((n.get_rotation_degrees() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn scale_and_skew_follow_the_engine_formula() {
        let mut n = Node2D::new();
        n.set_scale(Vector2::new(2.0, 3.0));
        n.set_rotation(0.0);
        n.set_skew(0.0);
        // columns[0] = (cos r * sx, sin r * sx)
        assert!(approx(n.get_transform().columns[0], Vector2::new(2.0, 0.0)));
        // columns[1] = (-sin(r+skew) * sy, cos(r+skew) * sy)
        assert!(approx(n.get_transform().columns[1], Vector2::new(0.0, 3.0)));

        n.set_skew(std::f32::consts::FRAC_PI_2);
        let c1 = n.get_transform().columns[1];
        assert!((c1.x + 3.0).abs() < 1e-5, "{c1:?}");
        assert!(c1.y.abs() < 1e-5, "{c1:?}");
    }

    #[test]
    fn zero_scale_is_clamped_not_accepted() {
        let mut n = Node2D::new();
        n.set_scale(Vector2::new(0.0, -0.0));
        let s = n.get_scale();
        assert!((s.x - funcs::CMP_EPSILON as f32).abs() < 1e-9);
        assert!((s.y - funcs::CMP_EPSILON as f32).abs() < 1e-9);
    }

    #[test]
    fn set_transform_is_decomposed_lazily() {
        let mut n = Node2D::new();
        let mut t = Transform2D::IDENTITY;
        t.set_rotation_scale_and_skew(0.5, Vector2::new(2.0, 4.0), 0.25);
        t.columns[2] = Vector2::new(7.0, -1.0);
        n.set_transform(t);

        // Getters trigger the same decomposition the engine does.
        assert!((n.get_rotation() - 0.5).abs() < 1e-5);
        assert!((n.get_skew() - 0.25).abs() < 1e-5);
        assert!((n.get_scale() - Vector2::new(2.0, 4.0)).length() < 1e-5);
        assert_eq!(n.get_position(), Vector2::new(7.0, -1.0));

        // Rebuilding from the decomposed values reproduces the matrix.
        let rebuilt = n.get_transform();
        for i in 0..3 {
            assert!((rebuilt.columns[i] - t.columns[i]).length() < 1e-5, "column {i}");
        }
    }

    #[test]
    fn translate_and_rotate_are_relative() {
        let mut n = Node2D::new();
        n.set_position(Vector2::new(1.0, 2.0));
        n.translate(Vector2::new(-3.0, 0.5));
        assert_eq!(n.get_position(), Vector2::new(-2.0, 2.5));

        n.set_rotation(0.4);
        n.rotate(0.6);
        assert!((n.get_rotation() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn relative_transform_undoes_the_parent() {
        let mut parent = Transform2D::IDENTITY;
        parent.set_rotation_scale_and_skew(0.7, Vector2::new(2.0, 2.0), 0.0);
        parent.columns[2] = Vector2::new(10.0, 5.0);

        let mut child = Node2D::new();
        child.set_position(Vector2::new(1.0, 0.0));
        child.set_rotation(0.2);

        // local = parent^-1 * global, so applying parent recovers the global.
        let local = child.get_transform_relative_to(&parent);
        let global = parent * local;
        let child_global = child.get_transform();
        for i in 0..3 {
            assert!((global.columns[i] - child_global.columns[i]).length() < 1e-4);
        }
    }
}
