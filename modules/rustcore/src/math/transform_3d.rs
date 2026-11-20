//! Port of `core/math/transform_3d.h` + `transform_3d.cpp`.
//!
//! `Transform3D` is an affine transform: a `Basis` (rotation/scale) plus an
//! `origin`. This is the type every `Node3D` carries, so it is on the hot path
//! of scene processing.
//!
//! Pending: `xform(Plane)`, `xform_fast`/`xform_inv_fast` (need a precomputed
//! inverse-transpose) and `xform(Vector<Vector3>)` variants.

use crate::math::aabb::Aabb;
use crate::math::basis::Basis;
use crate::math::plane::Plane;
use crate::math::quaternion::Quaternion;
use crate::math::vector3::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3D {
    pub basis: Basis,
    pub origin: Vector3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform3D {
    pub const IDENTITY: Transform3D = Transform3D {
        basis: Basis::IDENTITY,
        origin: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
    };

    #[inline]
    pub const fn new(basis: Basis, origin: Vector3) -> Self {
        Self { basis, origin }
    }

    #[inline]
    pub fn from_columns(x: Vector3, y: Vector3, z: Vector3, origin: Vector3) -> Self {
        Self { basis: Basis::from_columns(x, y, z), origin }
    }

    #[inline]
    pub const fn from_origin(origin: Vector3) -> Self {
        Self { basis: Basis::IDENTITY, origin }
    }

    /// Transposes the basis and recomputes the origin. Assumes a pure rotation.
    pub fn invert(&mut self) {
        self.basis.transpose();
        self.origin = self.basis.xform(-self.origin);
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let mut ret = *self;
        ret.invert();
        ret
    }

    /// Full inverse, valid with scaling.
    pub fn affine_invert(&mut self) {
        self.basis.invert();
        self.origin = self.basis.xform(-self.origin);
    }

    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let mut ret = *self;
        ret.affine_invert();
        ret
    }

    /// Left multiplication by a rotation: `T -> R * T`.
    pub fn rotated(&self, axis: Vector3, angle: f32) -> Self {
        let r = Basis::from_axis_angle(axis, angle);
        Self { basis: r * self.basis, origin: r.xform(self.origin) }
    }

    #[inline]
    pub fn rotate(&mut self, axis: Vector3, angle: f32) {
        *self = self.rotated(axis, angle);
    }

    /// Right multiplication by a rotation: `T -> T * R`.
    pub fn rotated_local(&self, axis: Vector3, angle: f32) -> Self {
        let r = Basis::from_axis_angle(axis, angle);
        Self { basis: self.basis * r, origin: self.origin }
    }

    #[inline]
    pub fn rotate_basis(&mut self, axis: Vector3, angle: f32) {
        self.basis.rotate(axis, angle);
    }

    /// Replaces the basis with a look-at rotation; keeps the origin.
    pub fn looking_at(&self, target: Vector3, up: Vector3, use_model_front: bool) -> Self {
        Self {
            basis: Basis::looking_at(target - self.origin, up, use_model_front),
            origin: self.origin,
        }
    }

    pub fn set_look_at(&mut self, eye: Vector3, target: Vector3, up: Vector3, use_model_front: bool) {
        self.basis = Basis::looking_at(target - eye, up, use_model_front);
        self.origin = eye;
    }

    /// `T -> S * T`.
    pub fn scale(&mut self, scale: Vector3) {
        self.basis.scale(scale);
        self.origin *= scale;
    }

    #[inline]
    pub fn scaled(&self, scale: Vector3) -> Self {
        Self { basis: self.basis.scaled(scale), origin: self.origin * scale }
    }

    /// `T -> T * S`.
    #[inline]
    pub fn scaled_local(&self, scale: Vector3) -> Self {
        Self { basis: self.basis.scaled_local(scale), origin: self.origin }
    }

    #[inline]
    pub fn scale_basis(&mut self, scale: Vector3) {
        self.basis.scale(scale);
    }

    pub fn translate_local(&mut self, translation: Vector3) {
        for i in 0..3 {
            self.origin[i] += self.basis.rows[i].dot(translation);
        }
    }

    #[inline]
    pub fn translated(&self, translation: Vector3) -> Self {
        Self { basis: self.basis, origin: self.origin + translation }
    }

    #[inline]
    pub fn translated_local(&self, translation: Vector3) -> Self {
        Self { basis: self.basis, origin: self.origin + self.basis.xform(translation) }
    }

    #[inline]
    pub fn orthonormalize(&mut self) {
        self.basis.orthonormalize();
    }

    #[inline]
    pub fn orthonormalized(&self) -> Self {
        let mut c = *self;
        c.orthonormalize();
        c
    }

    #[inline]
    pub fn orthogonalize(&mut self) {
        self.basis.orthogonalize();
    }

    #[inline]
    pub fn orthogonalized(&self) -> Self {
        let mut c = *self;
        c.orthogonalize();
        c
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Transform3D) -> bool {
        self.basis.is_equal_approx(&other.basis) && self.origin.is_equal_approx(other.origin)
    }

    #[inline]
    pub fn is_same(&self, other: &Transform3D) -> bool {
        self.basis.is_same(&other.basis) && self.origin.is_same(other.origin)
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.basis.is_finite() && self.origin.is_finite()
    }

    /// Transforms a point: `basis * v + origin`, row by row.
    #[inline]
    pub fn xform(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            self.basis.rows[0].dot(v) + self.origin.x,
            self.basis.rows[1].dot(v) + self.origin.y,
            self.basis.rows[2].dot(v) + self.origin.z,
        )
    }

    /// Inverse transform using the transposed basis (only exact for rotations).
    #[inline]
    pub fn xform_inv(&self, v: Vector3) -> Vector3 {
        let v = v - self.origin;
        Vector3::new(
            (self.basis.rows[0][0] * v.x) + (self.basis.rows[1][0] * v.y) + (self.basis.rows[2][0] * v.z),
            (self.basis.rows[0][1] * v.x) + (self.basis.rows[1][1] * v.y) + (self.basis.rows[2][1] * v.z),
            (self.basis.rows[0][2] * v.x) + (self.basis.rows[1][2] * v.y) + (self.basis.rows[2][2] * v.z),
        )
    }

    /// Batch transform (`Transform3D::xform(const Vector<Vector3> &)`); the
    /// engine returns a new array with the same order.
    pub fn xform_array(&self, points: &[Vector3]) -> Vec<Vector3> {
        points.iter().map(|p| self.xform(*p)).collect()
    }

    /// Batch inverse transform (`Transform3D::xform_inv(const Vector<Vector3> &)`).
    pub fn xform_array_inv(&self, points: &[Vector3]) -> Vec<Vector3> {
        points.iter().map(|p| self.xform_inv(*p)).collect()
    }
    /// Bounding box of the transformed box (theomader's method).
    pub fn xform_aabb(&self, aabb: &Aabb) -> Aabb {
        let min = aabb.position;
        let max = aabb.position + aabb.size;
        let mut tmin = Vector3::ZERO;
        let mut tmax = Vector3::ZERO;

        for i in 0..3 {
            tmin[i] = self.origin[i];
            tmax[i] = self.origin[i];
            for j in 0..3 {
                let e = self.basis.rows[i][j] * min[j];
                let f = self.basis.rows[i][j] * max[j];
                if e < f {
                    tmin[i] += e;
                    tmax[i] += f;
                } else {
                    tmin[i] += f;
                    tmax[i] += e;
                }
            }
        }

        Aabb::new(tmin, tmax - tmin)
    }

    pub fn xform_inv_aabb(&self, aabb: &Aabb) -> Aabb {
        let p = aabb.position;
        let s = aabb.size;
        let vertices = [
            Vector3::new(p.x + s.x, p.y + s.y, p.z + s.z),
            Vector3::new(p.x + s.x, p.y + s.y, p.z),
            Vector3::new(p.x + s.x, p.y, p.z + s.z),
            Vector3::new(p.x + s.x, p.y, p.z),
            Vector3::new(p.x, p.y + s.y, p.z + s.z),
            Vector3::new(p.x, p.y + s.y, p.z),
            Vector3::new(p.x, p.y, p.z + s.z),
            Vector3::new(p.x, p.y, p.z),
        ];

        let mut ret = Aabb::new(self.xform_inv(vertices[0]), Vector3::ZERO);
        for v in vertices.iter().skip(1) {
            ret.expand_to(self.xform_inv(*v));
        }
        ret
    }

    /// Transforms a plane by the inverse-transpose of the basis, which is
    /// correct even with non-uniform scaling.
    pub fn xform_plane(&self, plane: &Plane) -> Plane {
        let mut inv_transpose = self.basis.inverse();
        inv_transpose.transpose();
        self.xform_plane_fast(plane, &inv_transpose)
    }

    pub fn xform_plane_fast(&self, plane: &Plane, basis_inverse_transpose: &Basis) -> Plane {
        let point = self.xform(plane.normal * plane.d);
        let mut normal = basis_inverse_transpose.xform(plane.normal);
        normal.normalize();
        Plane { normal, d: normal.dot(point) }
    }

    pub fn xform_inv_plane(&self, plane: &Plane) -> Plane {
        let inv = self.affine_inverse();
        let basis_transpose = self.basis.transposed();
        let point = inv.xform(plane.normal * plane.d);
        let mut normal = basis_transpose.xform(plane.normal);
        normal.normalize();
        Plane { normal, d: normal.dot(point) }
    }
    /// Interpolates translation linearly and rotation spherically, scales linearly.
    pub fn interpolate_with(&self, other: &Transform3D, c: f32) -> Self {
        let src_scale = self.basis.get_scale();
        let src_rot = self.basis.get_rotation_quaternion();
        let src_loc = self.origin;

        let dst_scale = other.basis.get_scale();
        let dst_rot = other.basis.get_rotation_quaternion();
        let dst_loc = other.origin;

        let rot: Quaternion = src_rot.slerp(&dst_rot, c).normalized();
        let mut basis = Basis::IDENTITY;
        basis.set_quaternion(rot);
        basis.scale_local(src_scale.lerp(dst_scale, c));

        Self { basis, origin: src_loc.lerp(dst_loc, c) }
    }
}

