//! `scene/3d/skeleton_ik_3d.*` and `scene/3d/look_at_modifier_3d.*` (state only).
//!
//! `SkeletonIK3D` defaults: empty root/tip bone names, identity target transform,
//! no target node, tip-basis override on, magnet off with a zero position,
//! minimum distance 0.01, ten iterations and not running.
//!
//! `LookAtModifier3D` defaults: no bone, zero forward vector, `PLUS_Z` forward
//! axis, Y primary / X secondary axes with the secondary rotation enabled,
//! relative off, `ORIGIN_FROM_SELF`, zero origin offset, 0.1 safe margin, no
//! target node, zero duration with linear/in easing, angle limitation off but
//! symmetry on, and `TAU`/`PI` limits with 1.0 damp thresholds for both axes.
//!
//! The IK solver and the look-at rotation math (`look_at_with_axes`,
//! `remap_damped`, B-spline evaluation, transition timing) are not ported.

use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

// -- SkeletonIK3D -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct SkeletonIK3DData {
    root_bone: String,
    tip_bone: String,
    target_transform: Transform3D,
    target_node: String,
    override_tip_basis: bool,
    use_magnet: bool,
    magnet_position: Vector3,
    min_distance: f32,
    max_iterations: i32,
    active: bool,
    internal_active: bool,
}

impl Default for SkeletonIK3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonIK3DData {
    pub fn new() -> Self {
        Self {
            root_bone: String::new(),
            tip_bone: String::new(),
            target_transform: Transform3D::IDENTITY,
            target_node: String::new(),
            override_tip_basis: true,
            use_magnet: false,
            magnet_position: Vector3::ZERO,
            min_distance: 0.01,
            max_iterations: 10,
            active: false,
            internal_active: false,
        }
    }

    pub fn set_root_bone(&mut self, bone: &str) {
        self.root_bone = bone.to_string();
    }

    pub fn get_root_bone(&self) -> &str {
        &self.root_bone
    }

    pub fn set_tip_bone(&mut self, bone: &str) {
        self.tip_bone = bone.to_string();
    }

    pub fn get_tip_bone(&self) -> &str {
        &self.tip_bone
    }

    pub fn set_target_transform(&mut self, transform: Transform3D) {
        self.target_transform = transform;
    }

    pub fn get_target_transform(&self) -> Transform3D {
        self.target_transform
    }

    pub fn set_target_node(&mut self, node: &str) {
        self.target_node = node.to_string();
    }

    pub fn get_target_node(&self) -> &str {
        &self.target_node
    }

    pub fn set_override_tip_basis(&mut self, enabled: bool) {
        self.override_tip_basis = enabled;
    }

    pub fn is_override_tip_basis(&self) -> bool {
        self.override_tip_basis
    }

    pub fn set_use_magnet(&mut self, enabled: bool) {
        self.use_magnet = enabled;
    }

    pub fn is_using_magnet(&self) -> bool {
        self.use_magnet
    }

    pub fn set_magnet_position(&mut self, position: Vector3) {
        self.magnet_position = position;
    }

    pub fn get_magnet_position(&self) -> Vector3 {
        self.magnet_position
    }

    pub fn set_min_distance(&mut self, distance: f32) {
        self.min_distance = distance;
    }

    pub fn get_min_distance(&self) -> f32 {
        self.min_distance
    }

    pub fn set_max_iterations(&mut self, iterations: i32) {
        self.max_iterations = iterations;
    }

    pub fn get_max_iterations(&self) -> i32 {
        self.max_iterations
    }

    /// `start`.
    pub fn start(&mut self) {
        self.active = true;
    }

    /// `stop`.
    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn is_running(&self) -> bool {
        self.active
    }

    pub fn set_internal_active(&mut self, active: bool) {
        self.internal_active = active;
    }

    pub fn is_internal_active(&self) -> bool {
        self.internal_active
    }
}

// -- LookAtModifier3D -------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum OriginFrom {
    SelfBone,
    SpecificBone,
    ExternalNode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BoneAxis {
    PlusX,
    PlusY,
    PlusZ,
    MinusX,
    MinusY,
    MinusZ,
}

