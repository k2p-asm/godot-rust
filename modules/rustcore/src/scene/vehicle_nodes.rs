//! `scene/3d/physics/vehicle_body_3d.*` (state half): `VehicleBody3D` and
//! `VehicleWheel3D`.
//!
//! Body defaults: zero engine force, brake and steering; zero pitch control,
//! steering value and current speed (the last three are written by the physics
//! step in the engine).
//!
//! Wheel defaults: rest length 0.15, suspension travel 0.2, radius 0.5,
//! stiffness 5.88, maximum force 6000, damping compression 0.83, damping
//! relaxation 0.88, friction slip 10.5, not traction, not steering, not a front
//! wheel, and zero steering/rotation/delta-rotation/RPM with no contact.
//!
//! The Bullet-style raycast/suspension/friction solver is not ported; the state
//! setters are plain assignments like the C++.

use crate::servers::rid::Rid;

// -- VehicleBody3D ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct VehicleBody3DData {
    engine_force: f32,
    brake: f32,
    steering: f32,
    pitch_control: f32,
    steering_value: f32,
    current_speed_km_h: f32,
    excluded: Vec<Rid>,
}

impl Default for VehicleBody3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleBody3DData {
    pub fn new() -> Self {
        Self {
            engine_force: 0.0,
            brake: 0.0,
            steering: 0.0,
            pitch_control: 0.0,
            steering_value: 0.0,
            current_speed_km_h: 0.0,
            excluded: Vec::new(),
        }
    }

    pub fn set_engine_force(&mut self, force: f32) {
        self.engine_force = force;
    }

    pub fn get_engine_force(&self) -> f32 {
        self.engine_force
    }

    pub fn set_brake(&mut self, brake: f32) {
        self.brake = brake;
    }

    pub fn get_brake(&self) -> f32 {
        self.brake
    }

    pub fn set_steering(&mut self, steering: f32) {
        self.steering = steering;
    }

    pub fn get_steering(&self) -> f32 {
        self.steering
    }

    /// Written by the physics step.
    pub fn set_pitch_control(&mut self, value: f32) {
        self.pitch_control = value;
    }

    pub fn get_pitch_control(&self) -> f32 {
        self.pitch_control
    }

    pub fn set_steering_value(&mut self, value: f32) {
        self.steering_value = value;
    }

    pub fn get_steering_value(&self) -> f32 {
        self.steering_value
    }

    pub fn set_current_speed_km_h(&mut self, speed: f32) {
        self.current_speed_km_h = speed;
    }

    pub fn get_current_speed_km_h(&self) -> f32 {
        self.current_speed_km_h
    }

    pub fn add_excluded_body(&mut self, body: Rid) -> bool {
        if self.excluded.contains(&body) {
            return false;
        }
        self.excluded.push(body);
        true
    }

    pub fn get_excluded_bodies(&self) -> &[Rid] {
        &self.excluded
    }
}

// -- VehicleWheel3D ---------------------------------------------------------

#[derive(Clone, Debug)]
pub struct VehicleWheel3DData {
    wheel_rest_length: f32,
    suspension_travel: f32,
    wheel_radius: f32,
    suspension_stiffness: f32,
    suspension_max_force: f32,
    damping_compression: f32,
    damping_relaxation: f32,
    friction_slip: f32,
    use_as_traction: bool,
    use_as_steering: bool,
    is_front_wheel: bool,
    steering: f32,
    rotation: f32,
    delta_rotation: f32,
    rpm: f32,
    in_contact: bool,
}

impl Default for VehicleWheel3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleWheel3DData {
    pub fn new() -> Self {
        Self {
            wheel_rest_length: 0.15,
            suspension_travel: 0.2,
            wheel_radius: 0.5,
            suspension_stiffness: 5.88,
            suspension_max_force: 6000.0,
            damping_compression: 0.83,
            damping_relaxation: 0.88,
            friction_slip: 10.5,
            use_as_traction: false,
            use_as_steering: false,
            is_front_wheel: false,
            steering: 0.0,
            rotation: 0.0,
            delta_rotation: 0.0,
            rpm: 0.0,
            in_contact: false,
        }
    }

    pub fn set_wheel_rest_length(&mut self, length: f32) {
        self.wheel_rest_length = length;
    }

    pub fn get_wheel_rest_length(&self) -> f32 {
        self.wheel_rest_length
    }

    /// `set_suspension_travel` (the engine calls the member `m_maxSuspensionTravel`).
    pub fn set_suspension_travel(&mut self, travel: f32) {
        self.suspension_travel = travel;
    }

    pub fn get_suspension_travel(&self) -> f32 {
        self.suspension_travel
    }

    pub fn set_wheel_radius(&mut self, radius: f32) {
        self.wheel_radius = radius;
    }

    pub fn get_wheel_radius(&self) -> f32 {
        self.wheel_radius
    }

    pub fn set_suspension_stiffness(&mut self, stiffness: f32) {
        self.suspension_stiffness = stiffness;
    }

    pub fn get_suspension_stiffness(&self) -> f32 {
        self.suspension_stiffness
    }

    pub fn set_suspension_max_force(&mut self, force: f32) {
        self.suspension_max_force = force;
    }

    pub fn get_suspension_max_force(&self) -> f32 {
        self.suspension_max_force
    }

    pub fn set_damping_compression(&mut self, damping: f32) {
        self.damping_compression = damping;
    }

    pub fn get_damping_compression(&self) -> f32 {
        self.damping_compression
    }

    pub fn set_damping_relaxation(&mut self, damping: f32) {
        self.damping_relaxation = damping;
    }

    pub fn get_damping_relaxation(&self) -> f32 {
        self.damping_relaxation
    }

