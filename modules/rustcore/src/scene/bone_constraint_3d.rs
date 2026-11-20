//! `scene/3d/bone_constraint_3d.*` and `limit_angular_velocity_modifier_3d.*`
//! (state): bone-constraint settings and the angular-velocity limiter.
//!
//! `BoneConstraint3D` keeps an array of settings (`amount`, apply bone
//! name/index, reference type/bone/node) with index-checked accessors and the
//! engine's defaults. `LimitAngularVelocityModifier3D` keeps the chain list
//! (root/end bone names and indices), the exclusion flag and the maximum angular
//! velocity (`TAU` by default).
//!
//! The skeleton processing and the constraint math are not ported.

use crate::string::node_path::NodePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ReferenceType {
    Bone = 0,
    Node = 1,
}

#[derive(Clone, Debug)]
pub struct BoneConstraintSetting {
    pub amount: f32,
    pub apply_bone_name: String,
    pub apply_bone: i32,
    pub reference_type: ReferenceType,
    pub reference_bone_name: String,
    pub reference_bone: i32,
    pub reference_node: NodePath,
}

impl Default for BoneConstraintSetting {
    fn default() -> Self {
        Self {
            amount: 1.0,
            apply_bone_name: String::new(),
            apply_bone: -1,
            reference_type: ReferenceType::Bone,
            reference_bone_name: String::new(),
            reference_bone: -1,
            reference_node: NodePath::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BoneConstraint3DData {
    settings: Vec<BoneConstraintSetting>,
}

impl BoneConstraint3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.settings.resize(count, BoneConstraintSetting::default());
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn get_setting(&self, index: usize) -> Option<&BoneConstraintSetting> {
        self.settings.get(index)
    }

    pub fn clear_settings(&mut self) {
        self.settings.clear();
    }

    pub fn set_amount(&mut self, index: usize, amount: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.amount = amount;
        }
    }

    pub fn get_amount(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.amount)
            .unwrap_or(0.0)
    }

