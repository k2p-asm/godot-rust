//! `scene/3d/spring_bone_collision*.h`: spring-bone collider state.
//!
//! Ported state:
//!
//! - `SpringBoneCollision3D`: bone name (empty), bone index (-1), position and
//!   rotation offsets;
//! - `SpringBoneCollisionSphere3D`: radius 0.1 and the `inside` flag;
//! - `SpringBoneCollisionCapsule3D`: radius 0.1, height 0.5, the `inside` flag
//!   and the engine's mid-height mapping (`height = mid_height + 2 * radius`,
//!   negative mid-height refused);
//! - `SpringBoneCollisionPlane3D`: no extra state.
//!
//! The pose synchronisation against a `Skeleton3D` and the collision solve are
//! not ported.

use crate::math::quaternion::Quaternion;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug)]
pub struct SpringBoneCollision3DData {
    bone_name: String,
    bone: i32,
    position_offset: Vector3,
    rotation_offset: Quaternion,
}

impl Default for SpringBoneCollision3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringBoneCollision3DData {
    pub fn new() -> Self {
        Self {
            bone_name: String::new(),
            bone: -1,
            position_offset: Vector3::ZERO,
            rotation_offset: Quaternion::default(),
        }
    }

    pub fn set_bone_name(&mut self, name: &str) {
        self.bone_name = name.to_string();
    }

    pub fn get_bone_name(&self) -> &str {
        &self.bone_name
    }

    pub fn set_bone(&mut self, bone: i32) {
        self.bone = bone;
    }

    pub fn get_bone(&self) -> i32 {
        self.bone
    }

    pub fn set_position_offset(&mut self, offset: Vector3) {
        self.position_offset = offset;
    }

    pub fn get_position_offset(&self) -> Vector3 {
        self.position_offset
    }

    pub fn set_rotation_offset(&mut self, offset: Quaternion) {
        self.rotation_offset = offset;
    }

    pub fn get_rotation_offset(&self) -> Quaternion {
        self.rotation_offset
    }
}

#[derive(Clone, Debug)]
pub struct SpringBoneCollisionSphere3DData {
    base: SpringBoneCollision3DData,
    radius: f32,
    inside: bool,
}

impl Default for SpringBoneCollisionSphere3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringBoneCollisionSphere3DData {
    pub fn new() -> Self {
        Self {
            base: SpringBoneCollision3DData::new(),
            radius: 0.1,
            inside: false,
        }
    }

    pub fn base(&self) -> &SpringBoneCollision3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpringBoneCollision3DData {
        &mut self.base
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_inside(&mut self, enabled: bool) {
        self.inside = enabled;
    }

    pub fn is_inside(&self) -> bool {
        self.inside
    }
}

#[derive(Clone, Debug)]
pub struct SpringBoneCollisionCapsule3DData {
    base: SpringBoneCollision3DData,
    radius: f32,
    height: f32,
    inside: bool,
}

impl Default for SpringBoneCollisionCapsule3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringBoneCollisionCapsule3DData {
    pub fn new() -> Self {
        Self {
            base: SpringBoneCollision3DData::new(),
            radius: 0.1,
            height: 0.5,
            inside: false,
        }
    }

    pub fn base(&self) -> &SpringBoneCollision3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpringBoneCollision3DData {
        &mut self.base
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    /// `set_mid_height`: `height = mid_height + 2 * radius`; negative values are
    /// refused.
    pub fn set_mid_height(&mut self, mid_height: f32) -> bool {
        if mid_height < 0.0 {
            return false;
        }
        self.height = mid_height + self.radius * 2.0;
        true
    }

    pub fn get_mid_height(&self) -> f32 {
        self.height - self.radius * 2.0
    }

    pub fn set_inside(&mut self, enabled: bool) {
        self.inside = enabled;
    }

    pub fn is_inside(&self) -> bool {
        self.inside
    }
}

/// `SpringBoneCollisionPlane3D` has no state beyond the base collider.
#[derive(Clone, Debug, Default)]
pub struct SpringBoneCollisionPlane3DData {
    base: SpringBoneCollision3DData,
}

impl SpringBoneCollisionPlane3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base(&self) -> &SpringBoneCollision3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut SpringBoneCollision3DData {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collider_defaults_and_setters() {
        let mut base = SpringBoneCollision3DData::new();
        assert_eq!(base.get_bone_name(), "");
        assert_eq!(base.get_bone(), -1);
        assert_eq!(base.get_position_offset(), Vector3::ZERO);
        assert_eq!(base.get_rotation_offset(), Quaternion::default());

        base.set_bone_name("Bone");
        base.set_bone(3);
        base.set_position_offset(Vector3::new(0.1, 0.2, 0.3));
        let rotation = Quaternion::from_axis_angle(Vector3::UP, 0.5);
        base.set_rotation_offset(rotation);
        assert_eq!(base.get_bone_name(), "Bone");
        assert_eq!(base.get_bone(), 3);
        assert_eq!(base.get_position_offset(), Vector3::new(0.1, 0.2, 0.3));
        assert_eq!(base.get_rotation_offset(), rotation);
    }

    #[test]
    fn sphere_state() {
        let mut sphere = SpringBoneCollisionSphere3DData::new();
        assert_eq!(sphere.get_radius(), 0.1);
        assert!(!sphere.is_inside());
        assert_eq!(sphere.base().get_bone(), -1);
        sphere.set_radius(0.35);
        sphere.set_inside(true);
        sphere.base_mut().set_bone(2);
        assert_eq!(sphere.get_radius(), 0.35);
        assert!(sphere.is_inside());
        assert_eq!(sphere.base().get_bone(), 2);
    }

    #[test]
    fn capsule_mid_height_mapping() {
        let mut capsule = SpringBoneCollisionCapsule3DData::new();
        assert_eq!(capsule.get_radius(), 0.1);
        assert_eq!(capsule.get_height(), 0.5);
        assert!(!capsule.is_inside());
        assert!((capsule.get_mid_height() - 0.3).abs() < 1e-6);

        // Negative mid-heights are refused; valid ones add twice the radius.
        assert!(!capsule.set_mid_height(-0.1));
        assert_eq!(capsule.get_height(), 0.5);
        assert!(capsule.set_mid_height(0.8));
        assert!((capsule.get_height() - 1.0).abs() < 1e-6);
        assert!((capsule.get_mid_height() - 0.8).abs() < 1e-6);

        capsule.set_radius(0.2);
        assert!((capsule.get_mid_height() - 0.6).abs() < 1e-6);
        capsule.set_height(1.0);
        assert!((capsule.get_mid_height() - 0.6).abs() < 1e-6);
        capsule.set_inside(true);
        assert!(capsule.is_inside());
    }

    #[test]
    fn plane_collider_has_only_the_base() {
        let mut plane = SpringBoneCollisionPlane3DData::new();
        assert_eq!(plane.base().get_bone_name(), "");
        plane.base_mut().set_bone_name("Root");
        assert_eq!(plane.base().get_bone_name(), "Root");
        assert_eq!(plane.base().get_bone(), -1);
    }
}
