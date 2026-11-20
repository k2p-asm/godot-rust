//! `scene/3d/skeleton_3d.*`: bone hierarchy, rest and pose state.
//!
//! Bones are `(name, parent, rest, pose position/rotation/scale, enabled)`.
//! `add_bone` enforces the engine's name rules (non-empty, no `:` or `/`, no
//! duplicates), `set_bone_parent` rejects self-parenting, and global poses/rests
//! are composed up the chain with a cycle guard.
//!
//! Not ported: dirty caches/process order, skin bindings, modifiers, physical
//! bones, pose overrides and the RenderingServer update.

use std::collections::HashMap;

use crate::math::basis::Basis;
use crate::math::quaternion::Quaternion;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug)]
pub struct SkeletonBone {
    pub name: String,
    pub parent: i32,
    pub rest: Transform3D,
    pub pose_position: Vector3,
    pub pose_rotation: Quaternion,
    pub pose_scale: Vector3,
    pub enabled: bool,
}

impl SkeletonBone {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: -1,
            rest: Transform3D::IDENTITY,
            pose_position: Vector3::ZERO,
            pose_rotation: Quaternion::IDENTITY,
            pose_scale: Vector3::ONE,
            enabled: true,
        }
    }

    /// Local pose from position/rotation/scale (`R * S`).
    pub fn local_pose(&self) -> Transform3D {
        Transform3D {
            basis: Basis::from_quaternion_scale(self.pose_rotation, self.pose_scale),
            origin: self.pose_position,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Skeleton3DData {
    bones: Vec<SkeletonBone>,
    name_to_bone: HashMap<String, usize>,
    show_rest_only: bool,
}

impl Skeleton3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bone(&mut self, name: &str) -> Option<usize> {
        if name.is_empty() || name.contains(':') || name.contains('/') || self.name_to_bone.contains_key(name) {
            return None;
        }
        let index = self.bones.len();
        self.bones.push(SkeletonBone::new(name));
        self.name_to_bone.insert(name.to_string(), index);
        Some(index)
    }

    pub fn clear_bones(&mut self) {
        self.bones.clear();
        self.name_to_bone.clear();
    }

    pub fn get_bone_count(&self) -> usize {
        self.bones.len()
    }

    pub fn get_bone(&self, bone: usize) -> Option<&SkeletonBone> {
        self.bones.get(bone)
    }

    pub fn find_bone(&self, name: &str) -> i32 {
        self.name_to_bone.get(name).map(|i| *i as i32).unwrap_or(-1)
    }

    pub fn get_bone_name(&self, bone: usize) -> &str {
        self.bones.get(bone).map(|b| b.name.as_str()).unwrap_or("")
    }

    /// Renames unless the name belongs to another bone (same name is a no-op).
    pub fn set_bone_name(&mut self, bone: usize, name: &str) -> bool {
        if bone >= self.bones.len() {
            return false;
        }
        if name.is_empty() || name.contains(':') || name.contains('/') {
            return false;
        }
        if let Some(existing) = self.name_to_bone.get(name) {
            return *existing == bone;
        }
        self.name_to_bone.remove(&self.bones[bone].name);
        self.bones[bone].name = name.to_string();
        self.name_to_bone.insert(name.to_string(), bone);
        true
    }

    /// Parent must exist (or -1) and differ from the bone.
    pub fn set_bone_parent(&mut self, bone: usize, parent: i32) -> bool {
        if bone >= self.bones.len() || parent < -1 || parent as usize == bone {
            return false;
        }
        if parent >= 0 && parent as usize >= self.bones.len() {
            return false;
        }
        self.bones[bone].parent = parent;
        true
    }

    pub fn get_bone_parent(&self, bone: usize) -> i32 {
        self.bones.get(bone).map(|b| b.parent).unwrap_or(-1)
    }

    /// True when `parent_bone` is an ancestor of `bone` (or equal).
    pub fn is_bone_parent_of(&self, bone: usize, parent_bone: usize) -> bool {
        if bone >= self.bones.len() || parent_bone >= self.bones.len() {
            return false;
        }
        let mut current = bone as i32;
        let mut steps = 0;
        while current >= 0 {
            if current as usize == parent_bone {
                return true;
            }
            current = self.bones[current as usize].parent;
            steps += 1;
            if steps > self.bones.len() {
                return false;
            }
        }
        false
    }

    pub fn set_bone_rest(&mut self, bone: usize, rest: Transform3D) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.rest = rest;
                true
            }
            None => false,
        }
    }

    pub fn get_bone_rest(&self, bone: usize) -> Transform3D {
        self.bones
            .get(bone)
            .map(|b| b.rest)
            .unwrap_or(Transform3D::IDENTITY)
    }

    /// Composes rest transforms up the parent chain.
    pub fn get_bone_global_rest(&self, bone: usize) -> Option<Transform3D> {
        self.compose(bone, |b| b.rest)
    }

    pub fn set_bone_enabled(&mut self, bone: usize, enabled: bool) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.enabled = enabled;
                true
            }
            None => false,
        }
    }

    pub fn is_bone_enabled(&self, bone: usize) -> bool {
        self.bones.get(bone).map(|b| b.enabled).unwrap_or(false)
    }

    pub fn set_show_rest_only(&mut self, enabled: bool) {
        self.show_rest_only = enabled;
    }

    pub fn is_showing_rest_only(&self) -> bool {
        self.show_rest_only
    }

    pub fn set_bone_pose_position(&mut self, bone: usize, position: Vector3) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.pose_position = position;
                true
            }
            None => false,
        }
    }

    pub fn get_bone_pose_position(&self, bone: usize) -> Vector3 {
        self.bones
            .get(bone)
            .map(|b| b.pose_position)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn set_bone_pose_rotation(&mut self, bone: usize, rotation: Quaternion) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.pose_rotation = rotation;
                true
            }
            None => false,
        }
    }

    pub fn get_bone_pose_rotation(&self, bone: usize) -> Quaternion {
        self.bones
            .get(bone)
            .map(|b| b.pose_rotation)
            .unwrap_or(Quaternion::IDENTITY)
    }

    pub fn set_bone_pose_scale(&mut self, bone: usize, scale: Vector3) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.pose_scale = scale;
                true
            }
            None => false,
        }
    }

    pub fn get_bone_pose_scale(&self, bone: usize) -> Vector3 {
        self.bones
            .get(bone)
            .map(|b| b.pose_scale)
            .unwrap_or(Vector3::ONE)
    }

    /// Composes local poses (`show_rest_only` switches to the rests).
    pub fn get_bone_global_pose(&self, bone: usize) -> Option<Transform3D> {
        if self.show_rest_only {
            return self.get_bone_global_rest(bone);
        }
        self.compose(bone, |b| b.local_pose())
    }

    /// Sets the local pose so the bone ends up at `pose` in skeleton space.
    pub fn set_bone_global_pose(&mut self, bone: usize, pose: Transform3D) -> bool {
        let parent = match self.bones.get(bone) {
            Some(b) => b.parent,
            None => return false,
        };
        let local = if parent >= 0 {
            match self.get_bone_global_pose(parent as usize) {
                Some(parent_pose) => parent_pose.affine_inverse() * pose,
                None => return false,
            }
        } else {
            pose
        };
        let b = &mut self.bones[bone];
        b.pose_position = local.origin;
        b.pose_rotation = local.basis.get_rotation_quaternion();
        b.pose_scale = local.basis.get_scale();
        true
    }

    pub fn reset_bone_pose(&mut self, bone: usize) -> bool {
        match self.bones.get_mut(bone) {
            Some(b) => {
                b.pose_position = Vector3::ZERO;
                b.pose_rotation = Quaternion::IDENTITY;
                b.pose_scale = Vector3::ONE;
                true
            }
            None => false,
        }
    }

    pub fn reset_all_bone_poses(&mut self) {
        for bone in 0..self.bones.len() {
            self.reset_bone_pose(bone);
        }
    }

    fn compose<F>(&self, bone: usize, local: F) -> Option<Transform3D>
    where
        F: Fn(&SkeletonBone) -> Transform3D,
    {
        if bone >= self.bones.len() {
            return None;
        }
        let mut chain = Vec::new();
        let mut current = bone as i32;
        while current >= 0 {
            let index = current as usize;
            if index >= self.bones.len() || chain.contains(&index) {
                return None;
            }
            chain.push(index);
            current = self.bones[index].parent;
            if chain.len() > self.bones.len() {
                return None;
            }
        }

        let mut result = Transform3D::IDENTITY;
        for index in chain.into_iter().rev() {
            result = result * local(&self.bones[index]);
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton() -> (Skeleton3DData, usize, usize) {
        let mut s = Skeleton3DData::new();
        let root = s.add_bone("root").unwrap();
        let child = s.add_bone("child").unwrap();
        assert!(s.set_bone_parent(child, root as i32));
        (s, root, child)
    }

    #[test]
    fn add_and_name_rules() {
        let mut s = Skeleton3DData::new();
        assert_eq!(s.add_bone("root"), Some(0));
        assert_eq!(s.add_bone("root"), None);
        assert_eq!(s.add_bone(""), None);
        assert_eq!(s.add_bone("a:b"), None);
        assert_eq!(s.add_bone("a/b"), None);
        assert_eq!(s.add_bone("child"), Some(1));
        assert_eq!(s.get_bone_count(), 2);
        assert_eq!(s.find_bone("child"), 1);
        assert_eq!(s.find_bone("missing"), -1);

        assert!(s.set_bone_name(1, "hand"));
        assert_eq!(s.get_bone_name(1), "hand");
        assert_eq!(s.find_bone("hand"), 1);
        assert_eq!(s.find_bone("child"), -1);
        // Same name is a no-op; another bone's name is rejected.
        assert!(s.set_bone_name(1, "hand"));
        assert!(!s.set_bone_name(1, "root"));
        assert_eq!(s.get_bone_name(1), "hand");

        s.clear_bones();
        assert_eq!(s.get_bone_count(), 0);
        assert_eq!(s.find_bone("root"), -1);
    }

    #[test]
    fn parenting_and_ancestry() {
        let (mut s, root, child) = skeleton();
        assert_eq!(s.get_bone_parent(child), root as i32);
        assert!(s.is_bone_parent_of(child, root));
        assert!(s.is_bone_parent_of(root, root));
        assert!(!s.is_bone_parent_of(root, child));

        let other = s.add_bone("other").unwrap();
        assert!(!s.set_bone_parent(other, other as i32));
        assert!(!s.set_bone_parent(other, 99));
        assert!(s.set_bone_parent(other, -1));
        assert_eq!(s.get_bone_parent(other), -1);
    }

    #[test]
    fn rest_and_pose_composition() {
        let (mut s, root, child) = skeleton();
        let root_rest = Transform3D {
            basis: Basis::IDENTITY,
            origin: Vector3::new(1.0, 0.0, 0.0),
        };
        let child_rest = Transform3D {
            basis: Basis::IDENTITY,
            origin: Vector3::new(0.0, 2.0, 0.0),
        };
        assert!(s.set_bone_rest(root, root_rest));
        assert!(s.set_bone_rest(child, child_rest));
        let global = s.get_bone_global_rest(child).unwrap();
        assert!((global.origin - Vector3::new(1.0, 2.0, 0.0)).length() < 1e-5);

        // Pose defaults are identity, so the global pose equals the parent chain.
        assert!(s.set_bone_pose_position(root, Vector3::new(0.0, 0.0, 3.0)));
        assert!(s.set_bone_pose_position(child, Vector3::new(0.0, 1.0, 0.0)));
        let pose = s.get_bone_global_pose(child).unwrap();
        assert!((pose.origin - Vector3::new(0.0, 1.0, 3.0)).length() < 1e-5);

        // show_rest_only switches to rests.
        s.set_show_rest_only(true);
        let rest_pose = s.get_bone_global_pose(child).unwrap();
        assert!((rest_pose.origin - Vector3::new(1.0, 2.0, 0.0)).length() < 1e-5);
        s.set_show_rest_only(false);

        // Setting a global pose computes the local pose accordingly.
        let target = Transform3D {
            basis: Basis::IDENTITY,
            origin: Vector3::new(5.0, 5.0, 5.0),
        };
        assert!(s.set_bone_global_pose(child, target));
        let back = s.get_bone_global_pose(child).unwrap();
        assert!((back.origin - target.origin).length() < 1e-4, "{back:?}");

        assert!(s.reset_bone_pose(child));
        assert_eq!(s.get_bone_pose_position(child), Vector3::ZERO);
        assert_eq!(s.get_bone_pose_rotation(child), Quaternion::IDENTITY);
        assert_eq!(s.get_bone_pose_scale(child), Vector3::ONE);
    }

    #[test]
    fn enabled_flag_and_invalid_indices() {
        let (mut s, _root, child) = skeleton();
        assert!(s.is_bone_enabled(child));
        assert!(s.set_bone_enabled(child, false));
        assert!(!s.is_bone_enabled(child));

        assert!(!s.is_bone_enabled(9));
        assert_eq!(s.get_bone_name(9), "");
        assert_eq!(s.get_bone_parent(9), -1);
        assert_eq!(s.get_bone_rest(9), Transform3D::IDENTITY);
        assert!(s.get_bone_global_pose(9).is_none());
        assert!(!s.reset_bone_pose(9));
        assert!(!s.set_bone_pose_position(9, Vector3::ZERO));
    }
}
