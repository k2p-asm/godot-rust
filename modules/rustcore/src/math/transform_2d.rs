//! Port of `core/math/transform_2d.h` + `transform_2d.cpp`.
//!
//! A 2x3 affine transform stored as three columns, exactly like the engine:
//! `columns[0]` and `columns[1]` are the basis, `columns[2]` is the origin.
//! Every `CanvasItem` carries one of these.

use crate::math::funcs;
use crate::math::vector2::Vector2;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub columns: [Vector2; 3],
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform2D {
    pub const IDENTITY: Transform2D = Transform2D {
        columns: [
            Vector2 { x: 1.0, y: 0.0 },
            Vector2 { x: 0.0, y: 1.0 },
            Vector2 { x: 0.0, y: 0.0 },
        ],
    };

    pub const FLIP_X: Transform2D = Transform2D {
        columns: [
            Vector2 { x: -1.0, y: 0.0 },
            Vector2 { x: 0.0, y: 1.0 },
            Vector2 { x: 0.0, y: 0.0 },
        ],
    };

    pub const FLIP_Y: Transform2D = Transform2D {
        columns: [
            Vector2 { x: 1.0, y: 0.0 },
            Vector2 { x: 0.0, y: -1.0 },
            Vector2 { x: 0.0, y: 0.0 },
        ],
    };

    #[inline]
    pub const fn new(columns: [Vector2; 3]) -> Self {
        Self { columns }
    }

    /// Rotation (radians) plus translation.
    pub fn from_rotation_origin(rot: f32, pos: Vector2) -> Self {
        let cr = rot.cos();
        let sr = rot.sin();
        Self {
            columns: [
                Vector2::new(cr, sr),
                Vector2::new(-sr, cr),
                pos,
            ],
        }
    }

    /// Rotation, scale, skew and translation.
    pub fn from_rotation_scale_skew_origin(rot: f32, scale: Vector2, skew: f32, pos: Vector2) -> Self {
        Self {
            columns: [
                Vector2::new(rot.cos() * scale.x, rot.sin() * scale.x),
                Vector2::new(-(rot + skew).sin() * scale.y, (rot + skew).cos() * scale.y),
                pos,
            ],
        }
    }

    #[inline]
    pub fn tdotx(&self, v: Vector2) -> f32 {
        self.columns[0][0] * v.x + self.columns[1][0] * v.y
    }

    #[inline]
    pub fn tdoty(&self, v: Vector2) -> f32 {
        self.columns[0][1] * v.x + self.columns[1][1] * v.y
    }

    #[inline]
    pub fn basis_xform(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.tdotx(v), self.tdoty(v))
    }

    #[inline]
    pub fn basis_xform_inv(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.columns[0].dot(v), self.columns[1].dot(v))
    }

    #[inline]
    pub fn xform(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.tdotx(v), self.tdoty(v)) + self.columns[2]
    }

    #[inline]
    pub fn get_origin(&self) -> Vector2 {
        self.columns[2]
    }

    #[inline]
    pub fn set_origin(&mut self, origin: Vector2) {
        self.columns[2] = origin;
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.columns[0].x * self.columns[1].y - self.columns[0].y * self.columns[1].x
    }

    /// Transpose-inverse; assumes a pure rotation basis.
    pub fn invert(&mut self) {
        std::mem::swap(&mut self.columns[0].y, &mut self.columns[1].x);
        self.columns[2] = self.basis_xform(-self.columns[2]);
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let mut inv = *self;
        inv.invert();
        inv
    }

    pub fn affine_invert(&mut self) {
        let det = self.determinant();
        let idet = 1.0 / det;

        std::mem::swap(&mut self.columns[0].x, &mut self.columns[1].y);
        self.columns[0] *= Vector2::new(idet, -idet);
        self.columns[1] *= Vector2::new(-idet, idet);

        self.columns[2] = self.basis_xform(-self.columns[2]);
    }

    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let mut inv = *self;
        inv.affine_invert();
        inv
    }

    pub fn rotate(&mut self, angle: f32) {
        *self = Transform2D::from_rotation_origin(angle, Vector2::ZERO) * *self;
    }

    #[inline]
    pub fn rotated(&self, angle: f32) -> Self {
        Transform2D::from_rotation_origin(angle, Vector2::ZERO) * *self
    }

    #[inline]
    pub fn rotated_local(&self, angle: f32) -> Self {
        *self * Transform2D::from_rotation_origin(angle, Vector2::ZERO)
    }

    pub fn get_skew(&self) -> f32 {
        let det = self.determinant();
        funcs::acos(
            self.columns[0].normalized().dot(funcs::sign_f32(det) * self.columns[1].normalized()),
        ) - funcs::PI as f32 * 0.5
    }

    pub fn set_skew(&mut self, angle: f32) {
        let det = self.determinant();
        self.columns[1] = funcs::sign_f32(det)
            * self.columns[0].rotated(funcs::PI as f32 * 0.5 + angle).normalized()
            * self.columns[1].length();
    }

    #[inline]
    pub fn get_rotation(&self) -> f32 {
        self.columns[0].y.atan2(self.columns[0].x)
    }

    pub fn set_rotation(&mut self, rot: f32) {
        let scale = self.get_scale();
        let cr = rot.cos();
        let sr = rot.sin();
        self.columns[0].x = cr;
        self.columns[0].y = sr;
        self.columns[1].x = -sr;
        self.columns[1].y = cr;
        self.set_scale(scale);
    }

    /// `set_rotation_and_scale`: no skew, used by `CanvasLayer`.
    #[inline]
    pub fn set_rotation_and_scale(&mut self, rot: f32, scale: Vector2) {
        let (s, c) = (rot.sin(), rot.cos());
        self.columns[0] = Vector2::new(c * scale.x, s * scale.x);
        self.columns[1] = Vector2::new(-s * scale.y, c * scale.y);
    }

    /// `set_rotation_scale_and_skew`: the transform `Node2D` rebuilds whenever
    /// one of its components changes.
    #[inline]
    pub fn set_rotation_scale_and_skew(&mut self, rot: f32, scale: Vector2, skew: f32) {
        self.columns[0] = Vector2::new(rot.cos() * scale.x, rot.sin() * scale.x);
        self.columns[1] = Vector2::new(
            -(rot + skew).sin() * scale.y,
            (rot + skew).cos() * scale.y,
        );
    }

    #[inline]
    pub fn get_scale(&self) -> Vector2 {
        let det_sign = funcs::sign_f32(self.determinant());
        Vector2::new(self.columns[0].length(), det_sign * self.columns[1].length())
    }

    pub fn set_scale(&mut self, scale: Vector2) {
        self.columns[0].normalize();
        self.columns[1].normalize();
        self.columns[0] *= scale.x;
        self.columns[1] *= scale.y;
    }

    pub fn scale(&mut self, scale: Vector2) {
        self.scale_basis(scale);
        self.columns[2] *= scale;
    }

    pub fn scale_basis(&mut self, scale: Vector2) {
        self.columns[0].x *= scale.x;
        self.columns[0].y *= scale.y;
        self.columns[1].x *= scale.x;
        self.columns[1].y *= scale.y;
    }

    #[inline]
    pub fn translate_local(&mut self, translation: Vector2) {
        self.columns[2] += self.basis_xform(translation);
    }

    pub fn orthonormalize(&mut self) {
        let mut x = self.columns[0];
        let mut y = self.columns[1];

        x.normalize();
        y = y - x * x.dot(y);
        y.normalize();

        self.columns[0] = x;
        self.columns[1] = y;
    }

    #[inline]
    pub fn orthonormalized(&self) -> Self {
        let mut t = *self;
        t.orthonormalize();
        t
    }

    pub fn is_conformal(&self) -> bool {
        // Non-flipped case.
        if funcs::is_equal_approx(self.columns[0].x, self.columns[1].y)
            && funcs::is_equal_approx(self.columns[0].y, -self.columns[1].x)
        {
            return true;
        }
        // Flipped case.
        if funcs::is_equal_approx(self.columns[0].x, -self.columns[1].y)
            && funcs::is_equal_approx(self.columns[0].y, self.columns[1].x)
        {
            return true;
        }
        false
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Transform2D) -> bool {
        self.columns[0].is_equal_approx(other.columns[0])
            && self.columns[1].is_equal_approx(other.columns[1])
            && self.columns[2].is_equal_approx(other.columns[2])
    }

    #[inline]
    pub fn is_same(&self, other: &Transform2D) -> bool {
        self.columns[0].is_same(other.columns[0])
            && self.columns[1].is_same(other.columns[1])
            && self.columns[2].is_same(other.columns[2])
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.columns[0].is_finite() && self.columns[1].is_finite() && self.columns[2].is_finite()
    }

    pub fn looking_at(&self, target: Vector2) -> Self {
        let mut ret = Transform2D::from_rotation_origin(self.get_rotation(), self.get_origin());
        let target_position = self.affine_inverse().xform(target);
        ret.set_rotation(ret.get_rotation() + (target_position * self.get_scale()).angle());
        ret
    }

    pub fn scaled(&self, scale: Vector2) -> Self {
        let mut copy = *self;
        copy.scale(scale);
        copy
    }

    #[inline]
    pub fn scaled_local(&self, scale: Vector2) -> Self {
        Transform2D::new([
            self.columns[0] * scale.x,
            self.columns[1] * scale.y,
            self.columns[2],
        ])
    }

    #[inline]
    pub fn untranslated(&self) -> Self {
        Transform2D::new([self.columns[0], self.columns[1], Vector2::ZERO])
    }

    #[inline]
    pub fn translated(&self, offset: Vector2) -> Self {
        Transform2D::new([self.columns[0], self.columns[1], self.columns[2] + offset])
    }

    #[inline]
    pub fn translated_local(&self, offset: Vector2) -> Self {
        Transform2D::new([
            self.columns[0],
            self.columns[1],
            self.columns[2] + self.basis_xform(offset),
        ])
    }

    pub fn interpolate_with(&self, other: &Transform2D, weight: f32) -> Self {
        Transform2D::from_rotation_scale_skew_origin(
            funcs::lerp_angle(self.get_rotation(), other.get_rotation(), weight),
            self.get_scale().lerp(other.get_scale(), weight),
            funcs::lerp_angle(self.get_skew(), other.get_skew(), weight),
            self.get_origin().lerp(other.get_origin(), weight),
        )
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Transform2D;
    #[inline]
    fn mul(self, other: Transform2D) -> Transform2D {
        Transform2D::new([
            Vector2::new(self.tdotx(other.columns[0]), self.tdoty(other.columns[0])),
            Vector2::new(self.tdotx(other.columns[1]), self.tdoty(other.columns[1])),
            self.xform(other.columns[2]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_and_xform() {
        let t = Transform2D::IDENTITY;
        let v = Vector2::new(3.0, -4.0);
        assert!(t.xform(v).is_equal_approx(v));
        assert_eq!(t.determinant(), 1.0);
    }

    #[test]
    fn affine_inverse_roundtrip_with_scale() {
        let mut t = Transform2D::from_rotation_origin(0.7, Vector2::new(2.0, 3.0));
        t.set_scale(Vector2::new(2.0, 0.5));
        let v = Vector2::new(1.5, -0.25);
        let back = t.affine_inverse().xform(t.xform(v));
        assert!(back.is_equal_approx(v), "{back:?}");
    }

    #[test]
    fn rotation_and_skew_accessors() {
        let t = Transform2D::from_rotation_origin(1.2, Vector2::ZERO);
        assert!((t.get_rotation() - 1.2).abs() < 1e-6);
        let s = t.get_scale();
        assert!(s.is_equal_approx(Vector2::ONE), "{s:?}");
        assert!(t.get_skew().abs() < 1e-6);
    }

    #[test]
    fn composition_order() {
        let a = Transform2D::from_rotation_origin(0.0, Vector2::new(1.0, 0.0));
        let b = Transform2D::from_rotation_origin(0.0, Vector2::new(0.0, 2.0));
        let ab = a * b;
        assert!(ab.xform(Vector2::ZERO).is_equal_approx(Vector2::new(1.0, 2.0)));
    }
}
