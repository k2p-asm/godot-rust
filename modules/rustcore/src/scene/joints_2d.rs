//! `scene/2d/physics/joints/*`: 2D joint node state and parameters.
//!
//! `Joint2D` stores the two body paths, the bias (0), the exclude-from-collision
//! flag (on) and the joint handle.
//!
//! - `PinJoint2D`: softness 0, angular limits 0/0, motor target velocity 0, motor
//!   and angular-limit flags off;
//! - `GrooveJoint2D`: length 50, initial offset 25;
//! - `DampedSpringJoint2D`: stiffness 20, damping 1, rest length 0, length 50.
//!
//! All setters are plain assignments in the C++ (no clamping).
//!
//! `_configure_joint`, the physics server and the body lookup are not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct Joint2DData {
    node_a: String,
    node_b: String,
    bias: f32,
    exclude_from_collision: bool,
    joint: Option<Rid>,
    body_a: Option<Rid>,
    body_b: Option<Rid>,
    configured: bool,
}

impl Default for Joint2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Joint2DData {
    pub fn new() -> Self {
        Self {
            node_a: String::new(),
            node_b: String::new(),
            bias: 0.0,
            exclude_from_collision: true,
            joint: None,
            body_a: None,
            body_b: None,
            configured: false,
        }
    }

    pub fn set_node_a(&mut self, path: &str) {
        self.node_a = path.to_string();
    }

    pub fn get_node_a(&self) -> &str {
        &self.node_a
    }

    pub fn set_node_b(&mut self, path: &str) {
        self.node_b = path.to_string();
    }

    pub fn get_node_b(&self) -> &str {
        &self.node_b
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }

    pub fn get_bias(&self) -> f32 {
        self.bias
    }

    pub fn set_exclude_nodes_from_collision(&mut self, enable: bool) {
        self.exclude_from_collision = enable;
    }

    pub fn get_exclude_nodes_from_collision(&self) -> bool {
        self.exclude_from_collision
    }

    pub fn set_joint(&mut self, joint: Option<Rid>) {
        self.joint = joint;
    }

    pub fn get_joint(&self) -> Option<Rid> {
        self.joint
    }

    pub fn set_bodies(&mut self, a: Option<Rid>, b: Option<Rid>) {
        self.body_a = a;
        self.body_b = b;
    }

    pub fn get_body_a(&self) -> Option<Rid> {
        self.body_a
    }

    pub fn get_body_b(&self) -> Option<Rid> {
        self.body_b
    }

    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }
}

#[derive(Clone, Debug)]
pub struct PinJoint2DData {
    pub base: Joint2DData,
    softness: f32,
    angular_limit_lower: f32,
    angular_limit_upper: f32,
    motor_target_velocity: f32,
    motor_enabled: bool,
    angular_limit_enabled: bool,
}

impl Default for PinJoint2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PinJoint2DData {
    pub fn new() -> Self {
        Self {
            base: Joint2DData::new(),
            softness: 0.0,
            angular_limit_lower: 0.0,
            angular_limit_upper: 0.0,
            motor_target_velocity: 0.0,
            motor_enabled: false,
            angular_limit_enabled: false,
        }
    }

    pub fn set_softness(&mut self, softness: f32) {
        self.softness = softness;
    }

    pub fn get_softness(&self) -> f32 {
        self.softness
    }

    pub fn set_angular_limit_lower(&mut self, lower: f32) {
        self.angular_limit_lower = lower;
    }

    pub fn get_angular_limit_lower(&self) -> f32 {
        self.angular_limit_lower
    }

    pub fn set_angular_limit_upper(&mut self, upper: f32) {
        self.angular_limit_upper = upper;
    }

    pub fn get_angular_limit_upper(&self) -> f32 {
        self.angular_limit_upper
    }

    pub fn set_motor_target_velocity(&mut self, velocity: f32) {
        self.motor_target_velocity = velocity;
    }

    pub fn get_motor_target_velocity(&self) -> f32 {
        self.motor_target_velocity
    }

    pub fn set_motor_enabled(&mut self, enabled: bool) {
        self.motor_enabled = enabled;
    }

    pub fn is_motor_enabled(&self) -> bool {
        self.motor_enabled
    }

    pub fn set_angular_limit_enabled(&mut self, enabled: bool) {
        self.angular_limit_enabled = enabled;
    }

    pub fn is_angular_limit_enabled(&self) -> bool {
        self.angular_limit_enabled
    }
}

#[derive(Clone, Debug)]
pub struct GrooveJoint2DData {
    pub base: Joint2DData,
    length: f32,
    initial_offset: f32,
}

