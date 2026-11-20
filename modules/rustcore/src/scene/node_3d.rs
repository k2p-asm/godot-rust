//! `scene/3d/node_3d.*`: transform, Euler rotation and scale state.
//!
//! The engine keeps the local transform, the Euler rotation and the scale
//! separately and marks the other side dirty, exactly so that reading back a
//! rotation does not lose information through a matrix round-trip. This port
//! keeps the same two-sided model, recomputing eagerly instead of with dirty
//! bits, which preserves the observable behaviour:
//!
//! - `set_transform` / `set_basis` / `set_quaternion` make the rotation and
//!   scale derive from the matrix on the next read.
//! - `set_rotation` / `set_scale` rebuild the matrix from the Euler angles and
//!   the scale for the current `EulerOrder` (default `YXZ`).
//! - The origin is untouched by any rotation or scale change.
//!
//! Also ported: `RotationEditMode` (with the orthogonalize when leaving `Basis`
//! mode), rotation degrees, `rotate*` / `rotate_object_local`, `orthonormalize`,
//! `set_identity`, `look_at` / `look_at_from_position`, and the global transform
//! composed with a supplied parent transform (`top_level` ignores it).
//!
//! Not ported: the scene tree notifications, physics interpolation and the
//! 3D children list.

use crate::math::basis::{Basis, EulerOrder};
use crate::math::quaternion::Quaternion;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RotationEditMode {
    Euler,
    Quaternion,
    Basis,
}

#[derive(Clone, Copy, Debug)]
pub struct Node3DData {
    local_transform: Transform3D,
    /// Valid only while `rot_scale_valid`; otherwise derived from the matrix.
    euler_rotation: Vector3,
    scale: Vector3,
    rot_scale_valid: bool,
    rotation_order: EulerOrder,
    rotation_edit_mode: RotationEditMode,
    top_level: bool,
    visible: bool,
}

impl Default for Node3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Node3DData {
    pub fn new() -> Self {
        Self {
            local_transform: Transform3D::IDENTITY,
            euler_rotation: Vector3::ZERO,
            scale: Vector3::ONE,
            rot_scale_valid: true,
            rotation_order: EulerOrder::YXZ,
            rotation_edit_mode: RotationEditMode::Euler,
            top_level: false,
            visible: true,
        }
    }

    // -- dirty-model helpers ------------------------------------------------

    fn ensure_scale(&mut self) {
        if !self.rot_scale_valid {
            self.scale = self.local_transform.basis.get_scale();
        }
    }

    fn ensure_euler(&mut self) {
        if !self.rot_scale_valid {
            self.euler_rotation = self
                .local_transform
                .basis
                .get_euler_normalized(self.rotation_order);
        }
    }

    fn ensure_rot_scale(&mut self) {
        if !self.rot_scale_valid {
            self.ensure_euler();
            self.ensure_scale();
            self.rot_scale_valid = true;
        }
    }

    /// Rebuilds the basis from the Euler rotation and scale, keeping the origin.
    fn rebuild_basis(&mut self) {
        self.local_transform.basis =
            Basis::from_euler_scale(self.euler_rotation, self.scale, self.rotation_order);
        self.rot_scale_valid = true;
    }

    // -- transform ---------------------------------------------------------

    pub fn set_transform(&mut self, transform: Transform3D) {
        self.local_transform = transform;
        self.rot_scale_valid = false;
    }

    pub fn get_transform(&self) -> Transform3D {
        self.local_transform
    }

    pub fn set_basis(&mut self, basis: Basis) {
        self.set_transform(Transform3D {
            basis,
            origin: self.local_transform.origin,
        });
    }

    pub fn get_basis(&self) -> Basis {
        self.local_transform.basis
    }

    pub fn set_position(&mut self, position: Vector3) {
        self.local_transform.origin = position;
    }

    pub fn get_position(&self) -> Vector3 {
        self.local_transform.origin
    }

    pub fn set_quaternion(&mut self, quaternion: Quaternion) {
        // Only the scale is needed from the matrix, and it must survive.
        self.ensure_scale();
        let basis = Basis::from_quaternion_scale(quaternion, self.scale);
        self.local_transform.basis = basis;
        // The engine reconstructs the Euler angles here instead of leaving them
        // dirty, to avoid losing scale precision.
        self.euler_rotation = basis.get_euler_normalized(self.rotation_order);
        self.rot_scale_valid = true;
    }

