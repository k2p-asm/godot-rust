//! `scene/2d/skeleton_2d.*`: the skeleton's bone table (no node graph).
//!
//! Each `Skeleton2D` keeps an ordered bone list. The engine's `Bone` record is
//! modelled here as [`SkeletonBone`]: parent index, accumulated transform, the
//! inverse of the skeleton-space rest, and the local-pose override (transform,
//! amount and persistence). The two dirty flags (`bone_setup_dirty` and
//! `transform_dirty`) are exposed with their trigger methods.
//!
//! The node-based bone resolution (finding the `Bone2D` child by name), the
//! skeleton RID and the modification stack's evaluation are not ported; callers
//! add bones explicitly and keep their own handles.

use crate::math::transform_2d::Transform2D;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct SkeletonBone {
    /// Index of the parent bone, -1 for a root.
    pub parent_index: i32,
    pub accum_transform: Transform2D,
    pub rest_inverse: Transform2D,
    pub local_pose_override: Transform2D,
    pub local_pose_override_amount: f32,
    pub local_pose_override_persistent: bool,
}

impl Default for SkeletonBone {
    fn default() -> Self {
        Self {
            parent_index: -1,
            accum_transform: Transform2D::IDENTITY,
            rest_inverse: Transform2D::IDENTITY,
            local_pose_override: Transform2D::IDENTITY,
            local_pose_override_amount: 0.0,
            local_pose_override_persistent: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Skeleton2DData {
    bones: Vec<SkeletonBone>,
    bone_setup_dirty: bool,
    transform_dirty: bool,
    modification_stack: Option<Rid>,
}

impl Default for Skeleton2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton2DData {
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            bone_setup_dirty: true,
            transform_dirty: true,
            modification_stack: None,
        }
    }

    pub fn get_bone_count(&self) -> usize {
        self.bones.len()
    }

    pub fn get_bone(&self, index: usize) -> Option<&SkeletonBone> {
        self.bones.get(index)
    }

    pub fn get_bone_mut(&mut self, index: usize) -> Option<&mut SkeletonBone> {
        self.bones.get_mut(index)
    }

    /// Registers a bone under `parent_index` (-1 for a root) and returns its
    /// index. A parent index that does not exist is treated as a root.
    pub fn add_bone(&mut self, parent_index: i32) -> usize {
        let parent = if parent_index >= 0 && (parent_index as usize) < self.bones.len() {
            parent_index
        } else {
            -1
        };
        self.bones.push(SkeletonBone {
            parent_index: parent,
            ..SkeletonBone::default()
        });
        self._make_bone_setup_dirty();
        self.bones.len() - 1
    }

    /// Removes a bone: children of the removed bone become children of its parent,
    /// and higher indices shift down.
    pub fn remove_bone(&mut self, index: usize) -> bool {
        if index >= self.bones.len() {
            return false;
        }
        let removed = self.bones.remove(index);
        for bone in self.bones.iter_mut() {
            if bone.parent_index == index as i32 {
                bone.parent_index = removed.parent_index;
            } else if bone.parent_index > index as i32 {
                bone.parent_index -= 1;
            }
        }
        self._make_bone_setup_dirty();
        true
    }

    pub fn find_bone(&self, parent_index: i32) -> Option<usize> {
        self.bones
            .iter()
            .position(|bone| bone.parent_index == parent_index)
    }

    /// `set_bone_local_pose_override`: 0 amount clears the override effect.
    pub fn set_bone_local_pose_override(
        &mut self,
        index: usize,
        transform: Transform2D,
        amount: f32,
        persistent: bool,
    ) -> bool {
        let Some(bone) = self.bones.get_mut(index) else {
            return false;
        };
        bone.local_pose_override = transform;
        bone.local_pose_override_amount = amount;
        bone.local_pose_override_persistent = persistent;
        self._make_transform_dirty();
        true
    }

    pub fn get_bone_local_pose_override(&self, index: usize) -> Option<Transform2D> {
        self.bones.get(index).map(|bone| bone.local_pose_override)
    }

    pub fn get_bone_local_pose_override_amount(&self, index: usize) -> f32 {
        self.bones
            .get(index)
            .map(|bone| bone.local_pose_override_amount)
            .unwrap_or(0.0)
    }

    pub fn clear_bone_local_pose_override(&mut self, index: usize) -> bool {
        let Some(bone) = self.bones.get_mut(index) else {
            return false;
        };
        bone.local_pose_override = Transform2D::IDENTITY;
        bone.local_pose_override_amount = 0.0;
        bone.local_pose_override_persistent = false;
        self._make_transform_dirty();
        true
    }

    pub fn set_bone_accum_transform(&mut self, index: usize, transform: Transform2D) -> bool {
        match self.bones.get_mut(index) {
            Some(bone) => {
                bone.accum_transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn set_bone_rest_inverse(&mut self, index: usize, inverse: Transform2D) -> bool {
        match self.bones.get_mut(index) {
            Some(bone) => {
                bone.rest_inverse = inverse;
                true
            }
            None => false,
        }
    }

    // -- dirty flags ---------------------------------------------------------

    pub fn is_bone_setup_dirty(&self) -> bool {
        self.bone_setup_dirty
    }

    pub fn is_transform_dirty(&self) -> bool {
        self.transform_dirty
    }

    fn _make_bone_setup_dirty(&mut self) {
        self.bone_setup_dirty = true;
        self.transform_dirty = true;
    }

    fn _make_transform_dirty(&mut self) {
        self.transform_dirty = true;
    }

    /// `_update_bone_setup` clears the setup flag (and the transform flag with it
    /// in the engine's follow-up update).
    pub fn clear_bone_setup_dirty(&mut self) {
        self.bone_setup_dirty = false;
    }

    pub fn clear_transform_dirty(&mut self) {
        self.transform_dirty = false;
    }

    pub fn set_modification_stack(&mut self, stack: Option<Rid>) {
        self.modification_stack = stack;
    }

    pub fn get_modification_stack(&self) -> Option<Rid> {
        self.modification_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn defaults_and_bone_creation() {
        let mut s = Skeleton2DData::new();
        assert_eq!(s.get_bone_count(), 0);
        assert!(s.get_bone(0).is_none());
        assert!(s.is_bone_setup_dirty());
        assert!(s.is_transform_dirty());
        assert!(s.get_modification_stack().is_none());

        let root = s.add_bone(-1);
        assert_eq!(root, 0);
        let child = s.add_bone(0);
        assert_eq!(child, 1);
        let grandchild = s.add_bone(1);
        assert_eq!(grandchild, 2);
        assert_eq!(s.get_bone(1).unwrap().parent_index, 0);
        // A bad parent becomes a root.
        let orphan = s.add_bone(99);
        assert_eq!(s.get_bone(orphan).unwrap().parent_index, -1);
        assert_eq!(s.get_bone_count(), 4);

        s.clear_bone_setup_dirty();
        s.clear_transform_dirty();
        assert!(!s.is_bone_setup_dirty());
        assert!(!s.is_transform_dirty());
        s.add_bone(0);
        assert!(s.is_bone_setup_dirty() && s.is_transform_dirty());
    }

    #[test]
    fn removal_reparents_children() {
        let mut s = Skeleton2DData::new();
        s.add_bone(-1); // 0
        s.add_bone(0); // 1
        s.add_bone(1); // 2
        s.add_bone(1); // 3

        // Removing bone 1 reparents 2 and 3 to bone 0 and shifts indices.
        assert!(s.remove_bone(1));
        assert_eq!(s.get_bone_count(), 3);
        assert_eq!(s.get_bone(1).unwrap().parent_index, 0);
        assert_eq!(s.get_bone(2).unwrap().parent_index, 0);
        assert!(!s.remove_bone(9));
    }

    #[test]
    fn pose_overrides() {
        let mut s = Skeleton2DData::new();
        s.add_bone(-1);
        let pose = Transform2D {
            columns: [
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(3.0, 4.0),
            ],
        };
        s.clear_transform_dirty();
        assert!(s.set_bone_local_pose_override(0, pose, 0.5, true));
        assert!(s.is_transform_dirty());
        assert_eq!(s.get_bone_local_pose_override(0), Some(pose));
        assert_eq!(s.get_bone_local_pose_override_amount(0), 0.5);
        assert!(s.get_bone(0).unwrap().local_pose_override_persistent);

        assert!(s.clear_bone_local_pose_override(0));
        assert_eq!(s.get_bone_local_pose_override(0), Some(Transform2D::IDENTITY));
        assert_eq!(s.get_bone_local_pose_override_amount(0), 0.0);
        assert!(!s.set_bone_local_pose_override(5, pose, 1.0, false));

        assert!(s.set_bone_accum_transform(0, pose));
        assert!(s.set_bone_rest_inverse(0, pose));
        assert!(!s.set_bone_accum_transform(9, pose));
        assert!(!s.set_bone_rest_inverse(9, pose));

        s.set_modification_stack(Some(Rid::from_u64(2)));
        assert_eq!(s.get_modification_stack(), Some(Rid::from_u64(2)));
    }
}
