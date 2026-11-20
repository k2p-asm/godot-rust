//! `scene/3d/ik_modifier_3d.*` and `two_bone_ik_3d.*` (state): two-bone IK.
//!
//! Ported configuration: the base IK modifier's mutable-bone-axes flag and
//! settings count; `TwoBoneIK3D`'s per-setting root/middle/end bones, the virtual
//! end joint (use/extend/direction/length), the pole node and direction
//! (`NONE` by default) with its custom vector, and the target node.
//!
//! The solver (`_process_ik`), the joint rest/pose capture and the gizmo are not
//! ported.

use crate::math::vector3::Vector3;
use crate::string::node_path::NodePath;

pub const SECONDARY_DIRECTION_NONE: i32 = 0;
pub const SECONDARY_DIRECTION_PLUS_X: i32 = 1;
pub const SECONDARY_DIRECTION_MINUS_X: i32 = 2;
pub const SECONDARY_DIRECTION_PLUS_Y: i32 = 3;
pub const SECONDARY_DIRECTION_MINUS_Y: i32 = 4;
pub const SECONDARY_DIRECTION_PLUS_Z: i32 = 5;
pub const SECONDARY_DIRECTION_MINUS_Z: i32 = 6;
pub const SECONDARY_DIRECTION_CUSTOM: i32 = 7;

pub const BONE_DIRECTION_FROM_PARENT: i32 = 6;

#[derive(Clone, Debug)]
pub struct IkBoneJoint {
    pub name: String,
    pub bone: i32,
}

impl Default for IkBoneJoint {
    fn default() -> Self {
        Self {
            name: String::new(),
            bone: -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TwoBoneIkSetting {
    pub root_bone: IkBoneJoint,
    pub middle_bone: IkBoneJoint,
    pub end_bone: IkBoneJoint,
    pub use_virtual_end: bool,
    pub extend_end_bone: bool,
    pub end_bone_direction: i32,
    pub end_bone_length: f32,
    pub pole_node: NodePath,
    pub pole_direction: i32,
    pub pole_direction_vector: Vector3,
    pub target_node: NodePath,
}

impl Default for TwoBoneIkSetting {
    fn default() -> Self {
        Self {
            root_bone: IkBoneJoint::default(),
            middle_bone: IkBoneJoint::default(),
            end_bone: IkBoneJoint::default(),
            use_virtual_end: false,
            extend_end_bone: false,
            end_bone_direction: BONE_DIRECTION_FROM_PARENT,
            end_bone_length: 0.0,
            pole_node: NodePath::new(),
            pole_direction: SECONDARY_DIRECTION_NONE,
            pole_direction_vector: Vector3::ZERO,
            target_node: NodePath::new(),
        }
    }
}

impl TwoBoneIkSetting {
    /// `get_end_bone`: the virtual end joint resolves to the middle bone.
    pub fn get_end_bone(&self) -> i32 {
        if self.use_virtual_end {
            self.middle_bone.bone
        } else {
            self.end_bone.bone
        }
    }

    /// `is_end_valid`.
    pub fn is_end_valid(&self) -> bool {
        if self.use_virtual_end {
            self.end_bone_length.abs() > 1e-5
        } else {
            self.end_bone.bone != -1
        }
    }

    /// `get_pole_direction_vector`.
    pub fn get_pole_direction_vector(&self) -> Vector3 {
        match self.pole_direction {
            SECONDARY_DIRECTION_PLUS_X => Vector3::new(1.0, 0.0, 0.0),
            SECONDARY_DIRECTION_MINUS_X => Vector3::new(-1.0, 0.0, 0.0),
            SECONDARY_DIRECTION_PLUS_Y => Vector3::new(0.0, 1.0, 0.0),
            SECONDARY_DIRECTION_MINUS_Y => Vector3::new(0.0, -1.0, 0.0),
            SECONDARY_DIRECTION_PLUS_Z => Vector3::new(0.0, 0.0, 1.0),
            SECONDARY_DIRECTION_MINUS_Z => Vector3::new(0.0, 0.0, -1.0),
            SECONDARY_DIRECTION_CUSTOM => self.pole_direction_vector,
            _ => Vector3::ZERO,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TwoBoneIk3DData {
    mutable_bone_axes: bool,
    settings: Vec<TwoBoneIkSetting>,
}

impl TwoBoneIk3DData {
    pub fn new() -> Self {
        Self {
            mutable_bone_axes: true,
            settings: Vec::new(),
        }
    }

    pub fn set_mutable_bone_axes(&mut self, enabled: bool) {
        self.mutable_bone_axes = enabled;
    }

    pub fn are_bone_axes_mutable(&self) -> bool {
        self.mutable_bone_axes
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.settings.resize_with(count, TwoBoneIkSetting::default);
    }

    pub fn clear_settings(&mut self) {
        self.settings.clear();
    }

    pub fn get_setting(&self, index: usize) -> Option<&TwoBoneIkSetting> {
        self.settings.get(index)
    }

    pub fn set_root_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.root_bone.name = name.to_string();
        }
    }

    pub fn get_root_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.root_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_root_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.root_bone.bone = bone;
        }
    }

    pub fn get_root_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.root_bone.bone)
            .unwrap_or(-1)
    }