    pub fn get_quaternion(&self) -> Quaternion {
        self.local_transform.basis.get_rotation_quaternion()
    }

    // -- rotation and scale -------------------------------------------------

    pub fn set_rotation(&mut self, euler_radians: Vector3) {
        self.ensure_scale();
        self.euler_rotation = euler_radians;
        self.rebuild_basis();
        self.rot_scale_valid = true;
    }

    pub fn get_rotation(&mut self) -> Vector3 {
        self.ensure_rot_scale();
        self.euler_rotation
    }

    pub fn set_rotation_degrees(&mut self, degrees: Vector3) {
        self.set_rotation(Vector3::new(
            degrees.x.to_radians(),
            degrees.y.to_radians(),
            degrees.z.to_radians(),
        ));
    }

    pub fn get_rotation_degrees(&mut self) -> Vector3 {
        let r = self.get_rotation();
        Vector3::new(r.x.to_degrees(), r.y.to_degrees(), r.z.to_degrees())
    }

    pub fn set_scale(&mut self, scale: Vector3) {
        // The Euler rotation must come from the matrix, but the scale is
        // overridden, so it is not read back.
        self.ensure_euler();
        self.scale = scale;
        self.rebuild_basis();
    }

    pub fn get_scale(&mut self) -> Vector3 {
        self.ensure_rot_scale();
        self.scale
    }

    pub fn set_rotation_order(&mut self, order: EulerOrder) {
        if self.rotation_order == order {
            return;
        }
        if self.rot_scale_valid {
            // Euler angles are authoritative: reinterpret them in the new order.
            self.rotation_order = order;
            self.rebuild_basis();
        } else {
            // The matrix is authoritative: convert the stored Euler angles so
            // the transform does not change.
            self.euler_rotation = Basis::from_euler(self.euler_rotation, self.rotation_order)
                .get_euler_normalized(order);
            self.rotation_order = order;
        }
    }

    pub fn get_rotation_order(&self) -> EulerOrder {
        self.rotation_order
    }

    pub fn set_rotation_edit_mode(&mut self, mode: RotationEditMode) {
        if self.rotation_edit_mode == mode {
            return;
        }
        // Leaving Basis mode orthogonalizes the matrix, like the engine.
        if self.rotation_edit_mode == RotationEditMode::Basis && !self.rot_scale_valid {
            let mut t = self.local_transform;
            t.orthonormalize();
            self.local_transform = t;
        }
        self.rotation_edit_mode = mode;
        if mode == RotationEditMode::Euler && !self.rot_scale_valid {
            self.ensure_rot_scale();
        }
    }

    pub fn get_rotation_edit_mode(&self) -> RotationEditMode {
        self.rotation_edit_mode
    }

    // -- rotation helpers ---------------------------------------------------

    pub fn rotate(&mut self, axis: Vector3, angle: f32) {
        let mut t = self.get_transform();
        t.basis.rotate(axis, angle);
        self.set_transform(t);
    }

