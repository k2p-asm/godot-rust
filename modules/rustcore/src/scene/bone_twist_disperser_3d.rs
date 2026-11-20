//! `scene/3d/bone_twist_disperser_3d.*` (state): the twist-disperser modifier.
//!
//! Ported configuration: the mutable-bone-axes flag (on), the per-setting
//! disperse mode (`EVEN`), twist-from-rest (on) and twist quaternion, root/end
//! bones, joint list (bone name/index and custom amount), end-bone extension and
//! direction (`FROM_PARENT`), weight position (0.5) and damping curve handle.
//!
//! The twist distribution and skeleton processing are not ported.

use crate::math::quaternion::Quaternion;
use crate::servers::rid::Rid;

pub const BONE_DIRECTION_PLUS_X: i32 = 0;
pub const BONE_DIRECTION_MINUS_X: i32 = 1;
pub const BONE_DIRECTION_PLUS_Y: i32 = 2;
pub const BONE_DIRECTION_MINUS_Y: i32 = 3;
pub const BONE_DIRECTION_PLUS_Z: i32 = 4;
pub const BONE_DIRECTION_MINUS_Z: i32 = 5;
pub const BONE_DIRECTION_FROM_PARENT: i32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DisperseMode {
    Even = 0,
    Weighted = 1,
    Custom = 2,
}

#[derive(Clone, Debug)]
pub struct BoneJointState {
    pub name: String,
    pub bone: i32,
}

impl Default for BoneJointState {
    fn default() -> Self {
        Self {
            name: String::new(),
            bone: -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisperseJoint {
    pub joint: BoneJointState,
    pub custom_amount: f32,
}

impl Default for DisperseJoint {
    fn default() -> Self {
        Self {
            joint: BoneJointState::default(),
            custom_amount: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoneTwistDisperserSetting {
    pub disperse_mode: DisperseMode,
    pub twist_from_rest: bool,
    pub twist_from: Quaternion,
    pub root_bone: BoneJointState,
    pub end_bone: BoneJointState,
    pub joints: Vec<DisperseJoint>,
    pub extend_end_bone: bool,
    pub end_bone_direction: i32,
    pub weight_position: f32,
    pub damping_curve: Option<Rid>,
}

impl Default for BoneTwistDisperserSetting {
    fn default() -> Self {
        Self {
            disperse_mode: DisperseMode::Even,
            twist_from_rest: true,
            twist_from: Quaternion::default(),
            root_bone: BoneJointState::default(),
            end_bone: BoneJointState::default(),
            joints: Vec::new(),
            extend_end_bone: false,
            end_bone_direction: BONE_DIRECTION_FROM_PARENT,
            weight_position: 0.5,
            damping_curve: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoneTwistDisperser3DData {
    mutable_bone_axes: bool,
    settings: Vec<BoneTwistDisperserSetting>,
}

impl Default for BoneTwistDisperser3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl BoneTwistDisperser3DData {
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
        self.settings
            .resize_with(count, BoneTwistDisperserSetting::default);
    }

    pub fn clear_settings(&mut self) {
        self.settings.clear();
    }

    pub fn get_setting(&self, index: usize) -> Option<&BoneTwistDisperserSetting> {
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

    pub fn set_twist_from_rest(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.twist_from_rest = enabled;
        }
    }

    pub fn is_twist_from_rest(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.twist_from_rest)
            .unwrap_or(true)
    }

    pub fn set_twist_from(&mut self, index: usize, value: Quaternion) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.twist_from = value;
        }
    }

    pub fn get_twist_from(&self, index: usize) -> Quaternion {
        self.settings
            .get(index)
            .map(|setting| setting.twist_from)
            .unwrap_or_default()
    }

    pub fn set_disperse_mode(&mut self, index: usize, mode: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.disperse_mode = match mode {
                0 => DisperseMode::Even,
                1 => DisperseMode::Weighted,
                2 => DisperseMode::Custom,
                _ => setting.disperse_mode,
            };
        }
    }

    pub fn get_disperse_mode(&self, index: usize) -> DisperseMode {
        self.settings
            .get(index)
            .map(|setting| setting.disperse_mode)
            .unwrap_or(DisperseMode::Even)
    }

    pub fn set_weight_position(&mut self, index: usize, position: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.weight_position = position;
        }
    }

    pub fn get_weight_position(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.weight_position)
            .unwrap_or(0.5)
    }

    pub fn set_damping_curve(&mut self, index: usize, curve: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.damping_curve = curve;
        }
    }

    pub fn get_damping_curve(&self, index: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.damping_curve)
    }