    pub fn set_apply_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_bone_name = name.to_string();
        }
    }

    pub fn get_apply_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.apply_bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_apply_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_bone = bone;
        }
    }

    pub fn get_apply_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.apply_bone)
            .unwrap_or(-1)
    }

    pub fn set_reference_type(&mut self, index: usize, reference_type: ReferenceType) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_type = reference_type;
        }
    }

    pub fn get_reference_type(&self, index: usize) -> ReferenceType {
        self.settings
            .get(index)
            .map(|setting| setting.reference_type)
            .unwrap_or(ReferenceType::Bone)
    }

    pub fn set_reference_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_bone_name = name.to_string();
        }
    }

    pub fn get_reference_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.reference_bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_reference_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_bone = bone;
        }
    }

    pub fn get_reference_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.reference_bone)
            .unwrap_or(-1)
    }

    pub fn set_reference_node(&mut self, index: usize, node: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_node = node;
        }
    }

    pub fn get_reference_node(&self, index: usize) -> NodePath {
        self.settings
            .get(index)
            .map(|setting| setting.reference_node.clone())
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub struct BoneJoint {
    pub name: String,
    pub bone: i32,
}

impl Default for BoneJoint {
    fn default() -> Self {
        Self {
            name: String::new(),
            bone: -1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AngularVelocityChain {
    pub root_bone: BoneJoint,
    pub end_bone: BoneJoint,
}

#[derive(Clone, Debug)]
pub struct LimitAngularVelocityModifier3DData {
    exclude: bool,
    max_angular_velocity: f64,
    chains: Vec<AngularVelocityChain>,
}

impl Default for LimitAngularVelocityModifier3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitAngularVelocityModifier3DData {
    pub fn new() -> Self {
        Self {
            exclude: false,
            max_angular_velocity: std::f64::consts::TAU,
            chains: Vec::new(),
        }
    }

    pub fn set_chain_count(&mut self, count: usize) {
        self.chains.resize_with(count, AngularVelocityChain::default);
    }

    pub fn get_chain_count(&self) -> usize {
        self.chains.len()
    }

    pub fn clear_chains(&mut self) {
        self.chains.clear();
    }

    pub fn set_root_bone_name(&mut self, index: usize, name: &str) {
        if let Some(chain) = self.chains.get_mut(index) {
            chain.root_bone.name = name.to_string();
        }
    }

    pub fn get_root_bone_name(&self, index: usize) -> &str {
        self.chains
            .get(index)
            .map(|chain| chain.root_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_root_bone(&mut self, index: usize, bone: i32) {
        if let Some(chain) = self.chains.get_mut(index) {
            chain.root_bone.bone = bone;
        }
    }

    pub fn get_root_bone(&self, index: usize) -> i32 {
        self.chains
            .get(index)
            .map(|chain| chain.root_bone.bone)
            .unwrap_or(-1)
    }

    pub fn set_end_bone_name(&mut self, index: usize, name: &str) {
        if let Some(chain) = self.chains.get_mut(index) {
            chain.end_bone.name = name.to_string();
        }
    }

    pub fn get_end_bone_name(&self, index: usize) -> &str {
        self.chains
            .get(index)
            .map(|chain| chain.end_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_end_bone(&mut self, index: usize, bone: i32) {
        if let Some(chain) = self.chains.get_mut(index) {
            chain.end_bone.bone = bone;
        }
    }

    pub fn get_end_bone(&self, index: usize) -> i32 {
        self.chains
            .get(index)
            .map(|chain| chain.end_bone.bone)
            .unwrap_or(-1)
    }

    /// The joint bone is resolved from the skeleton in the engine; the port has
    /// no skeleton, so an out-of-range chain reports -1.
    pub fn get_joint_bone(&self, index: usize) -> i32 {
        self.chains
            .get(index)
            .map(|chain| {
                if chain.root_bone.bone >= 0 {
                    chain.root_bone.bone
                } else {
                    -1
                }
            })
            .unwrap_or(-1)
    }

    pub fn get_joint_bone_name(&self, index: usize) -> &str {
        self.chains
            .get(index)
            .map(|chain| chain.root_bone.name.as_str())
            .unwrap_or("")
    }

    pub fn set_max_angular_velocity(&mut self, velocity: f64) {
        self.max_angular_velocity = velocity;
    }

    pub fn get_max_angular_velocity(&self) -> f64 {
        self.max_angular_velocity
    }

    pub fn set_exclude(&mut self, exclude: bool) {
        self.exclude = exclude;
    }

    pub fn is_exclude(&self) -> bool {
        self.exclude
    }

    /// `reset`: clears the derived cache in the engine; the port keeps the
    /// configuration and only notes the call.
    pub fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bone_constraint_defaults() {
        let mut constraint = BoneConstraint3DData::new();
        assert_eq!(constraint.get_setting_count(), 0);
        constraint.set_setting_count(2);
        assert_eq!(constraint.get_setting_count(), 2);
        assert_eq!(constraint.get_amount(0), 1.0);
        assert_eq!(constraint.get_apply_bone_name(0), "");
        assert_eq!(constraint.get_apply_bone(0), -1);
        assert_eq!(constraint.get_reference_type(0), ReferenceType::Bone);
        assert_eq!(constraint.get_reference_bone_name(0), "");
        assert_eq!(constraint.get_reference_bone(0), -1);
        assert!(constraint.get_reference_node(0).is_empty());
        // Out-of-range accessors fall back to the engine's sentinels.
        assert_eq!(constraint.get_amount(9), 0.0);
        assert_eq!(constraint.get_apply_bone(9), -1);
    }

    #[test]
    fn bone_constraint_setters() {
        let mut constraint = BoneConstraint3DData::new();
        constraint.set_setting_count(1);
        constraint.set_amount(0, 0.5);
        constraint.set_apply_bone_name(0, "Hand.L");
        constraint.set_apply_bone(0, 4);
        constraint.set_reference_type(0, ReferenceType::Node);
        constraint.set_reference_bone_name(0, "Head");
        constraint.set_reference_bone(0, -2);
        let path = NodePath::parse("/root/Marker").expect("path");
        constraint.set_reference_node(0, path.clone());

        assert_eq!(constraint.get_amount(0), 0.5);
        assert_eq!(constraint.get_apply_bone_name(0), "Hand.L");
        assert_eq!(constraint.get_apply_bone(0), 4);
        assert_eq!(constraint.get_reference_type(0), ReferenceType::Node);
        assert_eq!(constraint.get_reference_bone_name(0), "Head");
        assert_eq!(constraint.get_reference_bone(0), -2);
        assert_eq!(constraint.get_reference_node(0), path);
        assert_eq!(ReferenceType::Bone as i32, 0);
        assert_eq!(ReferenceType::Node as i32, 1);

        // Index checks: out-of-range writes are ignored.
        constraint.set_amount(3, 2.0);
        constraint.set_apply_bone_name(3, "x");
        assert!(constraint.get_setting(3).is_none());

        constraint.clear_settings();
        assert_eq!(constraint.get_setting_count(), 0);
    }

    #[test]
    fn angular_velocity_defaults() {
        let mut modifier = LimitAngularVelocityModifier3DData::new();
        assert!(!modifier.is_exclude());
        assert!((modifier.get_max_angular_velocity() - std::f64::consts::TAU).abs() < 1e-12);
        assert_eq!(modifier.get_chain_count(), 0);
        modifier.set_exclude(true);
        modifier.set_max_angular_velocity(3.0);
        assert!(modifier.is_exclude());
        assert_eq!(modifier.get_max_angular_velocity(), 3.0);
        modifier.reset();
        assert!(modifier.is_exclude());
    }

    #[test]
    fn angular_velocity_chains() {
        let mut modifier = LimitAngularVelocityModifier3DData::new();
        modifier.set_chain_count(2);
        assert_eq!(modifier.get_chain_count(), 2);
        assert_eq!(modifier.get_root_bone_name(0), "");
        assert_eq!(modifier.get_root_bone(0), -1);
        assert_eq!(modifier.get_end_bone_name(0), "");
        assert_eq!(modifier.get_end_bone(0), -1);
        assert_eq!(modifier.get_joint_bone(9), -1);

        modifier.set_root_bone_name(0, "Hips");
        modifier.set_root_bone(0, 1);
        modifier.set_end_bone_name(0, "Hand.L");
        modifier.set_end_bone(0, 5);
        assert_eq!(modifier.get_root_bone_name(0), "Hips");
        assert_eq!(modifier.get_root_bone(0), 1);
        assert_eq!(modifier.get_end_bone_name(0), "Hand.L");
        assert_eq!(modifier.get_end_bone(0), 5);
        assert_eq!(modifier.get_joint_bone(0), 1);
        assert_eq!(modifier.get_joint_bone_name(0), "Hips");

        // Shrinking and clearing.
        modifier.set_chain_count(1);
        assert_eq!(modifier.get_chain_count(), 1);
        assert_eq!(modifier.get_end_bone(0), 5);
        modifier.clear_chains();
        assert_eq!(modifier.get_chain_count(), 0);
        assert_eq!(modifier.get_root_bone_name(0), "");
    }
}