    pub fn set_middle_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.middle_bone.name = name.to_string();
        }
    }

    pub fn get_middle_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.middle_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_middle_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.middle_bone.bone = bone;
        }
    }

    pub fn get_middle_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.middle_bone.bone)
            .unwrap_or(-1)
    }

    pub fn set_end_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone.name = name.to_string();
        }
    }

    pub fn get_end_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_end_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone.bone = bone;
        }
    }

    pub fn get_end_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.get_end_bone())
            .unwrap_or(-1)
    }

    pub fn set_use_virtual_end(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.use_virtual_end = enabled;
        }
    }

    pub fn is_using_virtual_end(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.use_virtual_end)
            .unwrap_or(false)
    }

    pub fn set_extend_end_bone(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.extend_end_bone = enabled;
        }
    }

    pub fn is_end_bone_extended(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.extend_end_bone)
            .unwrap_or(false)
    }

    pub fn set_end_bone_direction(&mut self, index: usize, direction: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone_direction = direction;
        }
    }

    pub fn get_end_bone_direction(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone_direction)
            .unwrap_or(BONE_DIRECTION_FROM_PARENT)
    }

    pub fn set_end_bone_length(&mut self, index: usize, length: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone_length = length;
        }
    }

    pub fn get_end_bone_length(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone_length)
            .unwrap_or(0.0)
    }

    pub fn set_pole_node(&mut self, index: usize, path: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.pole_node = path;
        }
    }

    pub fn get_pole_node(&self, index: usize) -> NodePath {
        self.settings
            .get(index)
            .map(|setting| setting.pole_node.clone())
            .unwrap_or_default()
    }

    pub fn set_pole_direction(&mut self, index: usize, direction: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.pole_direction = direction;
        }
    }

    pub fn get_pole_direction(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.pole_direction)
            .unwrap_or(SECONDARY_DIRECTION_NONE)
    }

    pub fn set_pole_direction_vector(&mut self, index: usize, vector: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.pole_direction_vector = vector;
        }
    }

    pub fn get_pole_direction_vector(&self, index: usize) -> Vector3 {
        self.settings
            .get(index)
            .map(|setting| setting.get_pole_direction_vector())
            .unwrap_or(Vector3::ZERO)
    }

    pub fn set_target_node(&mut self, index: usize, path: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.target_node = path;
        }
    }

    pub fn get_target_node(&self, index: usize) -> NodePath {
        self.settings
            .get(index)
            .map(|setting| setting.target_node.clone())
            .unwrap_or_default()
    }

    pub fn is_end_valid(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.is_end_valid())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let mut ik = TwoBoneIk3DData::new();
        assert!(ik.are_bone_axes_mutable());
        assert_eq!(ik.get_setting_count(), 0);
        ik.set_setting_count(1);
        assert_eq!(ik.get_setting_count(), 1);
        assert_eq!(ik.get_root_bone_name(0), "");
        assert_eq!(ik.get_root_bone(0), -1);
        assert_eq!(ik.get_middle_bone(0), -1);
        assert_eq!(ik.get_end_bone(0), -1);
        assert!(!ik.is_using_virtual_end(0));
        assert!(!ik.is_end_bone_extended(0));
        assert_eq!(ik.get_end_bone_direction(0), BONE_DIRECTION_FROM_PARENT);
        assert_eq!(ik.get_end_bone_length(0), 0.0);
        assert!(ik.get_pole_node(0).is_empty());
        assert_eq!(ik.get_pole_direction(0), SECONDARY_DIRECTION_NONE);
        assert_eq!(ik.get_pole_direction_vector(0), Vector3::ZERO);
        assert!(ik.get_target_node(0).is_empty());
        assert!(!ik.is_end_valid(0));
    }

    #[test]
    fn bones_and_joint_flags() {
        let mut ik = TwoBoneIk3DData::new();
        ik.set_setting_count(1);
        ik.set_mutable_bone_axes(false);
        ik.set_root_bone_name(0, "UpperArm");
        ik.set_root_bone(0, 1);
        ik.set_middle_bone_name(0, "LowerArm");
        ik.set_middle_bone(0, 2);
        ik.set_end_bone_name(0, "Hand");
        ik.set_end_bone(0, 3);
        ik.set_extend_end_bone(0, true);
        ik.set_end_bone_direction(0, 2);
        ik.set_end_bone_length(0, 0.15);

        assert!(!ik.are_bone_axes_mutable());
        assert_eq!(ik.get_root_bone_name(0), "UpperArm");
        assert_eq!(ik.get_root_bone(0), 1);
        assert_eq!(ik.get_middle_bone_name(0), "LowerArm");
        assert_eq!(ik.get_middle_bone(0), 2);
        assert_eq!(ik.get_end_bone_name(0), "Hand");
        assert_eq!(ik.get_end_bone(0), 3);
        assert!(ik.is_end_bone_extended(0));
        assert_eq!(ik.get_end_bone_direction(0), 2);
        assert_eq!(ik.get_end_bone_length(0), 0.15);
        assert!(ik.is_end_valid(0));

        // Virtual end: the end bone resolves to the middle bone.
        ik.set_use_virtual_end(0, true);
        assert!(ik.is_using_virtual_end(0));
        assert_eq!(ik.get_end_bone(0), 2);
        // A zero virtual length is invalid; a positive one is valid.
        ik.set_end_bone_length(0, 0.0);
        assert!(!ik.is_end_valid(0));
        ik.set_end_bone_length(0, 0.2);
        assert!(ik.is_end_valid(0));
        // Out-of-range settings keep defaults.
        assert_eq!(ik.get_root_bone(9), -1);
        assert!(!ik.is_end_valid(9));
    }

    #[test]
    fn pole_directions() {
        let mut ik = TwoBoneIk3DData::new();
        ik.set_setting_count(1);
        let cases = [
            (SECONDARY_DIRECTION_NONE, Vector3::ZERO),
            (SECONDARY_DIRECTION_PLUS_X, Vector3::new(1.0, 0.0, 0.0)),
            (SECONDARY_DIRECTION_MINUS_X, Vector3::new(-1.0, 0.0, 0.0)),
            (SECONDARY_DIRECTION_PLUS_Y, Vector3::new(0.0, 1.0, 0.0)),
            (SECONDARY_DIRECTION_MINUS_Y, Vector3::new(0.0, -1.0, 0.0)),
            (SECONDARY_DIRECTION_PLUS_Z, Vector3::new(0.0, 0.0, 1.0)),
            (SECONDARY_DIRECTION_MINUS_Z, Vector3::new(0.0, 0.0, -1.0)),
        ];
        for (direction, expected) in cases {
            ik.set_pole_direction(0, direction);
            assert_eq!(ik.get_pole_direction(0), direction);
            assert_eq!(ik.get_pole_direction_vector(0), expected);
        }
        // The custom direction uses the stored vector.
        ik.set_pole_direction(0, SECONDARY_DIRECTION_CUSTOM);
        assert_eq!(ik.get_pole_direction_vector(0), Vector3::ZERO);
        ik.set_pole_direction_vector(0, Vector3::new(0.5, 0.5, 0.0));
        assert_eq!(
            ik.get_pole_direction_vector(0),
            Vector3::new(0.5, 0.5, 0.0)
        );
        // Switching back ignores the custom vector.
        ik.set_pole_direction(0, SECONDARY_DIRECTION_PLUS_Z);
        assert_eq!(ik.get_pole_direction_vector(0), Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(SECONDARY_DIRECTION_CUSTOM, 7);
    }

    #[test]
    fn nodes_and_clearing() {
        let mut ik = TwoBoneIk3DData::new();
        ik.set_setting_count(2);
        let pole = NodePath::parse("/root/Pole").expect("path");
        let target = NodePath::parse("/root/Target").expect("path");
        ik.set_pole_node(0, pole.clone());
        ik.set_target_node(0, target.clone());
        assert_eq!(ik.get_pole_node(0), pole);
        assert_eq!(ik.get_target_node(0), target);
        assert!(ik.get_pole_node(1).is_empty());
        assert!(ik.get_target_node(1).is_empty());

        assert!(ik.get_setting(0).is_some());
        assert!(ik.get_setting(9).is_none());
        ik.clear_settings();
        assert_eq!(ik.get_setting_count(), 0);
        assert!(ik.get_target_node(0).is_empty());
    }
}
