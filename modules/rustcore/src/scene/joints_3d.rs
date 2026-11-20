//! `scene/3d/physics/joints/*`: joint node state and parameter tables.
//!
//! `Joint3D` stores the two body paths, the solver priority (1), the
//! exclude-from-collision flag (on) and the joint handle. Every concrete joint
//! adds a parameter table with the constructor's defaults:
//!
//! - `PinJoint3D`: bias 0.3, damping 1, impulse clamp 0;
//! - `HingeJoint3D`: bias 0.3, limit ±π/2, limit bias 0.3, softness 0.9,
//!   relaxation 1, motor velocity 1, motor max impulse 1, both flags off;
//! - `SliderJoint3D`: linear limits ±1 with 1/0.7/1 softness/restitution/damping,
//!   motion 1/0.7/0, orthogonal 1/0.7/1, and the angular table
//!   (0/0, 1/0.7/0, 1/0.7/1, 1/0.7/1);
//! - `ConeTwistJoint3D`: swing π/4, twist π, bias 0.3, softness 0.8, relaxation 1.
//!
//! `set_param`/`set_flag` reject the `MAX` sentinels with `ERR_FAIL_INDEX`, like
//! the C++; values themselves are not validated.
//!
//! `_configure_joint`, the physics server and `Generic6DOFJoint3D` (a 3-axis
//! table with per-axis flags, documented as pending) are not ported.

use crate::servers::rid::Rid;

/// `Joint3D`'s shared state.
#[derive(Clone, Debug)]
pub struct Joint3DData {
    node_a: String,
    node_b: String,
    solver_priority: i32,
    exclude_from_collision: bool,
    joint: Option<Rid>,
    configured: bool,
}

