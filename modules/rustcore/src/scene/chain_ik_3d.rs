//! `scene/3d/chain_ik_3d.*`, `iterate_ik_3d.*` and `ccd_ik_3d.*` (state).
//!
//! Ported configuration: the chain settings (root/end bones, virtual end joint,
//! joint list), the iterate modifier's `max_iterations` (4) and `min_distance`
//! (0.001), each setting's target node, and the per-joint rotation axis/custom
//! vector with the limitation handle and right-axis fields.
//!
//! The chain solver (CCD/iterate), joint limiting and the gizmo are not ported.

use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::string::node_path::NodePath;

pub const ROTATION_AXIS_X: i32 = 0;
pub const ROTATION_AXIS_Y: i32 = 1;
pub const ROTATION_AXIS_Z: i32 = 2;
pub const ROTATION_AXIS_ALL: i32 = 3;
pub const ROTATION_AXIS_CUSTOM: i32 = 4;

pub const SECONDARY_DIRECTION_NONE: i32 = 0;
pub const SECONDARY_DIRECTION_CUSTOM: i32 = 7;

pub const BONE_DIRECTION_FROM_PARENT: i32 = 6;

#[derive(Clone, Debug)]
pub struct IkJoint {
    pub name: String,
    pub bone: i32,
}

impl Default for IkJoint {
    fn default() -> Self {
        Self {
            name: String::new(),
            bone: -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IterateJointSetting {
    pub rotation_axis: i32,
    pub rotation_axis_vector: Vector3,
    pub limitation: Option<Rid>,
    pub limitation_right_axis: i32,
    pub limitation_right_axis_vector: Vector3,
}

impl Default for IterateJointSetting {
    fn default() -> Self {
        Self {
            rotation_axis: ROTATION_AXIS_ALL,
            rotation_axis_vector: Vector3::new(1.0, 0.0, 0.0),
            limitation: None,
            limitation_right_axis: SECONDARY_DIRECTION_NONE,
            limitation_right_axis_vector: Vector3::new(1.0, 0.0, 0.0),
        }
    }
}

impl IterateJointSetting {
    /// `get_rotation_axis_vector`.
    pub fn get_rotation_axis_vector(&self) -> Vector3 {
        match self.rotation_axis {
            ROTATION_AXIS_X => Vector3::new(1.0, 0.0, 0.0),
            ROTATION_AXIS_Y => Vector3::new(0.0, 1.0, 0.0),
            ROTATION_AXIS_Z => Vector3::new(0.0, 0.0, 1.0),
            ROTATION_AXIS_CUSTOM => self.rotation_axis_vector,
            _ => Vector3::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChainSetting {
    pub root_bone: IkJoint,
    pub end_bone: IkJoint,
    pub extend_end_bone: bool,
    pub end_bone_direction: i32,
    pub end_bone_length: f32,
    pub joints: Vec<IkJoint>,
    pub joint_settings: Vec<IterateJointSetting>,
    pub target_node: NodePath,
}

impl Default for ChainSetting {
    fn default() -> Self {
        Self {
            root_bone: IkJoint::default(),
            end_bone: IkJoint::default(),
            extend_end_bone: false,
            end_bone_direction: BONE_DIRECTION_FROM_PARENT,
            end_bone_length: 0.0,
            joints: Vec::new(),
            joint_settings: Vec::new(),
            target_node: NodePath::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChainIk3DData {
    max_iterations: i32,
    min_distance: f64,
    settings: Vec<ChainSetting>,
}

impl Default for ChainIk3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainIk3DData {
    pub fn new() -> Self {
        Self {
            max_iterations: 4,
            min_distance: 0.001,
            settings: Vec::new(),
        }
    }

    pub fn set_max_iterations(&mut self, iterations: i32) {
        self.max_iterations = iterations;
    }

    pub fn get_max_iterations(&self) -> i32 {
        self.max_iterations
    }

    pub fn set_min_distance(&mut self, distance: f64) {
        self.min_distance = distance;
    }

    pub fn get_min_distance(&self) -> f64 {
        self.min_distance
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.settings.resize_with(count, ChainSetting::default);
    }

    pub fn clear_settings(&mut self) {
        self.settings.clear();
    }

    pub fn get_setting(&self, index: usize) -> Option<&ChainSetting> {
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
            .map(|setting| setting.end_bone.bone)
            .unwrap_or(-1)
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

    pub fn get_joint_count(&self, index: usize) -> usize {
        self.settings
            .get(index)
            .map(|setting| setting.joints.len())
            .unwrap_or(0)
    }

    pub fn set_joint_count(&mut self, index: usize, count: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.joints.resize_with(count, IkJoint::default);
            setting
                .joint_settings
                .resize_with(count, IterateJointSetting::default);
        }
    }

    pub fn get_joint_bone_name(&self, index: usize, joint: usize) -> &str {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.name.as_str())
            .unwrap_or("")
    }

    pub fn get_joint_bone(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.bone)
            .unwrap_or(-1)
    }

    pub fn set_joint_rotation_axis(&mut self, index: usize, joint: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joint_settings.get_mut(joint) {
                entry.rotation_axis = axis;
            }
        }
    }

    pub fn get_joint_rotation_axis(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joint_settings.get(joint))
            .map(|joint| joint.rotation_axis)
            .unwrap_or(ROTATION_AXIS_ALL)
    }

    pub fn set_joint_rotation_axis_vector(&mut self, index: usize, joint: usize, vector: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joint_settings.get_mut(joint) {
                entry.rotation_axis_vector = vector;
            }
        }
    }

    pub fn get_joint_rotation_axis_vector(&self, index: usize, joint: usize) -> Vector3 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joint_settings.get(joint))
            .map(|joint| joint.get_rotation_axis_vector())
            .unwrap_or(Vector3::ZERO)
    }

    pub fn set_joint_limitation(&mut self, index: usize, joint: usize, limitation: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joint_settings.get_mut(joint) {
                entry.limitation = limitation;
            }
        }
    }

    pub fn get_joint_limitation(&self, index: usize, joint: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.joint_settings.get(joint))
            .and_then(|joint| joint.limitation)
    }