    pub fn set_joint_count(&mut self, index: usize, count: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.joints.resize_with(count, DisperseJoint::default);
        }
    }

    pub fn get_joint_count(&self, index: usize) -> usize {
        self.settings
            .get(index)
            .map(|setting| setting.joints.len())
            .unwrap_or(0)
    }

    pub fn get_joint_bone_name(&self, index: usize, joint: usize) -> &str {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.joint.name.as_str())
            .unwrap_or("")
    }

    pub fn get_joint_bone(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.joint.bone)
            .unwrap_or(-1)
    }

    pub fn set_joint_twist_amount(&mut self, index: usize, joint: usize, amount: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.custom_amount = amount;
            }
        }
    }

    pub fn get_joint_twist_amount(&self, index: usize, joint: usize) -> f32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.custom_amount)
            .unwrap_or(1.0)
    }

    /// `get_reference_bone_name`: the reference bone is resolved from the
    /// skeleton in the engine; without one the port reports an empty name.
    pub fn get_reference_bone_name(&self, index: usize) -> &str {
        let _ = index;
        ""
    }

    pub fn get_reference_bone(&self, index: usize) -> i32 {
        let _ = index;
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn defaults() {
        let mut disperser = BoneTwistDisperser3DData::new();
        assert!(disperser.are_bone_axes_mutable());
        assert_eq!(disperser.get_setting_count(), 0);
        disperser.set_setting_count(1);
        assert_eq!(disperser.get_setting_count(), 1);
        assert_eq!(disperser.get_root_bone_name(0), "");
        assert_eq!(disperser.get_root_bone(0), -1);
        assert_eq!(disperser.get_end_bone_name(0), "");
        assert_eq!(disperser.get_end_bone(0), -1);
        assert!(!disperser.is_end_bone_extended(0));
        assert_eq!(
            disperser.get_end_bone_direction(0),
            BONE_DIRECTION_FROM_PARENT
        );
        assert!(disperser.is_twist_from_rest(0));
        assert_eq!(disperser.get_twist_from(0), Quaternion::default());
        assert_eq!(disperser.get_disperse_mode(0), DisperseMode::Even);
        assert_eq!(disperser.get_weight_position(0), 0.5);
        assert!(disperser.get_damping_curve(0).is_none());
        assert_eq!(disperser.get_joint_count(0), 0);
        assert_eq!(disperser.get_reference_bone(0), -1);
        assert_eq!(disperser.get_reference_bone_name(0), "");
        assert_eq!(BONE_DIRECTION_PLUS_X, 0);
        assert_eq!(BONE_DIRECTION_FROM_PARENT, 6);
    }

    #[test]
    fn setting_accessors() {
        let mut disperser = BoneTwistDisperser3DData::new();
        disperser.set_setting_count(2);
        disperser.set_mutable_bone_axes(false);
        assert!(!disperser.are_bone_axes_mutable());

        disperser.set_root_bone_name(0, "Hips");
        disperser.set_root_bone(0, 1);
        disperser.set_end_bone_name(0, "Chest");
        disperser.set_end_bone(0, 3);
        disperser.set_extend_end_bone(0, true);
        disperser.set_end_bone_direction(0, BONE_DIRECTION_MINUS_Y);
        disperser.set_twist_from_rest(0, false);
        let twist = Quaternion::from_axis_angle(Vector3::UP, 0.5);
        disperser.set_twist_from(0, twist);
        disperser.set_disperse_mode(0, 2);
        disperser.set_weight_position(0, 0.25);
        disperser.set_damping_curve(0, Some(Rid::from_u64(4)));

        assert_eq!(disperser.get_root_bone_name(0), "Hips");
        assert_eq!(disperser.get_root_bone(0), 1);
        assert_eq!(disperser.get_end_bone_name(0), "Chest");
        assert_eq!(disperser.get_end_bone(0), 3);
        assert!(disperser.is_end_bone_extended(0));
        assert_eq!(disperser.get_end_bone_direction(0), BONE_DIRECTION_MINUS_Y);
        assert!(!disperser.is_twist_from_rest(0));
        assert_eq!(disperser.get_twist_from(0), twist);
        assert_eq!(disperser.get_disperse_mode(0), DisperseMode::Custom);
        assert_eq!(disperser.get_weight_position(0), 0.25);
        assert_eq!(disperser.get_damping_curve(0), Some(Rid::from_u64(4)));

        // Invalid modes keep the previous value; other settings are untouched.
        disperser.set_disperse_mode(0, 9);
        assert_eq!(disperser.get_disperse_mode(0), DisperseMode::Custom);
        assert_eq!(disperser.get_root_bone_name(1), "");
        assert_eq!(disperser.get_disperse_mode(5), DisperseMode::Even);
        assert_eq!(disperser.get_weight_position(5), 0.5);
    }

    #[test]
    fn joints() {
        let mut disperser = BoneTwistDisperser3DData::new();
        disperser.set_setting_count(1);
        disperser.set_joint_count(0, 2);
        assert_eq!(disperser.get_joint_count(0), 2);
        assert_eq!(disperser.get_joint_bone_name(0, 0), "");
        assert_eq!(disperser.get_joint_bone(0, 0), -1);
        assert_eq!(disperser.get_joint_twist_amount(0, 0), 1.0);
        assert_eq!(disperser.get_joint_bone(0, 9), -1);

        disperser.set_joint_twist_amount(0, 1, 0.5);
        assert_eq!(disperser.get_joint_twist_amount(0, 1), 0.5);
        disperser.set_joint_twist_amount(0, 9, 2.0); // ignored

        // Resizing keeps the existing joints.
        disperser.set_joint_count(0, 1);
        assert_eq!(disperser.get_joint_count(0), 1);
        assert_eq!(disperser.get_joint_twist_amount(0, 0), 1.0);
        // The setting resets.
        disperser.set_setting_count(0);
        assert_eq!(disperser.get_setting_count(), 0);
    }

    #[test]
    fn clear_and_enum_values() {
        let mut disperser = BoneTwistDisperser3DData::new();
        disperser.set_setting_count(3);
        disperser.clear_settings();
        assert_eq!(disperser.get_setting_count(), 0);
        assert_eq!(DisperseMode::Even as i32, 0);
        assert_eq!(DisperseMode::Weighted as i32, 1);
        assert_eq!(DisperseMode::Custom as i32, 2);
        let setting = BoneTwistDisperserSetting::default();
        assert_eq!(setting.end_bone.bone, -1);
        assert_eq!(setting.joints.len(), 0);
    }
}
