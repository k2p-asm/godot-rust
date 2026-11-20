//! `scene/3d/physics/physical_bone_3d.*`: bone physics state and joint data.
//!
//! Defaults: no joint (`JOINT_TYPE_NONE`), identity offsets, simulation off, no
//! bone, bounce 0, mass 1, friction 1, zero velocities, gravity scale 1,
//! `can_sleep` on, custom integrator off and both damp modes `COMBINE` with zero
//! damping.
//!
//! Joint data records (per type) keep the engine's constructor defaults:
//! Pin 0.3/1/0; Cone π/4 swing, π twist, 0.3 bias, 0.8 softness, 1 relaxation;
//! Hinge limit off with ±π/2 limits, 0.3 bias, 0.9 softness, 1 relaxation;
//! Slider 1/-1 linear limits with 1/0.7/1 softness/restitution/damping and zero
//! angular limits. `JOINT_TYPE_6DOF` is exposed but its three axis tables are not
//! ported (documented pending).
//!
//! The skeleton attachment, physics server bodies and force integration are not
//! ported.

use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DampMode {
    Combine,
    Replace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum JointType {
    None,
    Pin,
    Cone,
    Hinge,
    Slider,
    SixDof,
}

#[derive(Clone, Copy, Debug)]
pub struct PinJointData {
    pub bias: f32,
    pub damping: f32,
    pub impulse_clamp: f32,
}

impl Default for PinJointData {
    fn default() -> Self {
        Self {
            bias: 0.3,
            damping: 1.0,
            impulse_clamp: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConeJointData {
    pub swing_span: f32,
    pub twist_span: f32,
    pub bias: f32,
    pub softness: f32,
    pub relaxation: f32,
}

impl Default for ConeJointData {
    fn default() -> Self {
        Self {
            swing_span: std::f32::consts::PI * 0.25,
            twist_span: std::f32::consts::PI,
            bias: 0.3,
            softness: 0.8,
            relaxation: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HingeJointData {
    pub angular_limit_enabled: bool,
    pub angular_limit_upper: f32,
    pub angular_limit_lower: f32,
    pub angular_limit_bias: f32,
    pub angular_limit_softness: f32,
    pub angular_limit_relaxation: f32,
}

impl Default for HingeJointData {
    fn default() -> Self {
        Self {
            angular_limit_enabled: false,
            angular_limit_upper: std::f32::consts::PI * 0.5,
            angular_limit_lower: -std::f32::consts::PI * 0.5,
            angular_limit_bias: 0.3,
            angular_limit_softness: 0.9,
            angular_limit_relaxation: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SliderJointData {
    pub linear_limit_upper: f32,
    pub linear_limit_lower: f32,
    pub linear_limit_softness: f32,
    pub linear_limit_restitution: f32,
    pub linear_limit_damping: f32,
    pub angular_limit_upper: f32,
    pub angular_limit_lower: f32,
    pub angular_limit_softness: f32,
    pub angular_limit_restriction: f32,
    pub angular_limit_damping: f32,
}

impl Default for SliderJointData {
    fn default() -> Self {
        Self {
            linear_limit_upper: 1.0,
            linear_limit_lower: -1.0,
            linear_limit_softness: 1.0,
            linear_limit_restitution: 0.7,
            linear_limit_damping: 1.0,
            angular_limit_upper: 0.0,
            angular_limit_lower: 0.0,
            angular_limit_softness: 1.0,
            angular_limit_restriction: 0.7,
            angular_limit_damping: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalBone3DData {
    joint_type: JointType,
    pin: PinJointData,
    cone: ConeJointData,
    hinge: HingeJointData,
    slider: SliderJointData,
    joint_offset: Transform3D,
    body_offset: Transform3D,
    body_offset_inverse: Transform3D,
    simulate_physics: bool,
    internal_simulate_physics: bool,
    bone_id: i32,
    bone_name: String,
    bounce: f32,
    mass: f32,
    friction: f32,
    linear_velocity: Vector3,
    angular_velocity: Vector3,
    gravity_scale: f32,
    can_sleep: bool,
    custom_integrator: bool,
    linear_damp_mode: DampMode,
    angular_damp_mode: DampMode,
    linear_damp: f32,
    angular_damp: f32,
}

impl Default for PhysicalBone3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalBone3DData {
    pub fn new() -> Self {
        Self {
            joint_type: JointType::None,
            pin: PinJointData::default(),
            cone: ConeJointData::default(),
            hinge: HingeJointData::default(),
            slider: SliderJointData::default(),
            joint_offset: Transform3D::IDENTITY,
            body_offset: Transform3D::IDENTITY,
            body_offset_inverse: Transform3D::IDENTITY,
            simulate_physics: false,
            internal_simulate_physics: false,
            bone_id: -1,
            bone_name: String::new(),
            bounce: 0.0,
            mass: 1.0,
            friction: 1.0,
            linear_velocity: Vector3::ZERO,
            angular_velocity: Vector3::ZERO,
            gravity_scale: 1.0,
            can_sleep: true,
            custom_integrator: false,
            linear_damp_mode: DampMode::Combine,
            angular_damp_mode: DampMode::Combine,
            linear_damp: 0.0,
            angular_damp: 0.0,
        }
    }

    pub fn set_joint_type(&mut self, joint_type: JointType) {
        self.joint_type = joint_type;
    }

    pub fn get_joint_type(&self) -> JointType {
        self.joint_type
    }

    pub fn pin_data(&self) -> &PinJointData {
        &self.pin
    }

    pub fn pin_data_mut(&mut self) -> &mut PinJointData {
        &mut self.pin
    }

    pub fn cone_data(&self) -> &ConeJointData {
        &self.cone
    }

    pub fn cone_data_mut(&mut self) -> &mut ConeJointData {
        &mut self.cone
    }

    pub fn hinge_data(&self) -> &HingeJointData {
        &self.hinge
    }

    pub fn hinge_data_mut(&mut self) -> &mut HingeJointData {
        &mut self.hinge
    }

    pub fn slider_data(&self) -> &SliderJointData {
        &self.slider
    }

    pub fn slider_data_mut(&mut self) -> &mut SliderJointData {
        &mut self.slider
    }

    pub fn set_joint_offset(&mut self, offset: Transform3D) {
        self.joint_offset = offset;
    }

    pub fn get_joint_offset(&self) -> Transform3D {
        self.joint_offset
    }

    pub fn set_body_offset(&mut self, offset: Transform3D) {
        self.body_offset = offset;
        self.body_offset_inverse = offset.affine_inverse();
    }

    pub fn get_body_offset(&self) -> Transform3D {
        self.body_offset
    }

    pub fn get_body_offset_inverse(&self) -> Transform3D {
        self.body_offset_inverse
    }

    pub fn set_simulate_physics(&mut self, simulate: bool) {
        self.simulate_physics = simulate;
    }

    pub fn get_simulate_physics(&self) -> bool {
        self.simulate_physics
    }

    pub fn set_internal_simulate_physics(&mut self, simulate: bool) {
        self.internal_simulate_physics = simulate;
    }

    pub fn get_internal_simulate_physics(&self) -> bool {
        self.internal_simulate_physics
    }

    pub fn set_bone_id(&mut self, id: i32) {
        self.bone_id = id;
    }

    pub fn get_bone_id(&self) -> i32 {
        self.bone_id
    }

    pub fn set_bone_name(&mut self, name: &str) {
        self.bone_name = name.to_string();
    }

    pub fn get_bone_name(&self) -> &str {
        &self.bone_name
    }

    pub fn set_bounce(&mut self, bounce: f32) {
        self.bounce = bounce;
    }

    pub fn get_bounce(&self) -> f32 {
        self.bounce
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
    }

    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    pub fn set_linear_velocity(&mut self, velocity: Vector3) {
        self.linear_velocity = velocity;
    }

    pub fn get_linear_velocity(&self) -> Vector3 {
        self.linear_velocity
    }

    pub fn set_angular_velocity(&mut self, velocity: Vector3) {
        self.angular_velocity = velocity;
    }

    pub fn get_angular_velocity(&self) -> Vector3 {
        self.angular_velocity
    }

    pub fn set_gravity_scale(&mut self, scale: f32) {
        self.gravity_scale = scale;
    }

    pub fn get_gravity_scale(&self) -> f32 {
        self.gravity_scale
    }

    pub fn set_can_sleep(&mut self, can_sleep: bool) {
        self.can_sleep = can_sleep;
    }

    pub fn is_able_to_sleep(&self) -> bool {
        self.can_sleep
    }

    pub fn set_use_custom_integrator(&mut self, enabled: bool) {
        self.custom_integrator = enabled;
    }

    pub fn is_using_custom_integrator(&self) -> bool {
        self.custom_integrator
    }

    pub fn set_linear_damp_mode(&mut self, mode: DampMode) {
        self.linear_damp_mode = mode;
    }

    pub fn get_linear_damp_mode(&self) -> DampMode {
        self.linear_damp_mode
    }

    pub fn set_angular_damp_mode(&mut self, mode: DampMode) {
        self.angular_damp_mode = mode;
    }

    pub fn get_angular_damp_mode(&self) -> DampMode {
        self.angular_damp_mode
    }

    pub fn set_linear_damp(&mut self, damp: f32) {
        self.linear_damp = damp;
    }

    pub fn get_linear_damp(&self) -> f32 {
        self.linear_damp
    }

    pub fn set_angular_damp(&mut self, damp: f32) {
        self.angular_damp = damp;
    }

    pub fn get_angular_damp(&self) -> f32 {
        self.angular_damp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = PhysicalBone3DData::new();
        assert_eq!(p.get_joint_type(), JointType::None);
        assert_eq!(p.get_joint_offset(), Transform3D::IDENTITY);
        assert_eq!(p.get_body_offset(), Transform3D::IDENTITY);
        assert_eq!(p.get_body_offset_inverse(), Transform3D::IDENTITY);
        assert!(!p.get_simulate_physics());
        assert!(!p.get_internal_simulate_physics());
        assert_eq!(p.get_bone_id(), -1);
        assert_eq!(p.get_bone_name(), "");
        assert_eq!(p.get_bounce(), 0.0);
        assert_eq!(p.get_mass(), 1.0);
        assert_eq!(p.get_friction(), 1.0);
        assert_eq!(p.get_linear_velocity(), Vector3::ZERO);
        assert_eq!(p.get_angular_velocity(), Vector3::ZERO);
        assert_eq!(p.get_gravity_scale(), 1.0);
        assert!(p.is_able_to_sleep());
        assert!(!p.is_using_custom_integrator());
        assert_eq!(p.get_linear_damp_mode(), DampMode::Combine);
        assert_eq!(p.get_angular_damp_mode(), DampMode::Combine);
        assert_eq!(p.get_linear_damp(), 0.0);
        assert_eq!(p.get_angular_damp(), 0.0);

        assert_eq!(JointType::None as i32, 0);
        assert_eq!(JointType::Pin as i32, 1);
        assert_eq!(JointType::Cone as i32, 2);
        assert_eq!(JointType::Hinge as i32, 3);
        assert_eq!(JointType::Slider as i32, 4);
        assert_eq!(JointType::SixDof as i32, 5);
        assert_eq!(DampMode::Combine as i32, 0);
        assert_eq!(DampMode::Replace as i32, 1);
    }

    #[test]
    fn joint_data_defaults() {
        let p = PhysicalBone3DData::new();
        assert_eq!(p.pin_data().bias, 0.3);
        assert_eq!(p.pin_data().damping, 1.0);
        assert_eq!(p.pin_data().impulse_clamp, 0.0);

        assert!((p.cone_data().swing_span - std::f32::consts::PI * 0.25).abs() < 1e-6);
        assert!((p.cone_data().twist_span - std::f32::consts::PI).abs() < 1e-6);
        assert_eq!(p.cone_data().bias, 0.3);
        assert_eq!(p.cone_data().softness, 0.8);
        assert_eq!(p.cone_data().relaxation, 1.0);

        assert!(!p.hinge_data().angular_limit_enabled);
        assert!((p.hinge_data().angular_limit_upper - std::f32::consts::PI * 0.5).abs() < 1e-6);
        assert!((p.hinge_data().angular_limit_lower + std::f32::consts::PI * 0.5).abs() < 1e-6);

        assert_eq!(p.slider_data().linear_limit_upper, 1.0);
        assert_eq!(p.slider_data().linear_limit_lower, -1.0);
        assert_eq!(p.slider_data().linear_limit_restitution, 0.7);
        assert_eq!(p.slider_data().angular_limit_upper, 0.0);
    }

    #[test]
    fn setters_and_body_offset_inverse() {
        let mut p = PhysicalBone3DData::new();
        let offset = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::new(1.0, 2.0, 3.0),
        };
        p.set_body_offset(offset);
        assert_eq!(p.get_body_offset(), offset);
        let inverse = p.get_body_offset_inverse();
        let composed = offset * inverse;
        assert!(composed.origin.length() < 1e-5);
        assert!((composed.basis.get_column(0) - Vector3::RIGHT).length() < 1e-5);

        p.set_joint_type(JointType::Hinge);
        p.set_simulate_physics(true);
        p.set_bone_id(3);
        p.set_bone_name("Arm");
        p.set_bounce(0.5);
        p.set_mass(2.0);
        p.set_friction(0.25);
        p.set_linear_velocity(Vector3::new(1.0, 0.0, 0.0));
        p.set_angular_velocity(Vector3::new(0.0, 1.0, 0.0));
        p.set_gravity_scale(0.5);
        p.set_can_sleep(false);
        p.set_use_custom_integrator(true);
        p.set_linear_damp_mode(DampMode::Replace);
        p.set_angular_damp_mode(DampMode::Replace);
        p.set_linear_damp(0.1);
        p.set_angular_damp(0.2);

        assert_eq!(p.get_joint_type(), JointType::Hinge);
        assert!(p.get_simulate_physics());
        assert_eq!(p.get_bone_id(), 3);
        assert_eq!(p.get_bone_name(), "Arm");
        assert_eq!(p.get_bounce(), 0.5);
        assert_eq!(p.get_mass(), 2.0);
        assert_eq!(p.get_friction(), 0.25);
        assert_eq!(p.get_linear_velocity(), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(p.get_angular_velocity(), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(p.get_gravity_scale(), 0.5);
        assert!(!p.is_able_to_sleep());
        assert!(p.is_using_custom_integrator());
        assert_eq!(p.get_linear_damp_mode(), DampMode::Replace);
        assert_eq!(p.get_angular_damp(), 0.2);

        p.set_internal_simulate_physics(true);
        assert!(p.get_internal_simulate_physics());

        p.pin_data_mut().impulse_clamp = 1.0;
        p.cone_data_mut().softness = 0.5;
        p.hinge_data_mut().angular_limit_enabled = true;
        p.slider_data_mut().linear_limit_upper = 4.0;
        assert_eq!(p.pin_data().impulse_clamp, 1.0);
        assert_eq!(p.cone_data().softness, 0.5);
        assert!(p.hinge_data().angular_limit_enabled);
        assert_eq!(p.slider_data().linear_limit_upper, 4.0);
    }
}