impl std::ops::Mul for Transform3D {
    type Output = Transform3D;
    #[inline]
    fn mul(self, other: Transform3D) -> Transform3D {
        Transform3D {
            origin: self.xform(other.origin),
            basis: self.basis * other.basis,
        }
    }
}

impl std::ops::Mul<f32> for Transform3D {
    type Output = Transform3D;
    #[inline]
    fn mul(self, v: f32) -> Transform3D {
        Transform3D { basis: self.basis * v, origin: self.origin * v }
    }
}

impl std::ops::Div<f32> for Transform3D {
    type Output = Transform3D;
    #[inline]
    fn div(self, v: f32) -> Transform3D {
        Transform3D { basis: self.basis / v, origin: self.origin / v }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::basis::EulerOrder;

    #[test]
    fn xform_and_inverse() {
        let t = Transform3D::new(
            Basis::from_euler(Vector3::new(0.2, 0.6, -0.4), EulerOrder::YXZ),
            Vector3::new(1.0, -2.0, 3.0),
        );
        let v = Vector3::new(0.5, 1.5, -0.5);
        let world = t.xform(v);
        let back = t.affine_inverse().xform(world);
        assert!(back.is_equal_approx(v), "{back:?}");
        // xform_inv is the transpose-based inverse, exact for rotations.
        assert!(t.xform_inv(world).is_equal_approx(v));
    }

    #[test]
    fn composition_order() {
        let a = Transform3D::from_origin(Vector3::new(1.0, 0.0, 0.0));
        let b = Transform3D::from_origin(Vector3::new(0.0, 2.0, 0.0));
        let ab = a * b;
        // a * b applies b first, then a.
        assert_eq!(ab.xform(Vector3::ZERO), Vector3::new(1.0, 2.0, 0.0));
    }

    #[test]
    fn local_translation_uses_basis() {
        let mut t = Transform3D::new(
            Basis::from_axis_angle(Vector3::UP, std::f32::consts::FRAC_PI_2),
            Vector3::ZERO,
        );
        t.translate_local(Vector3::RIGHT);
        // +X in local space becomes -Z in world space after a +90 deg Y rotation.
        assert!(t.origin.is_equal_approx(Vector3::new(0.0, 0.0, -1.0)), "{:?}", t.origin);
    }

    #[test]
    fn aabb_xform_is_tight() {
        let t = Transform3D::new(
            Basis::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_4),
            Vector3::ZERO,
        );
        let box_ = Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::splat(2.0));
        let tb = t.xform_aabb(&box_);
        // A 45 degree rotated unit cube spans sqrt(2) on X and Z.
        assert!((tb.size.x - 2.0 * 2f32.sqrt()).abs() < 1e-4, "{:?}", tb.size);
        assert!((tb.size.y - 2.0).abs() < 1e-4, "{:?}", tb.size);
    }

    #[test]
    fn interpolate_endpoints() {
        let a = Transform3D::from_origin(Vector3::ZERO);
        let b = Transform3D::new(
            Basis::from_axis_angle(Vector3::UP, 1.0),
            Vector3::new(0.0, 10.0, 0.0),
        );
        assert!(a.interpolate_with(&b, 0.0).is_equal_approx(&a));
        assert!(a.interpolate_with(&b, 1.0).is_equal_approx(&b));
    }
}