impl Default for GrooveJoint2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GrooveJoint2DData {
    pub fn new() -> Self {
        Self {
            base: Joint2DData::new(),
            length: 50.0,
            initial_offset: 25.0,
        }
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_initial_offset(&mut self, offset: f32) {
        self.initial_offset = offset;
    }

    pub fn get_initial_offset(&self) -> f32 {
        self.initial_offset
    }
}

#[derive(Clone, Debug)]
pub struct DampedSpringJoint2DData {
    pub base: Joint2DData,
    stiffness: f32,
    damping: f32,
    rest_length: f32,
    length: f32,
}

impl Default for DampedSpringJoint2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl DampedSpringJoint2DData {
    pub fn new() -> Self {
        Self {
            base: Joint2DData::new(),
            stiffness: 20.0,
            damping: 1.0,
            rest_length: 0.0,
            length: 50.0,
        }
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_rest_length(&mut self, rest_length: f32) {
        self.rest_length = rest_length;
    }

    pub fn get_rest_length(&self) -> f32 {
        self.rest_length
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn get_damping(&self) -> f32 {
        self.damping
    }

    pub fn set_stiffness(&mut self, stiffness: f32) {
        self.stiffness = stiffness;
    }

    pub fn get_stiffness(&self) -> f32 {
        self.stiffness
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_base() {
        let mut j = Joint2DData::new();
        assert_eq!(j.get_node_a(), "");
        assert_eq!(j.get_node_b(), "");
        assert_eq!(j.get_bias(), 0.0);
        assert!(j.get_exclude_nodes_from_collision());
        assert!(j.get_joint().is_none());
        assert!(j.get_body_a().is_none());
        assert!(!j.is_configured());

        j.set_node_a("../A");
        j.set_node_b("../B");
        j.set_bias(0.5);
        j.set_exclude_nodes_from_collision(false);
        j.set_joint(Some(Rid::from_u64(1)));
        j.set_bodies(Some(Rid::from_u64(2)), Some(Rid::from_u64(3)));
        j.set_configured(true);
        assert_eq!(j.get_node_a(), "../A");
        assert_eq!(j.get_node_b(), "../B");
        assert_eq!(j.get_bias(), 0.5);
        assert!(!j.get_exclude_nodes_from_collision());
        assert_eq!(j.get_joint(), Some(Rid::from_u64(1)));
        assert_eq!(j.get_body_a(), Some(Rid::from_u64(2)));
        assert_eq!(j.get_body_b(), Some(Rid::from_u64(3)));
        assert!(j.is_configured());
    }

    #[test]
    fn pin_defaults() {
        let mut p = PinJoint2DData::new();
        assert_eq!(p.get_softness(), 0.0);
        assert_eq!(p.get_angular_limit_lower(), 0.0);
        assert_eq!(p.get_angular_limit_upper(), 0.0);
        assert_eq!(p.get_motor_target_velocity(), 0.0);
        assert!(!p.is_motor_enabled());
        assert!(!p.is_angular_limit_enabled());

        p.set_softness(0.25);
        p.set_angular_limit_lower(-1.0);
        p.set_angular_limit_upper(1.0);
        p.set_motor_target_velocity(3.0);
        p.set_motor_enabled(true);
        p.set_angular_limit_enabled(true);
        assert_eq!(p.get_softness(), 0.25);
        assert_eq!(p.get_angular_limit_lower(), -1.0);
        assert_eq!(p.get_angular_limit_upper(), 1.0);
        assert_eq!(p.get_motor_target_velocity(), 3.0);
        assert!(p.is_motor_enabled());
        assert!(p.is_angular_limit_enabled());
    }

    #[test]
    fn groove_and_spring_defaults() {
        let mut g = GrooveJoint2DData::new();
        assert_eq!(g.get_length(), 50.0);
        assert_eq!(g.get_initial_offset(), 25.0);
        g.set_length(80.0);
        g.set_initial_offset(10.0);
        assert_eq!(g.get_length(), 80.0);
        assert_eq!(g.get_initial_offset(), 10.0);

        let mut s = DampedSpringJoint2DData::new();
        assert_eq!(s.get_stiffness(), 20.0);
        assert_eq!(s.get_damping(), 1.0);
        assert_eq!(s.get_rest_length(), 0.0);
        assert_eq!(s.get_length(), 50.0);
        s.set_stiffness(5.0);
        s.set_damping(0.5);
        s.set_rest_length(12.0);
        s.set_length(30.0);
        assert_eq!(s.get_stiffness(), 5.0);
        assert_eq!(s.get_damping(), 0.5);
        assert_eq!(s.get_rest_length(), 12.0);
        assert_eq!(s.get_length(), 30.0);
    }
}