impl Default for Joint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Joint3DData {
    pub fn new() -> Self {
        Self {
            node_a: String::new(),
            node_b: String::new(),
            solver_priority: 1,
            exclude_from_collision: true,
            joint: None,
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

    pub fn set_solver_priority(&mut self, priority: i32) {
        self.solver_priority = priority;
    }

    pub fn get_solver_priority(&self) -> i32 {
        self.solver_priority
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

    pub fn set_configured(&mut self, configured: bool) {
        self.configured = configured;
    }

    pub fn is_configured(&self) -> bool {
        self.configured
    }
}

// -- PinJoint3D -------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PinParam {
    Bias,
    Damping,
    ImpulseClamp,
    Max,
}

pub const PIN_PARAM_MAX: usize = 3;

#[derive(Clone, Debug)]
pub struct PinJoint3DData {
    pub base: Joint3DData,
    params: [f32; PIN_PARAM_MAX],
}

impl Default for PinJoint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PinJoint3DData {
    pub fn new() -> Self {
        Self {
            base: Joint3DData::new(),
            params: [0.3, 1.0, 0.0],
        }
    }

    pub fn set_param(&mut self, param: PinParam, value: f32) -> bool {
        if param == PinParam::Max {
            return false;
        }
        self.params[param as usize] = value;
        true
    }

    pub fn get_param(&self, param: PinParam) -> Option<f32> {
        if param == PinParam::Max {
            return None;
        }
        Some(self.params[param as usize])
    }
}

// -- HingeJoint3D -----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HingeParam {
    Bias,
    LimitUpper,
    LimitLower,
    LimitBias,
    LimitSoftness,
    LimitRelaxation,
    MotorTargetVelocity,
    MotorMaxImpulse,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HingeFlag {
    UseLimit,
    EnableMotor,
    Max,
}

pub const HINGE_PARAM_MAX: usize = 8;
pub const HINGE_FLAG_MAX: usize = 2;

#[derive(Clone, Debug)]
pub struct HingeJoint3DData {
    pub base: Joint3DData,
    params: [f32; HINGE_PARAM_MAX],
    flags: [bool; HINGE_FLAG_MAX],
}

impl Default for HingeJoint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl HingeJoint3DData {
    pub fn new() -> Self {
        let half_pi = std::f32::consts::PI * 0.5;
        Self {
            base: Joint3DData::new(),
            params: [
                0.3,       // bias
                half_pi,   // limit upper
                -half_pi,  // limit lower
                0.3,       // limit bias
                0.9,       // limit softness
                1.0,       // limit relaxation
                1.0,       // motor target velocity
                1.0,       // motor max impulse
            ],
            flags: [false; HINGE_FLAG_MAX],
        }
    }

    pub fn set_param(&mut self, param: HingeParam, value: f32) -> bool {
        if param == HingeParam::Max {
            return false;
        }
        self.params[param as usize] = value;
        true
    }

    pub fn get_param(&self, param: HingeParam) -> Option<f32> {
        if param == HingeParam::Max {
            return None;
        }
        Some(self.params[param as usize])
    }

    pub fn set_flag(&mut self, flag: HingeFlag, value: bool) -> bool {
        if flag == HingeFlag::Max {
            return false;
        }
        self.flags[flag as usize] = value;
        true
    }

    pub fn get_flag(&self, flag: HingeFlag) -> bool {
        if flag == HingeFlag::Max {
            return false;
        }
        self.flags[flag as usize]
    }
}

// -- SliderJoint3D ----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SliderParam {
    LinearLimitUpper,
    LinearLimitLower,
    LinearLimitSoftness,
    LinearLimitRestitution,
    LinearLimitDamping,
    LinearMotionSoftness,
    LinearMotionRestitution,
    LinearMotionDamping,
    LinearOrthogonalSoftness,
    LinearOrthogonalRestitution,
    LinearOrthogonalDamping,
    AngularLimitUpper,
    AngularLimitLower,
    AngularLimitSoftness,
    AngularLimitRestitution,
    AngularLimitDamping,
    AngularMotionSoftness,
    AngularMotionRestitution,
    AngularMotionDamping,
    AngularOrthogonalSoftness,
    AngularOrthogonalRestitution,
    AngularOrthogonalDamping,
    Max,
}

pub const SLIDER_PARAM_MAX: usize = 22;

#[derive(Clone, Debug)]
pub struct SliderJoint3DData {
    pub base: Joint3DData,
    params: [f32; SLIDER_PARAM_MAX],
}

impl Default for SliderJoint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderJoint3DData {
    pub fn new() -> Self {
        Self {
            base: Joint3DData::new(),
            params: [
                1.0, -1.0, 1.0, 0.7, 1.0, // linear limit
                1.0, 0.7, 0.0, // linear motion
                1.0, 0.7, 1.0, // linear orthogonal
                0.0, 0.0, 1.0, 0.7, 0.0, // angular limit
                1.0, 0.7, 1.0, // angular motion
                1.0, 0.7, 1.0, // angular orthogonal
            ],
        }
    }

    pub fn set_param(&mut self, param: SliderParam, value: f32) -> bool {
        if param == SliderParam::Max {
            return false;
        }
        self.params[param as usize] = value;
        true
    }

    pub fn get_param(&self, param: SliderParam) -> Option<f32> {
        if param == SliderParam::Max {
            return None;
        }
        Some(self.params[param as usize])
    }
}

// -- ConeTwistJoint3D -------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ConeTwistParam {
    SwingSpan,
    TwistSpan,
    Bias,
    Softness,
    Relaxation,
    Max,
}

pub const CONE_TWIST_PARAM_MAX: usize = 5;

#[derive(Clone, Debug)]
pub struct ConeTwistJoint3DData {
    pub base: Joint3DData,
    params: [f32; CONE_TWIST_PARAM_MAX],
}

impl Default for ConeTwistJoint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl ConeTwistJoint3DData {
    pub fn new() -> Self {
        Self {
            base: Joint3DData::new(),
            params: [
                std::f32::consts::PI * 0.25, // swing span
                std::f32::consts::PI,        // twist span
                0.3,                         // bias
                0.8,                         // softness
                1.0,                         // relaxation
            ],
        }
    }