    pub fn rotate_object_local(&mut self, axis: Vector3, angle: f32) {
        let mut t = self.get_transform();
        t.basis.rotate_local(axis, angle);
        self.set_transform(t);
    }

    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Vector3::new(1.0, 0.0, 0.0), angle);
    }

    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Vector3::new(0.0, 1.0, 0.0), angle);
    }

    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Vector3::new(0.0, 0.0, 1.0), angle);
    }

    pub fn orthonormalize(&mut self) {
        let mut t = self.get_transform();
        t.orthonormalize();
        self.set_transform(t);
    }

    pub fn set_identity(&mut self) {
        self.set_transform(Transform3D::IDENTITY);
    }

    /// `look_at_from_position`: aims the local -Z at `target` (or +Z with
    /// `use_model_front`) while preserving the current scale. Fails on a
    /// degenerate target or a zero `up`, like the engine's `ERR_FAIL_COND_MSG`.
    pub fn look_at_from_position(
        &mut self,
        parent_global: Option<&Transform3D>,
        position: Vector3,
        target: Vector3,
        up: Vector3,
        use_model_front: bool,
    ) -> bool {
        if position.is_equal_approx(target) || up.is_zero_approx() {
            return false;
        }
        let forward = target - position;
        let lookat_basis = Basis::looking_at(forward, up, use_model_front);
        let original_scale = self.get_scale();
        self.set_global_transform(
            parent_global,
            Transform3D {
                basis: lookat_basis,
                origin: position,
            },
        );
        self.set_scale(original_scale);
        true
    }

    /// `look_at`: keeps the current global origin as the eye position.
    pub fn look_at(
        &mut self,
        parent_global: Option<&Transform3D>,
        target: Vector3,
        up: Vector3,
        use_model_front: bool,
    ) -> bool {
        let origin = self.get_global_transform(parent_global).origin;
        self.look_at_from_position(parent_global, origin, target, up, use_model_front)
    }

    // -- global transform ---------------------------------------------------

    pub fn set_global_transform(
        &mut self,
        parent_global: Option<&Transform3D>,
        transform: Transform3D,
    ) {
        let local = match parent_global {
            Some(parent) if !self.top_level => parent.affine_inverse() * transform,
            _ => transform,
        };
        self.set_transform(local);
    }

    pub fn get_global_transform(&self, parent_global: Option<&Transform3D>) -> Transform3D {
        match parent_global {
            Some(parent) if !self.top_level => *parent * self.local_transform,
            _ => self.local_transform,
        }
    }

    pub fn get_global_rotation(&mut self, parent_global: Option<&Transform3D>) -> Vector3 {
        self.get_global_transform(parent_global)
            .basis
            .get_euler_normalized(self.rotation_order)
    }

    pub fn to_local(&self, parent_global: Option<&Transform3D>, global_point: Vector3) -> Vector3 {
        self.get_global_transform(parent_global)
            .affine_inverse()
            .xform(global_point)
    }

    pub fn to_global(&self, parent_global: Option<&Transform3D>, local_point: Vector3) -> Vector3 {
        self.get_global_transform(parent_global).xform(local_point)
    }

    // -- flags --------------------------------------------------------------

    pub fn set_top_level(&mut self, top_level: bool) {
        self.top_level = top_level;
    }

    pub fn is_top_level(&self) -> bool {
        self.top_level
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::funcs;

    fn approx(a: Vector3, b: Vector3) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn defaults() {
        let mut n = Node3DData::new();
        assert_eq!(n.get_transform(), Transform3D::IDENTITY);
        assert_eq!(n.get_rotation(), Vector3::ZERO);
        assert_eq!(n.get_scale(), Vector3::ONE);
        assert_eq!(n.get_rotation_order(), EulerOrder::YXZ);
        assert_eq!(n.get_rotation_edit_mode(), RotationEditMode::Euler);
        assert!(n.is_visible());
        assert!(!n.is_top_level());
    }

    #[test]
    fn rotation_does_not_disturb_position_or_scale() {
        let mut n = Node3DData::new();
        n.set_position(Vector3::new(1.0, 2.0, 3.0));
        n.set_scale(Vector3::new(2.0, 3.0, 4.0));

        let e = Vector3::new(0.1, 0.2, 0.3);
        n.set_rotation(e);
        assert_eq!(n.get_position(), Vector3::new(1.0, 2.0, 3.0));
        assert!(approx(n.get_rotation(), e));
        assert!(approx(n.get_scale(), Vector3::new(2.0, 3.0, 4.0)));
        assert!(n
            .get_transform()
            .basis
            .is_equal_approx(&Basis::from_euler_scale(
                e,
                Vector3::new(2.0, 3.0, 4.0),
                EulerOrder::YXZ
            )));
    }

    #[test]
    fn transform_decomposes_lazily() {
        let mut n = Node3DData::new();
        let e = Vector3::new(-0.4, 0.9, 0.25);
        let s = Vector3::new(2.0, 0.5, 3.0);
        n.set_transform(Transform3D {
            basis: Basis::from_euler_scale(e, s, EulerOrder::YXZ),
            origin: Vector3::new(5.0, 6.0, 7.0),
        });

        let got_scale = n.get_scale();
        assert!((got_scale.x - 2.0).abs() < 1e-4, "{got_scale:?}");
        assert!((got_scale.y - 0.5).abs() < 1e-4, "{got_scale:?}");
        assert!((got_scale.z - 3.0).abs() < 1e-4, "{got_scale:?}");

        // Euler comes back through the normalized decomposition.
        let got_e = n.get_rotation();
        assert!(Basis::from_euler(got_e, EulerOrder::YXZ)
            .is_equal_approx(&Basis::from_euler(e, EulerOrder::YXZ)));
        // Reading did not alter the position.
        assert_eq!(n.get_position(), Vector3::new(5.0, 6.0, 7.0));
    }

    #[test]
    fn quaternion_keeps_the_scale() {
        let mut n = Node3DData::new();
        n.set_scale(Vector3::new(2.0, 3.0, 4.0));
        let q = Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), funcs::PI as f32 / 2.0);
        n.set_quaternion(q);

        assert!(n.get_quaternion().is_equal_approx(&q));
        let s = n.get_scale();
        assert!((s.x - 2.0).abs() < 1e-4, "{s:?}");
        assert!((s.y - 3.0).abs() < 1e-4, "{s:?}");
        assert!((s.z - 4.0).abs() < 1e-4, "{s:?}");
        // Rotation reads back as 90 degrees around Y.
        let e = n.get_rotation();
        assert!((e.y - funcs::PI as f32 / 2.0).abs() < 1e-4, "{e:?}");
    }

    #[test]
    fn global_transform_composes_and_inverts() {
        let parent = Transform3D {
            basis: Basis::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), funcs::PI as f32 / 2.0),
            origin: Vector3::new(10.0, 0.0, 0.0),
        };
        let mut n = Node3DData::new();
        n.set_position(Vector3::new(0.0, 0.0, 1.0));

        let global = n.get_global_transform(Some(&parent));
        assert!(approx(global.origin, parent.xform(Vector3::new(0.0, 0.0, 1.0))));
        // Rotated 90 degrees about Y, local +Z maps to +X.
        assert!(approx(global.origin, Vector3::new(11.0, 0.0, 0.0)));

        // Round trip: setting the global transform gives the same global result.
        let target = Transform3D {
            basis: Basis::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), 0.5),
            origin: Vector3::new(-3.0, 4.0, 9.0),
        };
        n.set_global_transform(Some(&parent), target);
        let back = n.get_global_transform(Some(&parent));
        assert!(approx(back.origin, target.origin));
        assert!(back.basis.is_equal_approx(&target.basis));

        // to_local / to_global are inverses through the same parent.
        let world = Vector3::new(2.0, 5.0, -1.0);
        let local = n.to_local(Some(&parent), world);
        assert!(approx(n.to_global(Some(&parent), local), world));
    }

    #[test]
    fn top_level_ignores_the_parent() {
        let parent = Transform3D {
            basis: Basis::IDENTITY,
            origin: Vector3::new(100.0, 0.0, 0.0),
        };
        let mut n = Node3DData::new();
        n.set_top_level(true);
        n.set_position(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(n.get_global_transform(Some(&parent)).origin, Vector3::new(1.0, 2.0, 3.0));
        assert!(n.is_top_level());
    }

    #[test]
    fn look_at_points_minus_z_at_the_target() {
        let mut n = Node3DData::new();
        n.set_scale(Vector3::new(2.0, 2.0, 2.0));
        assert!(n.look_at_from_position(
            None,
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::ZERO,
            Vector3::UP,
            false
        ));

        // Default model front: +Z leaves the node, so the column is +Z, scaled.
        assert!(approx(n.get_basis().get_column(2), Vector3::new(0.0, 0.0, 2.0)));
        assert_eq!(n.get_position(), Vector3::new(0.0, 0.0, 5.0));
        // Scale preserved.
        assert!(approx(n.get_scale(), Vector3::new(2.0, 2.0, 2.0)));

        // With the model front the +Z column looks at the target instead.
        let mut m = Node3DData::new();
        assert!(m.look_at_from_position(
            None,
            Vector3::new(0.0, 0.0, 5.0),
            Vector3::ZERO,
            Vector3::UP,
            true
        ));
        assert!(approx(m.get_basis().get_column(2), Vector3::new(0.0, 0.0, -1.0)));
    }

    #[test]
    fn look_at_rejects_degenerate_input() {
        let mut n = Node3DData::new();
        assert!(!n.look_at_from_position(
            None,
            Vector3::ZERO,
            Vector3::ZERO,
            Vector3::UP,
            false
        ));
        assert!(!n.look_at_from_position(
            None,
            Vector3::ZERO,
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::ZERO,
            false
        ));
        // look_at uses the current position.
        n.set_position(Vector3::new(0.0, 0.0, 3.0));
        assert!(n.look_at(None, Vector3::ZERO, Vector3::UP, false));
    }

    #[test]
    fn rotate_helpers_and_identity() {
        let mut n = Node3DData::new();
        n.rotate_y(funcs::PI as f32 / 2.0);
        // Rotating +X by 90 degrees about +Y gives -Z.
        assert!(approx(n.get_basis().xform(Vector3::new(1.0, 0.0, 0.0)), Vector3::new(0.0, 0.0, -1.0)));

        n.set_identity();
        assert_eq!(n.get_transform(), Transform3D::IDENTITY);

        let mut m = Node3DData::new();
        m.rotate_object_local(Vector3::new(0.0, 0.0, 1.0), funcs::PI as f32 / 2.0);
        assert!(approx(m.get_basis().xform(Vector3::new(1.0, 0.0, 0.0)), Vector3::new(0.0, 1.0, 0.0)));
    }

    #[test]
    fn orthonormalize_and_basis_edit_mode() {
        let mut n = Node3DData::new();
        n.set_transform(Transform3D {
            basis: Basis {
                rows: [
                    Vector3::new(2.0, 0.0, 0.0),
                    Vector3::new(0.5, 1.0, 0.0),
                    Vector3::new(0.0, 0.0, 3.0),
                ],
            },
            origin: Vector3::ZERO,
        });
        n.orthonormalize();
        let b = n.get_basis();
        for i in 0..3 {
            assert!((b.get_column(i).length() - 1.0).abs() < 1e-4);
        }

        // Leaving Basis mode orthogonalizes an authoritative matrix.
        let mut m = Node3DData::new();
        m.set_transform(Transform3D {
            basis: Basis::from_scale(Vector3::new(2.0, 3.0, 4.0)),
            origin: Vector3::ZERO,
        });
        m.set_rotation_edit_mode(RotationEditMode::Basis);
        m.set_rotation_edit_mode(RotationEditMode::Euler);
        assert!(approx(m.get_scale(), Vector3::ONE));
        assert_eq!(m.get_rotation_edit_mode(), RotationEditMode::Euler);
    }

    #[test]
    fn rotation_order_change_keeps_euler_but_rebuilds_the_matrix() {
        let e = Vector3::new(0.3, 0.4, 0.5);
        let mut n = Node3DData::new();
        n.set_rotation(e);
        n.set_rotation_order(EulerOrder::ZYX);

        assert_eq!(n.get_rotation_order(), EulerOrder::ZYX);
        // The angles are untouched...
        assert_eq!(n.get_rotation(), e);
        // ...but they now mean something else in the new order.
        assert!(n
            .get_transform()
            .basis
            .is_equal_approx(&Basis::from_euler(e, EulerOrder::ZYX)));

        // From an authoritative matrix, changing the order must not move it.
        let mut m = Node3DData::new();
        let basis = Basis::from_euler(e, EulerOrder::YXZ);
        m.set_basis(basis);
        m.set_rotation_order(EulerOrder::ZYX);
        assert!(m.get_transform().basis.is_equal_approx(&basis));
    }

    #[test]
    fn degrees_and_visibility() {
        let mut n = Node3DData::new();
        n.set_rotation_degrees(Vector3::new(0.0, 90.0, 0.0));
        let r = n.get_rotation();
        assert!((r.y - funcs::PI as f32 / 2.0).abs() < 1e-5);
        let d = n.get_rotation_degrees();
        assert!((d.y - 90.0).abs() < 1e-4, "{d:?}");

        n.hide();
        assert!(!n.is_visible());
        n.show();
        assert!(n.is_visible());
    }
}
