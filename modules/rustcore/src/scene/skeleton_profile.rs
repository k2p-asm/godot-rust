//! `scene/resources/skeleton_profile.*`: group/bone metadata.
//!
//! Defaults: empty root/scale-base names, no groups or bones. `set_group_size`
//! and `set_bone_size` reject negative sizes; every per-index accessor validates
//! and returns the engine's default on failure. All mutating setters are no-ops
//! while `read_only` is set (the engine flag used by `SkeletonProfileHumanoid`).
//! `find_bone("")` is -1, matching the engine's early out.

use crate::math::transform_3d::Transform3D;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;
use crate::string::string_name::StringName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TailDirection {
    AverageChildren,
    SpecificChild,
    End,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkeletonProfileGroup {
    pub name: StringName,
    pub texture: Option<Rid>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonProfileBone {
    pub bone_name: StringName,
    pub bone_parent: StringName,
    pub tail_direction: TailDirection,
    pub bone_tail: StringName,
    pub reference_pose: Transform3D,
    pub handle_offset: Vector2,
    pub group: StringName,
    pub required: bool,
}

impl Default for SkeletonProfileBone {
    fn default() -> Self {
        Self {
            bone_name: StringName::new(""),
            bone_parent: StringName::new(""),
            tail_direction: TailDirection::AverageChildren,
            bone_tail: StringName::new(""),
            reference_pose: Transform3D::IDENTITY,
            handle_offset: Vector2::ZERO,
            group: StringName::new(""),
            required: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkeletonProfileData {
    read_only: bool,
    root_bone: StringName,
    scale_base_bone: StringName,
    groups: Vec<SkeletonProfileGroup>,
    bones: Vec<SkeletonProfileBone>,
}

impl SkeletonProfileData {
    pub fn new() -> Self {
        Self {
            read_only: false,
            root_bone: StringName::new(""),
            scale_base_bone: StringName::new(""),
            groups: Vec::new(),
            bones: Vec::new(),
        }
    }

    /// Used by read-only subclasses such as `SkeletonProfileHumanoid`.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn get_root_bone(&self) -> StringName {
        self.root_bone
    }

    pub fn set_root_bone(&mut self, name: &str) -> bool {
        if self.read_only {
            return false;
        }
        self.root_bone = StringName::new(name);
        true
    }

    pub fn get_scale_base_bone(&self) -> StringName {
        self.scale_base_bone
    }

    pub fn set_scale_base_bone(&mut self, name: &str) -> bool {
        if self.read_only {
            return false;
        }
        self.scale_base_bone = StringName::new(name);
        true
    }

    // -- groups -------------------------------------------------------------

    pub fn get_group_size(&self) -> usize {
        self.groups.len()
    }

    /// Rejects negative sizes; no-op while read-only.
    pub fn set_group_size(&mut self, size: i32) -> bool {
        if self.read_only || size < 0 {
            return false;
        }
        self.groups
            .resize(size as usize, SkeletonProfileGroup::default());
        true
    }

    pub fn get_group_name(&self, group: usize) -> StringName {
        self.groups
            .get(group)
            .map(|g| g.name)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn set_group_name(&mut self, group: usize, name: &str) -> bool {
        if self.read_only {
            return false;
        }
        match self.groups.get_mut(group) {
            Some(g) => {
                g.name = StringName::new(name);
                true
            }
            None => false,
        }
    }

    pub fn get_texture(&self, group: usize) -> Option<Rid> {
        self.groups.get(group).and_then(|g| g.texture)
    }

    pub fn set_texture(&mut self, group: usize, texture: Option<Rid>) -> bool {
        if self.read_only {
            return false;
        }
        match self.groups.get_mut(group) {
            Some(g) => {
                g.texture = texture;
                true
            }
            None => false,
        }
    }

    // -- bones --------------------------------------------------------------

    pub fn get_bone_size(&self) -> usize {
        self.bones.len()
    }

    pub fn set_bone_size(&mut self, size: i32) -> bool {
        if self.read_only || size < 0 {
            return false;
        }
        self.bones
            .resize(size as usize, SkeletonProfileBone::default());
        true
    }

    /// First matching bone, or -1; the empty name is never found.
    pub fn find_bone(&self, name: &str) -> i32 {
        let wanted = StringName::new(name);
        if wanted == StringName::new("") {
            return -1;
        }
        self.bones
            .iter()
            .position(|b| b.bone_name == wanted)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    pub fn has_bone(&self, name: &str) -> bool {
        self.find_bone(name) >= 0
    }

    pub fn get_bone_names(&self) -> Vec<StringName> {
        self.bones.iter().map(|b| b.bone_name).collect()
    }

    pub fn get_bone_name(&self, bone: usize) -> StringName {
        self.bones
            .get(bone)
            .map(|b| b.bone_name)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn set_bone_name(&mut self, bone: usize, name: &str) -> bool {
        self.edit_bone(bone, |b| b.bone_name = StringName::new(name))
    }

    pub fn get_bone_parent(&self, bone: usize) -> StringName {
        self.bones
            .get(bone)
            .map(|b| b.bone_parent)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn set_bone_parent(&mut self, bone: usize, parent: &str) -> bool {
        self.edit_bone(bone, |b| b.bone_parent = StringName::new(parent))
    }

    pub fn get_tail_direction(&self, bone: usize) -> TailDirection {
        self.bones
            .get(bone)
            .map(|b| b.tail_direction)
            .unwrap_or(TailDirection::AverageChildren)
    }

    pub fn set_tail_direction(&mut self, bone: usize, direction: TailDirection) -> bool {
        self.edit_bone(bone, |b| b.tail_direction = direction)
    }

    pub fn get_bone_tail(&self, bone: usize) -> StringName {
        self.bones
            .get(bone)
            .map(|b| b.bone_tail)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn set_bone_tail(&mut self, bone: usize, tail: &str) -> bool {
        self.edit_bone(bone, |b| b.bone_tail = StringName::new(tail))
    }

    pub fn get_reference_pose(&self, bone: usize) -> Transform3D {
        self.bones
            .get(bone)
            .map(|b| b.reference_pose)
            .unwrap_or(Transform3D::IDENTITY)
    }

    pub fn set_reference_pose(&mut self, bone: usize, pose: Transform3D) -> bool {
        self.edit_bone(bone, |b| b.reference_pose = pose)
    }

    pub fn get_handle_offset(&self, bone: usize) -> Vector2 {
        self.bones
            .get(bone)
            .map(|b| b.handle_offset)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn set_handle_offset(&mut self, bone: usize, offset: Vector2) -> bool {
        self.edit_bone(bone, |b| b.handle_offset = offset)
    }

    pub fn get_group(&self, bone: usize) -> StringName {
        self.bones
            .get(bone)
            .map(|b| b.group)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn set_group(&mut self, bone: usize, group: &str) -> bool {
        self.edit_bone(bone, |b| b.group = StringName::new(group))
    }

    pub fn is_required(&self, bone: usize) -> bool {
        self.bones.get(bone).map(|b| b.required).unwrap_or(false)
    }

    pub fn set_required(&mut self, bone: usize, required: bool) -> bool {
        self.edit_bone(bone, |b| b.required = required)
    }

    fn edit_bone(&mut self, bone: usize, f: impl FnOnce(&mut SkeletonProfileBone)) -> bool {
        if self.read_only {
            return false;
        }
        match self.bones.get_mut(bone) {
            Some(b) => {
                f(b);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn defaults() {
        let p = SkeletonProfileData::new();
        assert_eq!(p.get_root_bone().as_string(), "");
        assert_eq!(p.get_scale_base_bone().as_string(), "");
        assert_eq!(p.get_group_size(), 0);
        assert_eq!(p.get_bone_size(), 0);
        assert!(!p.is_read_only());
        assert_eq!(p.find_bone(""), -1);
        assert!(!p.has_bone("x"));
    }

    #[test]
    fn groups_and_bones_resize() {
        let mut p = SkeletonProfileData::new();
        assert!(!p.set_group_size(-1));
        assert!(p.set_group_size(2));
        assert_eq!(p.get_group_size(), 2);
        assert_eq!(p.get_group_name(0).as_string(), "");
        assert!(p.get_texture(0).is_none());

        assert!(p.set_group_name(1, "Body"));
        assert!(p.set_texture(1, Some(Rid::from_u64(3))));
        assert_eq!(p.get_group_name(1).as_string(), "Body");
        assert_eq!(p.get_texture(1), Some(Rid::from_u64(3)));
        assert!(!p.set_group_name(2, "x"));
        assert_eq!(p.get_group_name(9).as_string(), "");

        assert!(!p.set_bone_size(-1));
        assert!(p.set_bone_size(2));
        assert_eq!(p.get_bone_size(), 2);
        assert_eq!(p.get_bone_name(0).as_string(), "");
        assert_eq!(p.get_bone_parent(0).as_string(), "");
        assert_eq!(p.get_tail_direction(0), TailDirection::AverageChildren);
        assert_eq!(p.get_reference_pose(0), Transform3D::IDENTITY);
        assert_eq!(p.get_handle_offset(0), Vector2::ZERO);
        assert!(!p.is_required(0));

        // Resizing keeps the existing entries.
        assert!(p.set_bone_name(0, "Root"));
        assert!(p.set_bone_size(4));
        assert_eq!(p.get_bone_name(0).as_string(), "Root");
        assert_eq!(p.get_bone_name(3).as_string(), "");
    }

    #[test]
    fn bone_fields_and_lookup() {
        let mut p = SkeletonProfileData::new();
        p.set_bone_size(3);
        let pose = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::new(0.0, 1.0, 0.0),
        };

        assert!(p.set_bone_name(0, "Root"));
        assert!(p.set_bone_name(1, "Spine"));
        assert!(p.set_bone_parent(1, "Root"));
        assert!(p.set_tail_direction(1, TailDirection::SpecificChild));
        assert!(p.set_bone_tail(1, "Chest"));
        assert!(p.set_reference_pose(1, pose));
        assert!(p.set_handle_offset(1, Vector2::new(2.0, 3.0)));
        assert!(p.set_group(1, "Body"));
        assert!(p.set_required(1, true));

        assert_eq!(p.get_bone_name(1).as_string(), "Spine");
        assert_eq!(p.get_bone_parent(1).as_string(), "Root");
        assert_eq!(p.get_tail_direction(1), TailDirection::SpecificChild);
        assert_eq!(p.get_bone_tail(1).as_string(), "Chest");
        assert_eq!(p.get_reference_pose(1).origin, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(p.get_handle_offset(1), Vector2::new(2.0, 3.0));
        assert_eq!(p.get_group(1).as_string(), "Body");
        assert!(p.is_required(1));

        assert_eq!(p.find_bone("Spine"), 1);
        assert_eq!(p.find_bone("missing"), -1);
        assert!(p.has_bone("Root"));
        let names: Vec<String> = p.get_bone_names().iter().map(|n| n.as_string()).collect();
        assert_eq!(names, vec!["Root", "Spine", ""]);

        assert!(!p.set_bone_name(9, "x"));
        assert!(!p.set_required(9, true));
    }

    #[test]
    fn read_only_blocks_mutations() {
        let mut p = SkeletonProfileData::new();
        p.set_bone_size(1);
        p.set_group_size(1);
        p.set_read_only(true);
        assert!(p.is_read_only());

        assert!(!p.set_root_bone("Root"));
        assert!(!p.set_scale_base_bone("Root"));
        assert!(!p.set_group_size(2));
        assert!(!p.set_group_name(0, "Body"));
        assert!(!p.set_bone_size(2));
        assert!(!p.set_bone_name(0, "Root"));
        assert!(!p.set_required(0, true));

        // Reads still work.
        assert_eq!(p.get_bone_size(), 1);
        assert_eq!(p.get_group_size(), 1);

        assert_eq!(TailDirection::AverageChildren as i32, 0);
        assert_eq!(TailDirection::SpecificChild as i32, 1);
        assert_eq!(TailDirection::End as i32, 2);
    }
}