    pub fn set_param(&mut self, param: ConeTwistParam, value: f32) -> bool {
        if param == ConeTwistParam::Max {
            return false;
        }
        self.params[param as usize] = value;
        true
    }

    pub fn get_param(&self, param: ConeTwistParam) -> Option<f32> {
        if param == ConeTwistParam::Max {
            return None;
        }
        Some(self.params[param as usize])
    }
}

// -- Generic6DOFJoint3D -----------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum G6dofParam {
    LinearLowerLimit,
    LinearUpperLimit,
    LinearLimitSoftness,
    LinearRestitution,
    LinearDamping,
    LinearMotorTargetVelocity,
    LinearMotorForceLimit,
    LinearSpringStiffness,
    LinearSpringDamping,
    LinearSpringEquilibriumPoint,
    AngularLowerLimit,
    AngularUpperLimit,
    AngularLimitSoftness,
    AngularDamping,
    AngularRestitution,
    AngularForceLimit,
    AngularErp,
    AngularMotorTargetVelocity,
    AngularMotorForceLimit,
    AngularSpringStiffness,
    AngularSpringDamping,
    AngularSpringEquilibriumPoint,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum G6dofFlag {
    EnableLinearLimit,
    EnableAngularLimit,
    EnableLinearSpring,
    EnableAngularSpring,
    EnableMotor,
    EnableLinearMotor,
    Max,
}

pub const G6DOF_PARAM_MAX: usize = 22;
pub const G6DOF_FLAG_MAX: usize = 6;

/// The per-axis parameter/flag tables of `Generic6DOFJoint3D`.
#[derive(Clone, Debug)]
pub struct Generic6DOFJoint3DData {
    pub base: Joint3DData,
    params_x: [f32; G6DOF_PARAM_MAX],
    params_y: [f32; G6DOF_PARAM_MAX],
    params_z: [f32; G6DOF_PARAM_MAX],
    flags_x: [bool; G6DOF_FLAG_MAX],
    flags_y: [bool; G6DOF_FLAG_MAX],
    flags_z: [bool; G6DOF_FLAG_MAX],
}

impl Default for Generic6DOFJoint3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Generic6DOFJoint3DData {
    pub fn new() -> Self {
        // The constructor sets the same table on all three axes.
        let params = [
            0.0,   // linear lower limit
            0.0,   // linear upper limit
            0.7,   // linear limit softness
            0.5,   // linear restitution
            1.0,   // linear damping
            0.0,   // linear motor target velocity
            0.0,   // linear motor force limit
            0.01,  // linear spring stiffness
            0.01,  // linear spring damping
            0.0,   // linear spring equilibrium point
            0.0,   // angular lower limit
            0.0,   // angular upper limit
            0.5,   // angular limit softness
            1.0,   // angular damping
            0.0,   // angular restitution
            0.0,   // angular force limit
            0.5,   // angular ERP
            0.0,   // angular motor target velocity
            300.0, // angular motor force limit
            0.0,   // angular spring stiffness
            0.0,   // angular spring damping
            0.0,   // angular spring equilibrium point
        ];
        let flags = [true, true, false, false, false, false];
        Self {
            base: Joint3DData::new(),
            params_x: params,
            params_y: params,
            params_z: params,
            flags_x: flags,
            flags_y: flags,
            flags_z: flags,
        }
    }

    pub fn set_param_x(&mut self, param: G6dofParam, value: f32) -> bool {
        if param == G6dofParam::Max {
            return false;
        }
        self.params_x[param as usize] = value;
        true
    }

    pub fn get_param_x(&self, param: G6dofParam) -> Option<f32> {
        if param == G6dofParam::Max {
            return None;
        }
        Some(self.params_x[param as usize])
    }

    pub fn set_param_y(&mut self, param: G6dofParam, value: f32) -> bool {
        if param == G6dofParam::Max {
            return false;
        }
        self.params_y[param as usize] = value;
        true
    }