#[derive(Clone, Debug)]
pub struct LookAtModifier3DData {
    bone_name: String,
    bone: i32,
    forward_vector: Vector3,
    forward_axis: BoneAxis,
    primary_rotation_axis: i32,
    secondary_rotation_axis: i32,
    use_secondary_rotation: bool,
    relative: bool,
    origin_from: OriginFrom,
    origin_bone_name: String,
    origin_bone: i32,
    origin_external_node: String,
    origin_offset: Vector3,
    origin_safe_margin: f32,
    target_node: String,
    duration: f32,
    transition_type: i32,
    ease_type: i32,
    use_angle_limitation: bool,
    symmetry_limitation: bool,
    primary_limit_angle: f32,
    primary_damp_threshold: f32,
    primary_positive_limit_angle: f32,
    primary_positive_damp_threshold: f32,
    primary_negative_limit_angle: f32,
    primary_negative_damp_threshold: f32,
    secondary_limit_angle: f32,
    secondary_damp_threshold: f32,
    secondary_positive_limit_angle: f32,
    secondary_positive_damp_threshold: f32,
    secondary_negative_limit_angle: f32,
    secondary_negative_damp_threshold: f32,
    is_within_limitations: bool,
}

impl Default for LookAtModifier3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl LookAtModifier3DData {
    pub fn new() -> Self {
        use std::f32::consts::{PI, TAU};
        Self {
            bone_name: String::new(),
            bone: -1,
            forward_vector: Vector3::ZERO,
            forward_axis: BoneAxis::PlusZ,
            primary_rotation_axis: 1, // Y
            secondary_rotation_axis: 0, // X
            use_secondary_rotation: true,
            relative: false,
            origin_from: OriginFrom::SelfBone,
            origin_bone_name: String::new(),
            origin_bone: -1,
            origin_external_node: String::new(),
            origin_offset: Vector3::ZERO,
            origin_safe_margin: 0.1,
            target_node: String::new(),
            duration: 0.0,
            transition_type: 0,
            ease_type: 0,
            use_angle_limitation: false,
            symmetry_limitation: true,
            primary_limit_angle: TAU,
            primary_damp_threshold: 1.0,
            primary_positive_limit_angle: PI,
            primary_positive_damp_threshold: 1.0,
            primary_negative_limit_angle: PI,
            primary_negative_damp_threshold: 1.0,
            secondary_limit_angle: TAU,
            secondary_damp_threshold: 1.0,
            secondary_positive_limit_angle: PI,
            secondary_positive_damp_threshold: 1.0,
            secondary_negative_limit_angle: PI,
            secondary_negative_damp_threshold: 1.0,
            is_within_limitations: false,
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

    pub fn set_forward_vector(&mut self, vector: Vector3) {
        self.forward_vector = vector;
    }

    pub fn get_forward_vector(&self) -> Vector3 {
        self.forward_vector
    }

    pub fn set_forward_axis(&mut self, axis: BoneAxis) {
        self.forward_axis = axis;
    }

    pub fn get_forward_axis(&self) -> BoneAxis {
        self.forward_axis
    }

    /// `Vector3::Axis`: 0 = X, 1 = Y, 2 = Z.
    pub fn set_primary_rotation_axis(&mut self, axis: i32) -> bool {
        if !(0..=2).contains(&axis) {
            return false;
        }
        self.primary_rotation_axis = axis;
        true
    }

    pub fn get_primary_rotation_axis(&self) -> i32 {
        self.primary_rotation_axis
    }

    pub fn set_secondary_rotation_axis(&mut self, axis: i32) -> bool {
        if !(0..=2).contains(&axis) {
            return false;
        }
        self.secondary_rotation_axis = axis;
        true
    }

    pub fn get_secondary_rotation_axis(&self) -> i32 {
        self.secondary_rotation_axis
    }

    pub fn set_use_secondary_rotation(&mut self, enabled: bool) {
        self.use_secondary_rotation = enabled;
    }

    pub fn is_using_secondary_rotation(&self) -> bool {
        self.use_secondary_rotation
    }

    pub fn set_relative(&mut self, enabled: bool) {
        self.relative = enabled;
    }

    pub fn is_relative(&self) -> bool {
        self.relative
    }

    pub fn set_origin_from(&mut self, origin: i32) -> bool {
        let parsed = match origin {
            0 => OriginFrom::SelfBone,
            1 => OriginFrom::SpecificBone,
            2 => OriginFrom::ExternalNode,
            _ => return false,
        };
        self.origin_from = parsed;
        true
    }

    pub fn get_origin_from(&self) -> OriginFrom {
        self.origin_from
    }

    pub fn set_origin_bone_name(&mut self, name: &str) {
        self.origin_bone_name = name.to_string();
    }

    pub fn get_origin_bone_name(&self) -> &str {
        &self.origin_bone_name
    }

    pub fn set_origin_bone(&mut self, bone: i32) {
        self.origin_bone = bone;
    }

    pub fn get_origin_bone(&self) -> i32 {
        self.origin_bone
    }

    pub fn set_origin_external_node(&mut self, node: &str) {
        self.origin_external_node = node.to_string();
    }

    pub fn get_origin_external_node(&self) -> &str {
        &self.origin_external_node
    }

    pub fn set_origin_offset(&mut self, offset: Vector3) {
        self.origin_offset = offset;
    }

    pub fn get_origin_offset(&self) -> Vector3 {
        self.origin_offset
    }

    pub fn set_origin_safe_margin(&mut self, margin: f32) {
        self.origin_safe_margin = margin;
    }

    pub fn get_origin_safe_margin(&self) -> f32 {
        self.origin_safe_margin
    }

    pub fn set_target_node(&mut self, node: &str) {
        self.target_node = node.to_string();
    }

    pub fn get_target_node(&self) -> &str {
        &self.target_node
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }

    pub fn get_duration(&self) -> f32 {
        self.duration
    }

    pub fn set_transition_type(&mut self, transition: i32) {
        self.transition_type = transition;
    }

    pub fn get_transition_type(&self) -> i32 {
        self.transition_type
    }

    pub fn set_ease_type(&mut self, ease: i32) {
        self.ease_type = ease;
    }

    pub fn get_ease_type(&self) -> i32 {
        self.ease_type
    }

    pub fn set_use_angle_limitation(&mut self, enabled: bool) {
        self.use_angle_limitation = enabled;
    }

    pub fn is_using_angle_limitation(&self) -> bool {
        self.use_angle_limitation
    }

    pub fn set_symmetry_limitation(&mut self, enabled: bool) {
        self.symmetry_limitation = enabled;
    }

    pub fn is_limitation_symmetry(&self) -> bool {
        self.symmetry_limitation
    }

    pub fn set_primary_limit_angle(&mut self, angle: f32) {
        self.primary_limit_angle = angle;
    }

    pub fn get_primary_limit_angle(&self) -> f32 {
        self.primary_limit_angle
    }

    pub fn set_primary_damp_threshold(&mut self, threshold: f32) {
        self.primary_damp_threshold = threshold;
    }

    pub fn get_primary_damp_threshold(&self) -> f32 {
        self.primary_damp_threshold
    }

    pub fn set_primary_positive_limit_angle(&mut self, angle: f32) {
        self.primary_positive_limit_angle = angle;
    }

    pub fn get_primary_positive_limit_angle(&self) -> f32 {
        self.primary_positive_limit_angle
    }

    pub fn set_primary_positive_damp_threshold(&mut self, threshold: f32) {
        self.primary_positive_damp_threshold = threshold;
    }

    pub fn get_primary_positive_damp_threshold(&self) -> f32 {
        self.primary_positive_damp_threshold
    }

    pub fn set_primary_negative_limit_angle(&mut self, angle: f32) {
        self.primary_negative_limit_angle = angle;
    }

    pub fn get_primary_negative_limit_angle(&self) -> f32 {
        self.primary_negative_limit_angle
    }

    pub fn set_primary_negative_damp_threshold(&mut self, threshold: f32) {
        self.primary_negative_damp_threshold = threshold;
    }

    pub fn get_primary_negative_damp_threshold(&self) -> f32 {
        self.primary_negative_damp_threshold
    }

    pub fn set_secondary_limit_angle(&mut self, angle: f32) {
        self.secondary_limit_angle = angle;
    }

    pub fn get_secondary_limit_angle(&self) -> f32 {
        self.secondary_limit_angle
    }

    pub fn set_secondary_damp_threshold(&mut self, threshold: f32) {
        self.secondary_damp_threshold = threshold;
    }

    pub fn get_secondary_damp_threshold(&self) -> f32 {
        self.secondary_damp_threshold
    }

    pub fn set_secondary_positive_limit_angle(&mut self, angle: f32) {
        self.secondary_positive_limit_angle = angle;
    }

    pub fn get_secondary_positive_limit_angle(&self) -> f32 {
        self.secondary_positive_limit_angle
    }

    pub fn set_secondary_positive_damp_threshold(&mut self, threshold: f32) {
        self.secondary_positive_damp_threshold = threshold;
    }

    pub fn get_secondary_positive_damp_threshold(&self) -> f32 {
        self.secondary_positive_damp_threshold
    }

    pub fn set_secondary_negative_limit_angle(&mut self, angle: f32) {
        self.secondary_negative_limit_angle = angle;
    }

    pub fn get_secondary_negative_limit_angle(&self) -> f32 {
        self.secondary_negative_limit_angle
    }

    pub fn set_secondary_negative_damp_threshold(&mut self, threshold: f32) {
        self.secondary_negative_damp_threshold = threshold;
    }

    pub fn get_secondary_negative_damp_threshold(&self) -> f32 {
        self.secondary_negative_damp_threshold
    }

    pub fn is_within_limitations(&self) -> bool {
        self.is_within_limitations
    }

    pub fn set_within_limitations(&mut self, within: bool) {
        self.is_within_limitations = within;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_ik_defaults_and_state() {
        let mut ik = SkeletonIK3DData::new();
        assert_eq!(ik.get_root_bone(), "");
        assert_eq!(ik.get_tip_bone(), "");
        assert_eq!(ik.get_target_transform(), Transform3D::IDENTITY);
        assert_eq!(ik.get_target_node(), "");
        assert!(ik.is_override_tip_basis());
        assert!(!ik.is_using_magnet());
        assert_eq!(ik.get_magnet_position(), Vector3::ZERO);
        assert_eq!(ik.get_min_distance(), 0.01);
        assert_eq!(ik.get_max_iterations(), 10);
        assert!(!ik.is_running());

        ik.set_root_bone("Hips");
        ik.set_tip_bone("Hand");
        ik.set_target_node("../Target");
        ik.set_override_tip_basis(false);
        ik.set_use_magnet(true);
        ik.set_magnet_position(Vector3::new(1.0, 2.0, 3.0));
        ik.set_min_distance(0.05);
        ik.set_max_iterations(20);
        ik.start();
        assert_eq!(ik.get_root_bone(), "Hips");
        assert_eq!(ik.get_tip_bone(), "Hand");
        assert_eq!(ik.get_target_node(), "../Target");
        assert!(!ik.is_override_tip_basis());
        assert!(ik.is_using_magnet());
        assert_eq!(ik.get_magnet_position(), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(ik.get_min_distance(), 0.05);
        assert_eq!(ik.get_max_iterations(), 20);
        assert!(ik.is_running());
        ik.stop();
        assert!(!ik.is_running());

        let transform = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::ONE,
        };
        ik.set_target_transform(transform);
        assert_eq!(ik.get_target_transform().origin, Vector3::ONE);
        ik.set_internal_active(true);
        assert!(ik.is_internal_active());
    }

    #[test]
    fn look_at_defaults_and_setters() {
        use std::f32::consts::{PI, TAU};
        let mut look = LookAtModifier3DData::new();
        assert_eq!(look.get_bone_name(), "");
        assert_eq!(look.get_bone(), -1);
        assert_eq!(look.get_forward_vector(), Vector3::ZERO);
        assert_eq!(look.get_forward_axis(), BoneAxis::PlusZ);
        assert_eq!(look.get_primary_rotation_axis(), 1);
        assert_eq!(look.get_secondary_rotation_axis(), 0);
        assert!(look.is_using_secondary_rotation());
        assert!(!look.is_relative());
        assert_eq!(look.get_origin_from(), OriginFrom::SelfBone);
        assert_eq!(look.get_origin_bone_name(), "");
        assert_eq!(look.get_origin_bone(), -1);
        assert_eq!(look.get_origin_external_node(), "");
        assert_eq!(look.get_origin_offset(), Vector3::ZERO);
        assert_eq!(look.get_origin_safe_margin(), 0.1);
        assert_eq!(look.get_target_node(), "");
        assert_eq!(look.get_duration(), 0.0);
        assert_eq!(look.get_transition_type(), 0);
        assert_eq!(look.get_ease_type(), 0);
        assert!(!look.is_using_angle_limitation());
        assert!(look.is_limitation_symmetry());
        assert_eq!(look.get_primary_limit_angle(), TAU);
        assert_eq!(look.get_primary_damp_threshold(), 1.0);
        assert_eq!(look.get_primary_positive_limit_angle(), PI);
        assert_eq!(look.get_primary_positive_damp_threshold(), 1.0);
        assert_eq!(look.get_primary_negative_limit_angle(), PI);
        assert_eq!(look.get_primary_negative_damp_threshold(), 1.0);
        assert_eq!(look.get_secondary_limit_angle(), TAU);
        assert_eq!(look.get_secondary_damp_threshold(), 1.0);
        assert_eq!(look.get_secondary_positive_limit_angle(), PI);
        assert_eq!(look.get_secondary_positive_damp_threshold(), 1.0);
        assert_eq!(look.get_secondary_negative_limit_angle(), PI);
        assert_eq!(look.get_secondary_negative_damp_threshold(), 1.0);
        assert!(!look.is_within_limitations());

        look.set_bone_name("Head");
        look.set_bone(2);
        look.set_forward_vector(Vector3::new(0.0, 0.0, 1.0));
        look.set_forward_axis(BoneAxis::MinusZ);
        assert!(look.set_primary_rotation_axis(2));
        assert!(!look.set_primary_rotation_axis(3));
        assert!(look.set_secondary_rotation_axis(0));
        assert!(!look.set_secondary_rotation_axis(-1));
        look.set_use_secondary_rotation(false);
        look.set_relative(true);
        assert!(look.set_origin_from(2));
        assert!(!look.set_origin_from(3));
        look.set_origin_bone_name("Root");
        look.set_origin_bone(0);
        look.set_origin_external_node("../Cam");
        look.set_origin_offset(Vector3::new(1.0, 0.0, 0.0));
        look.set_origin_safe_margin(0.5);
        look.set_target_node("../Target");
        look.set_duration(0.25);
        look.set_transition_type(1);
        look.set_ease_type(2);
        look.set_use_angle_limitation(true);
        look.set_symmetry_limitation(false);
        look.set_primary_limit_angle(1.0);
        look.set_primary_damp_threshold(0.9);
        look.set_primary_positive_limit_angle(0.8);
        look.set_primary_positive_damp_threshold(0.7);
        look.set_primary_negative_limit_angle(-0.6);
        look.set_primary_negative_damp_threshold(0.5);
        look.set_secondary_limit_angle(0.4);
        look.set_secondary_damp_threshold(0.3);
        look.set_secondary_positive_limit_angle(0.2);
        look.set_secondary_positive_damp_threshold(0.15);
        look.set_secondary_negative_limit_angle(-0.1);
        look.set_secondary_negative_damp_threshold(0.05);
        look.set_within_limitations(true);

        assert_eq!(look.get_bone_name(), "Head");
        assert_eq!(look.get_bone(), 2);
        assert_eq!(look.get_forward_axis(), BoneAxis::MinusZ);
        assert_eq!(look.get_primary_rotation_axis(), 2);
        assert_eq!(look.get_secondary_rotation_axis(), 0);
        assert!(!look.is_using_secondary_rotation());
        assert!(look.is_relative());
        assert_eq!(look.get_origin_from(), OriginFrom::ExternalNode);
        assert_eq!(look.get_origin_bone_name(), "Root");
        assert_eq!(look.get_origin_bone(), 0);
        assert_eq!(look.get_origin_external_node(), "../Cam");
        assert_eq!(look.get_origin_offset(), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(look.get_origin_safe_margin(), 0.5);
        assert_eq!(look.get_target_node(), "../Target");
        assert_eq!(look.get_duration(), 0.25);
        assert_eq!(look.get_transition_type(), 1);
        assert_eq!(look.get_ease_type(), 2);
        assert!(look.is_using_angle_limitation());
        assert!(!look.is_limitation_symmetry());
        assert_eq!(look.get_primary_limit_angle(), 1.0);
        assert_eq!(look.get_primary_negative_damp_threshold(), 0.5);
        assert_eq!(look.get_secondary_limit_angle(), 0.4);
        assert_eq!(look.get_secondary_negative_damp_threshold(), 0.05);
        assert!(look.is_within_limitations());
        assert_eq!(OriginFrom::SelfBone as i32, 0);
        assert_eq!(OriginFrom::ExternalNode as i32, 2);
        assert_eq!(BoneAxis::PlusZ as i32, 2);
        assert_eq!(BoneAxis::MinusZ as i32, 5);
    }
}