    pub fn set_friction_slip(&mut self, friction: f32) {
        self.friction_slip = friction;
    }

    pub fn get_friction_slip(&self) -> f32 {
        self.friction_slip
    }

    pub fn set_use_as_traction(&mut self, enabled: bool) {
        self.use_as_traction = enabled;
    }

    pub fn is_used_as_traction(&self) -> bool {
        self.use_as_traction
    }

    pub fn set_use_as_steering(&mut self, enabled: bool) {
        self.use_as_steering = enabled;
    }

    pub fn is_used_as_steering(&self) -> bool {
        self.use_as_steering
    }

    /// The engine's `is_front_wheel` flag (set through the body's wheel list).
    pub fn set_front_wheel(&mut self, front: bool) {
        self.is_front_wheel = front;
    }

    pub fn is_front_wheel(&self) -> bool {
        self.is_front_wheel
    }

    pub fn set_steering(&mut self, steering: f32) {
        self.steering = steering;
    }

    pub fn get_steering(&self) -> f32 {
        self.steering
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    /// `get_delta_rotation`.
    pub fn get_delta_rotation(&self) -> f32 {
        self.delta_rotation
    }

    pub fn set_delta_rotation(&mut self, delta: f32) {
        self.delta_rotation = delta;
    }

    pub fn get_rpm(&self) -> f32 {
        self.rpm
    }

    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm;
    }

    pub fn is_in_contact(&self) -> bool {
        self.in_contact
    }

    pub fn set_in_contact(&mut self, contact: bool) {
        self.in_contact = contact;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_defaults_and_controls() {
        let mut body = VehicleBody3DData::new();
        assert_eq!(body.get_engine_force(), 0.0);
        assert_eq!(body.get_brake(), 0.0);
        assert_eq!(body.get_steering(), 0.0);
        assert_eq!(body.get_pitch_control(), 0.0);
        assert_eq!(body.get_steering_value(), 0.0);
        assert_eq!(body.get_current_speed_km_h(), 0.0);
        assert!(body.get_excluded_bodies().is_empty());

        body.set_engine_force(300.0);
        body.set_brake(1.0);
        body.set_steering(0.25);
        body.set_pitch_control(0.1);
        body.set_steering_value(0.2);
        body.set_current_speed_km_h(42.0);
        assert_eq!(body.get_engine_force(), 300.0);
        assert_eq!(body.get_brake(), 1.0);
        assert_eq!(body.get_steering(), 0.25);
        assert_eq!(body.get_pitch_control(), 0.1);
        assert_eq!(body.get_steering_value(), 0.2);
        assert_eq!(body.get_current_speed_km_h(), 42.0);

        assert!(body.add_excluded_body(Rid::from_u64(3)));
        assert!(!body.add_excluded_body(Rid::from_u64(3)));
        assert_eq!(body.get_excluded_bodies(), &[Rid::from_u64(3)]);
    }

    #[test]
    fn wheel_defaults_and_flags() {
        let mut wheel = VehicleWheel3DData::new();
        assert_eq!(wheel.get_wheel_rest_length(), 0.15);
        assert_eq!(wheel.get_suspension_travel(), 0.2);
        assert_eq!(wheel.get_wheel_radius(), 0.5);
        assert_eq!(wheel.get_suspension_stiffness(), 5.88);
        assert_eq!(wheel.get_suspension_max_force(), 6000.0);
        assert_eq!(wheel.get_damping_compression(), 0.83);
        assert_eq!(wheel.get_damping_relaxation(), 0.88);
        assert_eq!(wheel.get_friction_slip(), 10.5);
        assert!(!wheel.is_used_as_traction());
        assert!(!wheel.is_used_as_steering());
        assert!(!wheel.is_front_wheel());
        assert_eq!(wheel.get_steering(), 0.0);
        assert_eq!(wheel.get_rotation(), 0.0);
        assert_eq!(wheel.get_delta_rotation(), 0.0);
        assert_eq!(wheel.get_rpm(), 0.0);
        assert!(!wheel.is_in_contact());

        wheel.set_wheel_rest_length(0.3);
        wheel.set_suspension_travel(0.4);
        wheel.set_wheel_radius(0.25);
        wheel.set_suspension_stiffness(9.0);
        wheel.set_suspension_max_force(1000.0);
        wheel.set_damping_compression(0.5);
        wheel.set_damping_relaxation(0.6);
        wheel.set_friction_slip(2.0);
        wheel.set_use_as_traction(true);
        wheel.set_use_as_steering(true);
        wheel.set_front_wheel(true);
        wheel.set_steering(0.1);
        wheel.set_rotation(1.0);
        wheel.set_delta_rotation(0.01);
        wheel.set_rpm(120.0);
        wheel.set_in_contact(true);

        assert_eq!(wheel.get_wheel_rest_length(), 0.3);
        assert_eq!(wheel.get_suspension_travel(), 0.4);
        assert_eq!(wheel.get_wheel_radius(), 0.25);
        assert_eq!(wheel.get_suspension_stiffness(), 9.0);
        assert_eq!(wheel.get_suspension_max_force(), 1000.0);
        assert_eq!(wheel.get_damping_compression(), 0.5);
        assert_eq!(wheel.get_damping_relaxation(), 0.6);
        assert_eq!(wheel.get_friction_slip(), 2.0);
        assert!(wheel.is_used_as_traction());
        assert!(wheel.is_used_as_steering());
        assert!(wheel.is_front_wheel());
        assert_eq!(wheel.get_steering(), 0.1);
        assert_eq!(wheel.get_rotation(), 1.0);
        assert_eq!(wheel.get_delta_rotation(), 0.01);
        assert_eq!(wheel.get_rpm(), 120.0);
        assert!(wheel.is_in_contact());
    }
}