    pub fn set_joint_limitation_right_axis(&mut self, index: usize, joint: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joint_settings.get_mut(joint) {
                entry.limitation_right_axis = axis;
            }
        }
    }

    pub fn get_joint_limitation_right_axis(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joint_settings.get(joint))
            .map(|joint| joint.limitation_right_axis)
            .unwrap_or(SECONDARY_DIRECTION_NONE)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let mut ik = ChainIk3DData::new();
        assert_eq!(ik.get_max_iterations(), 4);
        assert_eq!(ik.get_min_distance(), 0.001);
        assert_eq!(ik.get_setting_count(), 0);
        ik.set_setting_count(1);
        assert_eq!(ik.get_setting_count(), 1);
        assert_eq!(ik.get_root_bone_name(0), "");
        assert_eq!(ik.get_root_bone(0), -1);
        assert_eq!(ik.get_end_bone(0), -1);
        assert!(!ik.is_end_bone_extended(0));
        assert_eq!(ik.get_end_bone_direction(0), BONE_DIRECTION_FROM_PARENT);
        assert_eq!(ik.get_end_bone_length(0), 0.0);
        assert_eq!(ik.get_joint_count(0), 0);
        assert_eq!(ik.get_joint_bone(0, 0), -1);
        assert_eq!(ik.get_joint_rotation_axis(0, 0), ROTATION_AXIS_ALL);
        assert_eq!(ik.get_joint_limitation(0, 0), None);
        assert_eq!(
            ik.get_joint_limitation_right_axis(0, 0),
            SECONDARY_DIRECTION_NONE
        );
        assert!(ik.get_target_node(0).is_empty());
        assert!(ik.get_setting(0).is_some());
        assert!(ik.get_setting(9).is_none());
    }

    #[test]
    fn bones_and_joints() {
        let mut ik = ChainIk3DData::new();
        ik.set_setting_count(1);
        ik.set_max_iterations(12);
        ik.set_min_distance(0.005);
        ik.set_root_bone_name(0, "Hips");
        ik.set_root_bone(0, 1);
        ik.set_end_bone_name(0, "Head");
        ik.set_end_bone(0, 5);
        ik.set_extend_end_bone(0, true);
        ik.set_end_bone_direction(0, 2);
        ik.set_end_bone_length(0, 0.4);

        assert_eq!(ik.get_max_iterations(), 12);
        assert_eq!(ik.get_min_distance(), 0.005);
        assert_eq!(ik.get_root_bone_name(0), "Hips");
        assert_eq!(ik.get_root_bone(0), 1);
        assert_eq!(ik.get_end_bone_name(0), "Head");
        assert_eq!(ik.get_end_bone(0), 5);
        assert!(ik.is_end_bone_extended(0));
        assert_eq!(ik.get_end_bone_direction(0), 2);
        assert_eq!(ik.get_end_bone_length(0), 0.4);

        ik.set_joint_count(0, 3);
        assert_eq!(ik.get_joint_count(0), 3);
        for joint in 0..3 {
            assert_eq!(ik.get_joint_bone(0, joint), -1);
            assert_eq!(ik.get_joint_rotation_axis(0, joint), ROTATION_AXIS_ALL);
        }

        ik.set_joint_rotation_axis(0, 1, ROTATION_AXIS_CUSTOM);
        ik.set_joint_rotation_axis_vector(0, 1, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(ik.get_joint_rotation_axis(0, 1), ROTATION_AXIS_CUSTOM);
        assert_eq!(
            ik.get_joint_rotation_axis_vector(0, 1),
            Vector3::new(0.0, 1.0, 0.0)
        );
        ik.set_joint_rotation_axis(0, 1, ROTATION_AXIS_Z);
        assert_eq!(
            ik.get_joint_rotation_axis_vector(0, 1),
            Vector3::new(0.0, 0.0, 1.0)
        );
        ik.set_joint_limitation(0, 1, Some(Rid::from_u64(8)));
        assert_eq!(ik.get_joint_limitation(0, 1), Some(Rid::from_u64(8)));
        ik.set_joint_limitation_right_axis(0, 1, 2);
        assert_eq!(ik.get_joint_limitation_right_axis(0, 1), 2);

        // Joint count shrinks both arrays.
        ik.set_joint_count(0, 1);
        assert_eq!(ik.get_joint_count(0), 1);
        assert_eq!(ik.get_joint_limitation(0, 1), None);
    }

    #[test]
    fn target_node_and_guards() {
        let mut ik = ChainIk3DData::new();
        ik.set_setting_count(2);
        let path = NodePath::parse("/root/Target").expect("path");
        ik.set_target_node(0, path.clone());
        assert_eq!(ik.get_target_node(0), path);
        assert!(ik.get_target_node(1).is_empty());

        // Out-of-range writes are ignored.
        ik.set_root_bone(5, 3);
        ik.set_target_node(5, path);
        ik.set_joint_count(5, 2);
        ik.set_joint_rotation_axis(0, 5, 1);
        assert_eq!(ik.get_setting_count(), 2);
        assert_eq!(ik.get_joint_count(0), 0);

        ik.clear_settings();
        assert_eq!(ik.get_setting_count(), 0);
        assert_eq!(ROTATION_AXIS_CUSTOM, 4);
        assert_eq!(SECONDARY_DIRECTION_CUSTOM, 7);
    }
}