    pub fn get_param_y(&self, param: G6dofParam) -> Option<f32> {
        if param == G6dofParam::Max {
            return None;
        }
        Some(self.params_y[param as usize])
    }

    pub fn set_param_z(&mut self, param: G6dofParam, value: f32) -> bool {
        if param == G6dofParam::Max {
            return false;
        }
        self.params_z[param as usize] = value;
        true
    }

    pub fn get_param_z(&self, param: G6dofParam) -> Option<f32> {
        if param == G6dofParam::Max {
            return None;
        }
        Some(self.params_z[param as usize])
    }

    pub fn set_flag_x(&mut self, flag: G6dofFlag, enabled: bool) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_x[flag as usize] = enabled;
        true
    }

    pub fn get_flag_x(&self, flag: G6dofFlag) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_x[flag as usize]
    }

    pub fn set_flag_y(&mut self, flag: G6dofFlag, enabled: bool) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_y[flag as usize] = enabled;
        true
    }

    pub fn get_flag_y(&self, flag: G6dofFlag) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_y[flag as usize]
    }

    pub fn set_flag_z(&mut self, flag: G6dofFlag, enabled: bool) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_z[flag as usize] = enabled;
        true
    }

    pub fn get_flag_z(&self, flag: G6dofFlag) -> bool {
        if flag == G6dofFlag::Max {
            return false;
        }
        self.flags_z[flag as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_base() {
        let mut j = Joint3DData::new();
        assert_eq!(j.get_node_a(), "");
        assert_eq!(j.get_node_b(), "");
        assert_eq!(j.get_solver_priority(), 1);
        assert!(j.get_exclude_nodes_from_collision());
        assert!(j.get_joint().is_none());
        assert!(!j.is_configured());

        j.set_node_a("../BodyA");
        j.set_node_b("../BodyB");
        j.set_solver_priority(4);
        j.set_exclude_nodes_from_collision(false);
        j.set_joint(Some(Rid::from_u64(9)));
        j.set_configured(true);
        assert_eq!(j.get_node_a(), "../BodyA");
        assert_eq!(j.get_node_b(), "../BodyB");
        assert_eq!(j.get_solver_priority(), 4);
        assert!(!j.get_exclude_nodes_from_collision());
        assert_eq!(j.get_joint(), Some(Rid::from_u64(9)));
        assert!(j.is_configured());
    }

    #[test]
    fn pin_defaults_and_params() {
        let mut p = PinJoint3DData::new();
        assert_eq!(p.get_param(PinParam::Bias), Some(0.3));
        assert_eq!(p.get_param(PinParam::Damping), Some(1.0));
        assert_eq!(p.get_param(PinParam::ImpulseClamp), Some(0.0));
        assert!(!p.set_param(PinParam::Max, 1.0));
        assert_eq!(p.get_param(PinParam::Max), None);

        assert!(p.set_param(PinParam::Bias, 0.6));
        assert_eq!(p.get_param(PinParam::Bias), Some(0.6));
        assert_eq!(PinParam::Bias as i32, 0);
        assert_eq!(PinParam::ImpulseClamp as i32, 2);
        assert_eq!(PinParam::Max as i32, 3);
    }

    #[test]
    fn hinge_defaults_and_flags() {
        let mut h = HingeJoint3DData::new();
        let half_pi = std::f32::consts::PI * 0.5;
        assert!((h.get_param(HingeParam::Bias).unwrap() - 0.3).abs() < 1e-6);
        assert!((h.get_param(HingeParam::LimitUpper).unwrap() - half_pi).abs() < 1e-6);
        assert!((h.get_param(HingeParam::LimitLower).unwrap() + half_pi).abs() < 1e-6);
        assert!((h.get_param(HingeParam::LimitBias).unwrap() - 0.3).abs() < 1e-6);
        assert!((h.get_param(HingeParam::LimitSoftness).unwrap() - 0.9).abs() < 1e-6);
        assert!((h.get_param(HingeParam::LimitRelaxation).unwrap() - 1.0).abs() < 1e-6);
        assert!((h.get_param(HingeParam::MotorTargetVelocity).unwrap() - 1.0).abs() < 1e-6);
        assert!((h.get_param(HingeParam::MotorMaxImpulse).unwrap() - 1.0).abs() < 1e-6);
        assert!(!h.get_flag(HingeFlag::UseLimit));
        assert!(!h.get_flag(HingeFlag::EnableMotor));

        assert!(h.set_flag(HingeFlag::UseLimit, true));
        assert!(h.get_flag(HingeFlag::UseLimit));
        assert!(!h.set_flag(HingeFlag::Max, true));
        assert!(!h.get_flag(HingeFlag::Max));
        assert!(!h.set_param(HingeParam::Max, 1.0));
        assert_eq!(h.get_param(HingeParam::Max), None);

        assert_eq!(HingeParam::MotorMaxImpulse as i32, 7);
        assert_eq!(HingeParam::Max as i32, 8);
        assert_eq!(HingeFlag::UseLimit as i32, 0);
        assert_eq!(HingeFlag::Max as i32, 2);
    }

    #[test]
    fn slider_defaults() {
        let mut s = SliderJoint3DData::new();
        assert!((s.get_param(SliderParam::LinearLimitUpper).unwrap() - 1.0).abs() < 1e-6);
        assert!((s.get_param(SliderParam::LinearLimitLower).unwrap() + 1.0).abs() < 1e-6);
        assert!((s.get_param(SliderParam::LinearMotionDamping).unwrap() - 0.0).abs() < 1e-6);
        assert!((s.get_param(SliderParam::AngularLimitUpper).unwrap() - 0.0).abs() < 1e-6);
        assert!((s.get_param(SliderParam::AngularLimitDamping).unwrap() - 0.0).abs() < 1e-6);
        assert!((s.get_param(SliderParam::AngularOrthogonalDamping).unwrap() - 1.0).abs() < 1e-6);
        assert!(!s.set_param(SliderParam::Max, 1.0));
        assert_eq!(s.get_param(SliderParam::Max), None);

        assert!(s.set_param(SliderParam::LinearLimitUpper, 5.0));
        assert_eq!(s.get_param(SliderParam::LinearLimitUpper), Some(5.0));
        assert_eq!(SliderParam::AngularLimitUpper as i32, 11);
        assert_eq!(SliderParam::Max as i32, 22);
    }

    #[test]
    fn cone_twist_defaults() {
        let mut c = ConeTwistJoint3DData::new();
        let pi = std::f32::consts::PI;
        assert!((c.get_param(ConeTwistParam::SwingSpan).unwrap() - pi * 0.25).abs() < 1e-6);
        assert!((c.get_param(ConeTwistParam::TwistSpan).unwrap() - pi).abs() < 1e-6);
        assert!((c.get_param(ConeTwistParam::Bias).unwrap() - 0.3).abs() < 1e-6);
        assert!((c.get_param(ConeTwistParam::Softness).unwrap() - 0.8).abs() < 1e-6);
        assert!((c.get_param(ConeTwistParam::Relaxation).unwrap() - 1.0).abs() < 1e-6);
        assert!(!c.set_param(ConeTwistParam::Max, 1.0));
        assert_eq!(c.get_param(ConeTwistParam::Max), None);

        assert_eq!(ConeTwistParam::SwingSpan as i32, 0);
        assert_eq!(ConeTwistParam::Relaxation as i32, 4);
        assert_eq!(ConeTwistParam::Max as i32, 5);
    }

    #[test]
    fn generic_6dof_defaults() {
        let mut g = Generic6DOFJoint3DData::new();

        // All three axes start with the same table.
        for axis in 0..3 {
            let get = |param: G6dofParam| match axis {
                0 => g.get_param_x(param),
                1 => g.get_param_y(param),
                _ => g.get_param_z(param),
            };
            assert_eq!(get(G6dofParam::LinearLowerLimit), Some(0.0));
            assert_eq!(get(G6dofParam::LinearUpperLimit), Some(0.0));
            assert!((get(G6dofParam::LinearLimitSoftness).unwrap() - 0.7).abs() < 1e-6);
            assert!((get(G6dofParam::LinearRestitution).unwrap() - 0.5).abs() < 1e-6);
            assert!((get(G6dofParam::LinearDamping).unwrap() - 1.0).abs() < 1e-6);
            assert!((get(G6dofParam::LinearSpringStiffness).unwrap() - 0.01).abs() < 1e-6);
            assert!((get(G6dofParam::LinearSpringDamping).unwrap() - 0.01).abs() < 1e-6);
            assert!((get(G6dofParam::AngularLimitSoftness).unwrap() - 0.5).abs() < 1e-6);
            assert!((get(G6dofParam::AngularDamping).unwrap() - 1.0).abs() < 1e-6);
            assert!((get(G6dofParam::AngularErp).unwrap() - 0.5).abs() < 1e-6);
            assert!((get(G6dofParam::AngularMotorForceLimit).unwrap() - 300.0).abs() < 1e-6);
        }

        // Limits are enabled, motors and springs are not.
        for axis in 0..3 {
            let flag = |flag: G6dofFlag| match axis {
                0 => g.get_flag_x(flag),
                1 => g.get_flag_y(flag),
                _ => g.get_flag_z(flag),
            };
            assert!(flag(G6dofFlag::EnableLinearLimit));
            assert!(flag(G6dofFlag::EnableAngularLimit));
            assert!(!flag(G6dofFlag::EnableLinearSpring));
            assert!(!flag(G6dofFlag::EnableAngularSpring));
            assert!(!flag(G6dofFlag::EnableMotor));
            assert!(!flag(G6dofFlag::EnableLinearMotor));
        }
    }

    #[test]
    fn generic_6dof_axes_are_independent() {
        let mut g = Generic6DOFJoint3DData::new();
        assert!(g.set_param_x(G6dofParam::LinearUpperLimit, 1.0));
        assert!(g.set_param_y(G6dofParam::LinearUpperLimit, 2.0));
        assert!(g.set_param_z(G6dofParam::LinearUpperLimit, 3.0));
        assert_eq!(g.get_param_x(G6dofParam::LinearUpperLimit), Some(1.0));
        assert_eq!(g.get_param_y(G6dofParam::LinearUpperLimit), Some(2.0));
        assert_eq!(g.get_param_z(G6dofParam::LinearUpperLimit), Some(3.0));

        assert!(g.set_flag_x(G6dofFlag::EnableMotor, true));
        assert!(!g.get_flag_y(G6dofFlag::EnableMotor));
        assert!(!g.get_flag_z(G6dofFlag::EnableMotor));
        assert!(g.get_flag_x(G6dofFlag::EnableMotor));

        // Sentinels are rejected on every axis.
        assert!(!g.set_param_x(G6dofParam::Max, 1.0));
        assert!(!g.set_param_y(G6dofParam::Max, 1.0));
        assert!(!g.set_param_z(G6dofParam::Max, 1.0));
        assert!(!g.set_flag_x(G6dofFlag::Max, true));
        assert!(!g.set_flag_y(G6dofFlag::Max, true));
        assert!(!g.set_flag_z(G6dofFlag::Max, true));
        assert_eq!(g.get_param_x(G6dofParam::Max), None);
        assert!(!g.get_flag_x(G6dofFlag::Max));

        assert_eq!(G6dofParam::LinearSpringEquilibriumPoint as i32, 9);
        assert_eq!(G6dofParam::AngularUpperLimit as i32, 11);
        assert_eq!(G6dofParam::Max as i32, 22);
        assert_eq!(G6dofFlag::EnableAngularLimit as i32, 1);
        assert_eq!(G6dofFlag::EnableLinearMotor as i32, 5);
        assert_eq!(G6dofFlag::Max as i32, 6);
    }
}
