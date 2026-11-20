//! `scene/3d/physics/{character_body,static_body,rigid_body}_3d.*` (state half).
//!
//! `CharacterBody3D` is a kinematic body. Its defaults come from the header:
//! grounded motion, add-velocity when leaving a platform, slide on ceiling,
//! floor stop-on-slope and block-on-wall on, floor constant speed off, six
//! slides, floor snap 0.1, floor maximum angle 45°, wall minimum slide angle 15°,
//! platform floor layers all and wall layers none, up `(0,1,0)` and zero
//! velocities/normals. `set_max_slides` rejects values below 1 and
//! `set_floor_snap_length` rejects negatives, like the C++.
//!
//! `StaticBody3D` keeps its two constant velocities and the material handle.
//! `RigidBody3D` keeps mass, gravity scale, damp modes/values, freeze state/mode,
//! CCD, sleeping, reported contacts, the custom integrator flag, centre-of-mass
//! mode/vector and both velocities.
//!
//! `move_and_slide`, the physics server and collision resolution are not ported.

use crate::math::funcs::deg_to_rad;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

// -- CharacterBody3D --------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MotionMode {
    Grounded,
    Floating,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PlatformOnLeave {
    AddVelocity,
    AddUpwardVelocity,
    DoNothing,
}

#[derive(Clone, Debug)]
pub struct CharacterBody3DData {
    motion_mode: MotionMode,
    platform_on_leave: PlatformOnLeave,
    floor_constant_speed: bool,
    floor_stop_on_slope: bool,
    floor_block_on_wall: bool,
    slide_on_ceiling: bool,
    max_slides: i32,
    platform_floor_layers: u32,
    platform_wall_layers: u32,
    floor_snap_length: f32,
    floor_max_angle: f32,
    wall_min_slide_angle: f32,
    safe_margin: f32,
    up_direction: Vector3,
    velocity: Vector3,
    floor_normal: Vector3,
    wall_normal: Vector3,
    ceiling_normal: Vector3,
    last_motion: Vector3,
    platform_velocity: Vector3,
    platform_angular_velocity: Vector3,
    real_velocity: Vector3,
}

impl Default for CharacterBody3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterBody3DData {
    pub fn new() -> Self {
        Self {
            motion_mode: MotionMode::Grounded,
            platform_on_leave: PlatformOnLeave::AddVelocity,
            floor_constant_speed: false,
            floor_stop_on_slope: true,
            floor_block_on_wall: true,
            slide_on_ceiling: true,
            max_slides: 6,
            platform_floor_layers: u32::MAX,
            platform_wall_layers: 0,
            floor_snap_length: 0.1,
            floor_max_angle: deg_to_rad(45.0),
            wall_min_slide_angle: deg_to_rad(15.0),
            safe_margin: 0.001,
            up_direction: Vector3::new(0.0, 1.0, 0.0),
            velocity: Vector3::ZERO,
            floor_normal: Vector3::ZERO,
            wall_normal: Vector3::ZERO,
            ceiling_normal: Vector3::ZERO,
            last_motion: Vector3::ZERO,
            platform_velocity: Vector3::ZERO,
            platform_angular_velocity: Vector3::ZERO,
            real_velocity: Vector3::ZERO,
        }
    }

    pub fn set_motion_mode(&mut self, mode: MotionMode) {
        self.motion_mode = mode;
    }

    pub fn get_motion_mode(&self) -> MotionMode {
        self.motion_mode
    }

    pub fn set_platform_on_leave(&mut self, mode: PlatformOnLeave) {
        self.platform_on_leave = mode;
    }

    pub fn get_platform_on_leave(&self) -> PlatformOnLeave {
        self.platform_on_leave
    }

    pub fn set_floor_constant_speed_enabled(&mut self, enabled: bool) {
        self.floor_constant_speed = enabled;
    }

    pub fn is_floor_constant_speed_enabled(&self) -> bool {
        self.floor_constant_speed
    }

    pub fn set_floor_stop_on_slope_enabled(&mut self, enabled: bool) {
        self.floor_stop_on_slope = enabled;
    }

    pub fn is_floor_stop_on_slope_enabled(&self) -> bool {
        self.floor_stop_on_slope
    }

    pub fn set_floor_block_on_wall_enabled(&mut self, enabled: bool) {
        self.floor_block_on_wall = enabled;
    }

    pub fn is_floor_block_on_wall_enabled(&self) -> bool {
        self.floor_block_on_wall
    }

    pub fn set_slide_on_ceiling_enabled(&mut self, enabled: bool) {
        self.slide_on_ceiling = enabled;
    }

    pub fn is_slide_on_ceiling_enabled(&self) -> bool {
        self.slide_on_ceiling
    }

    /// `ERR_FAIL_COND(p_max_slides < 1)`.
    pub fn set_max_slides(&mut self, max_slides: i32) -> bool {
        if max_slides < 1 {
            return false;
        }
        self.max_slides = max_slides;
        true
    }

    pub fn get_max_slides(&self) -> i32 {
        self.max_slides
    }

    pub fn set_platform_floor_layers(&mut self, layers: u32) {
        self.platform_floor_layers = layers;
    }

    pub fn get_platform_floor_layers(&self) -> u32 {
        self.platform_floor_layers
    }

    pub fn set_platform_wall_layers(&mut self, layers: u32) {
        self.platform_wall_layers = layers;
    }

    pub fn get_platform_wall_layers(&self) -> u32 {
        self.platform_wall_layers
    }

    /// `ERR_FAIL_COND(p_floor_snap_length < 0)`.
    pub fn set_floor_snap_length(&mut self, length: f32) -> bool {
        if length < 0.0 {
            return false;
        }
        self.floor_snap_length = length;
        true
    }

    pub fn get_floor_snap_length(&self) -> f32 {
        self.floor_snap_length
    }

    pub fn set_floor_max_angle(&mut self, radians: f32) {
        self.floor_max_angle = radians;
    }

    pub fn get_floor_max_angle(&self) -> f32 {
        self.floor_max_angle
    }

    /// `get_floor_angle`: the angle between the floor normal and `up`.
    pub fn get_floor_angle(&self, up_direction: Vector3) -> f32 {
        self.floor_normal.angle_to(up_direction)
    }

    pub fn set_wall_min_slide_angle(&mut self, radians: f32) {
        self.wall_min_slide_angle = radians;
    }

    pub fn get_wall_min_slide_angle(&self) -> f32 {
        self.wall_min_slide_angle
    }

    pub fn set_safe_margin(&mut self, margin: f32) {
        self.safe_margin = margin;
    }

    pub fn get_safe_margin(&self) -> f32 {
        self.safe_margin
    }

    pub fn set_up_direction(&mut self, up: Vector3) {
        self.up_direction = up;
    }

    pub fn get_up_direction(&self) -> Vector3 {
        self.up_direction
    }

    pub fn set_velocity(&mut self, velocity: Vector3) {
        self.velocity = velocity;
    }

    pub fn get_velocity(&self) -> Vector3 {
        self.velocity
    }

    /// The movement state is written by `move_and_slide` in the engine; the port
    /// exposes setters so callers can drive it.
    pub fn set_motion_state(
        &mut self,
        floor_normal: Vector3,
        wall_normal: Vector3,
        ceiling_normal: Vector3,
        last_motion: Vector3,
        real_velocity: Vector3,
    ) {
        self.floor_normal = floor_normal;
        self.wall_normal = wall_normal;
        self.ceiling_normal = ceiling_normal;
        self.last_motion = last_motion;
        self.real_velocity = real_velocity;
    }

    pub fn get_floor_normal(&self) -> Vector3 {
        self.floor_normal
    }

    pub fn get_wall_normal(&self) -> Vector3 {
        self.wall_normal
    }

    pub fn get_ceiling_normal(&self) -> Vector3 {
        self.ceiling_normal
    }

    pub fn get_last_motion(&self) -> Vector3 {
        self.last_motion
    }

    pub fn get_real_velocity(&self) -> Vector3 {
        self.real_velocity
    }

    pub fn set_platform_velocity(&mut self, velocity: Vector3) {
        self.platform_velocity = velocity;
    }

    pub fn get_platform_velocity(&self) -> Vector3 {
        self.platform_velocity
    }

    pub fn set_platform_angular_velocity(&mut self, velocity: Vector3) {
        self.platform_angular_velocity = velocity;
    }

    pub fn get_platform_angular_velocity(&self) -> Vector3 {
        self.platform_angular_velocity
    }
}

// -- StaticBody3D -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StaticBody3DData {
    constant_linear_velocity: Vector3,
    constant_angular_velocity: Vector3,
    physics_material_override: Option<Rid>,
}

impl Default for StaticBody3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticBody3DData {
    pub fn new() -> Self {
        Self {
            constant_linear_velocity: Vector3::ZERO,
            constant_angular_velocity: Vector3::ZERO,
            physics_material_override: None,
        }
    }

    pub fn set_constant_linear_velocity(&mut self, velocity: Vector3) {
        self.constant_linear_velocity = velocity;
    }

    pub fn get_constant_linear_velocity(&self) -> Vector3 {
        self.constant_linear_velocity
    }

    pub fn set_constant_angular_velocity(&mut self, velocity: Vector3) {
        self.constant_angular_velocity = velocity;
    }

    pub fn get_constant_angular_velocity(&self) -> Vector3 {
        self.constant_angular_velocity
    }

    pub fn set_physics_material_override(&mut self, material: Option<Rid>) {
        self.physics_material_override = material;
    }

    pub fn get_physics_material_override(&self) -> Option<Rid> {
        self.physics_material_override
    }
}

// -- RigidBody3D ------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FreezeMode {
    Static,
    Kinematic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CenterOfMassMode {
    Auto,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DampMode {
    Combine,
    Replace,
}

#[derive(Clone, Debug)]
pub struct RigidBody3DData {
    mass: f32,
    gravity_scale: f32,
    linear_damp_mode: DampMode,
    angular_damp_mode: DampMode,
    linear_damp: f32,
    angular_damp: f32,
    freeze: bool,
    freeze_mode: FreezeMode,
    ccd: bool,
    sleeping: bool,
    max_contacts_reported: i32,
    contact_count: i32,
    custom_integrator: bool,
    center_of_mass_mode: CenterOfMassMode,
    center_of_mass: Vector3,
    linear_velocity: Vector3,
    angular_velocity: Vector3,
    physics_material_override: Option<Rid>,
}

impl Default for RigidBody3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RigidBody3DData {
    pub fn new() -> Self {
        Self {
            mass: 1.0,
            gravity_scale: 1.0,
            linear_damp_mode: DampMode::Combine,
            angular_damp_mode: DampMode::Combine,
            linear_damp: 0.0,
            angular_damp: 0.0,
            freeze: false,
            freeze_mode: FreezeMode::Static,
            ccd: false,
            sleeping: false,
            max_contacts_reported: 0,
            contact_count: 0,
            custom_integrator: false,
            center_of_mass_mode: CenterOfMassMode::Auto,
            center_of_mass: Vector3::ZERO,
            linear_velocity: Vector3::ZERO,
            angular_velocity: Vector3::ZERO,
            physics_material_override: None,
        }
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn set_gravity_scale(&mut self, scale: f32) {
        self.gravity_scale = scale;
    }

    pub fn get_gravity_scale(&self) -> f32 {
        self.gravity_scale
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

    pub fn set_freeze_enabled(&mut self, freeze: bool) {
        self.freeze = freeze;
    }

    pub fn is_freeze_enabled(&self) -> bool {
        self.freeze
    }

    pub fn set_freeze_mode(&mut self, mode: FreezeMode) {
        self.freeze_mode = mode;
    }

    pub fn get_freeze_mode(&self) -> FreezeMode {
        self.freeze_mode
    }

    pub fn set_continuous_cd_enabled(&mut self, enabled: bool) {
        self.ccd = enabled;
    }

    pub fn is_continuous_cd_enabled(&self) -> bool {
        self.ccd
    }

    pub fn set_sleeping(&mut self, sleeping: bool) {
        self.sleeping = sleeping;
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    pub fn set_max_contacts_reported(&mut self, count: i32) {
        self.max_contacts_reported = count;
    }

    pub fn get_max_contacts_reported(&self) -> i32 {
        self.max_contacts_reported
    }

    pub fn get_contact_count(&self) -> i32 {
        self.contact_count
    }

    pub fn set_contact_count(&mut self, count: i32) {
        self.contact_count = count;
    }

    pub fn set_use_custom_integrator(&mut self, enabled: bool) {
        self.custom_integrator = enabled;
    }

    pub fn is_using_custom_integrator(&self) -> bool {
        self.custom_integrator
    }

    pub fn set_center_of_mass_mode(&mut self, mode: CenterOfMassMode) {
        self.center_of_mass_mode = mode;
    }

    pub fn get_center_of_mass_mode(&self) -> CenterOfMassMode {
        self.center_of_mass_mode
    }

    pub fn set_center_of_mass(&mut self, center: Vector3) {
        self.center_of_mass = center;
    }

    pub fn get_center_of_mass(&self) -> Vector3 {
        self.center_of_mass
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

    pub fn set_physics_material_override(&mut self, material: Option<Rid>) {
        self.physics_material_override = material;
    }

    pub fn get_physics_material_override(&self) -> Option<Rid> {
        self.physics_material_override
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_defaults() {
        let c = CharacterBody3DData::new();
        assert_eq!(c.get_motion_mode(), MotionMode::Grounded);
        assert_eq!(c.get_platform_on_leave(), PlatformOnLeave::AddVelocity);
        assert!(!c.is_floor_constant_speed_enabled());
        assert!(c.is_floor_stop_on_slope_enabled());
        assert!(c.is_floor_block_on_wall_enabled());
        assert!(c.is_slide_on_ceiling_enabled());
        assert_eq!(c.get_max_slides(), 6);
        assert_eq!(c.get_platform_floor_layers(), u32::MAX);
        assert_eq!(c.get_platform_wall_layers(), 0);
        assert_eq!(c.get_floor_snap_length(), 0.1);
        assert!((c.get_floor_max_angle() - deg_to_rad(45.0)).abs() < 1e-6);
        assert!((c.get_wall_min_slide_angle() - deg_to_rad(15.0)).abs() < 1e-6);
        assert_eq!(c.get_safe_margin(), 0.001);
        assert_eq!(c.get_up_direction(), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(c.get_velocity(), Vector3::ZERO);
        assert_eq!(c.get_floor_normal(), Vector3::ZERO);
        assert_eq!(c.get_real_velocity(), Vector3::ZERO);
    }

    #[test]
    fn character_validation_and_setters() {
        let mut c = CharacterBody3DData::new();
        assert!(!c.set_max_slides(0));
        assert_eq!(c.get_max_slides(), 6);
        assert!(c.set_max_slides(12));
        assert_eq!(c.get_max_slides(), 12);

        assert!(!c.set_floor_snap_length(-0.5));
        assert!(c.set_floor_snap_length(0.0));
        assert_eq!(c.get_floor_snap_length(), 0.0);

        c.set_motion_mode(MotionMode::Floating);
        c.set_platform_on_leave(PlatformOnLeave::DoNothing);
        c.set_floor_constant_speed_enabled(true);
        c.set_floor_stop_on_slope_enabled(false);
        c.set_floor_block_on_wall_enabled(false);
        c.set_slide_on_ceiling_enabled(false);
        c.set_platform_floor_layers(3);
        c.set_platform_wall_layers(7);
        c.set_floor_max_angle(0.5);
        c.set_wall_min_slide_angle(0.25);
        c.set_safe_margin(0.01);
        c.set_up_direction(Vector3::DOWN);
        c.set_velocity(Vector3::new(1.0, 0.0, 0.0));

        assert_eq!(c.get_motion_mode(), MotionMode::Floating);
        assert_eq!(c.get_platform_on_leave(), PlatformOnLeave::DoNothing);
        assert!(c.is_floor_constant_speed_enabled());
        assert!(!c.is_floor_stop_on_slope_enabled());
        assert!(!c.is_floor_block_on_wall_enabled());
        assert!(!c.is_slide_on_ceiling_enabled());
        assert_eq!(c.get_platform_floor_layers(), 3);
        assert_eq!(c.get_platform_wall_layers(), 7);
        assert_eq!(c.get_floor_max_angle(), 0.5);
        assert_eq!(c.get_wall_min_slide_angle(), 0.25);
        assert_eq!(c.get_safe_margin(), 0.01);
        assert_eq!(c.get_up_direction(), Vector3::DOWN);
        assert_eq!(c.get_velocity(), Vector3::new(1.0, 0.0, 0.0));

        assert_eq!(MotionMode::Grounded as i32, 0);
        assert_eq!(MotionMode::Floating as i32, 1);
        assert_eq!(PlatformOnLeave::AddVelocity as i32, 0);
        assert_eq!(PlatformOnLeave::AddUpwardVelocity as i32, 1);
        assert_eq!(PlatformOnLeave::DoNothing as i32, 2);
    }

    #[test]
    fn motion_state_and_floor_angle() {
        let mut c = CharacterBody3DData::new();
        c.set_motion_state(
            Vector3::UP,
            Vector3::RIGHT,
            Vector3::DOWN,
            Vector3::new(0.0, 0.5, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        );
        assert_eq!(c.get_floor_normal(), Vector3::UP);
        assert_eq!(c.get_wall_normal(), Vector3::RIGHT);
        assert_eq!(c.get_ceiling_normal(), Vector3::DOWN);
        assert_eq!(c.get_last_motion(), Vector3::new(0.0, 0.5, 0.0));
        assert_eq!(c.get_real_velocity(), Vector3::new(1.0, 0.0, 0.0));
        // A flat floor is 0 radians from up.
        assert!(c.get_floor_angle(Vector3::UP).abs() < 1e-6);

        c.set_platform_velocity(Vector3::new(0.0, 0.0, 1.0));
        c.set_platform_angular_velocity(Vector3::new(0.0, 2.0, 0.0));
        assert_eq!(c.get_platform_velocity(), Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(c.get_platform_angular_velocity(), Vector3::new(0.0, 2.0, 0.0));
    }

    #[test]
    fn static_body_defaults() {
        let mut s = StaticBody3DData::new();
        assert_eq!(s.get_constant_linear_velocity(), Vector3::ZERO);
        assert_eq!(s.get_constant_angular_velocity(), Vector3::ZERO);
        assert!(s.get_physics_material_override().is_none());

        s.set_constant_linear_velocity(Vector3::new(1.0, 2.0, 3.0));
        s.set_constant_angular_velocity(Vector3::new(0.0, 1.0, 0.0));
        s.set_physics_material_override(Some(Rid::from_u64(2)));
        assert_eq!(s.get_constant_linear_velocity(), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(s.get_constant_angular_velocity(), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(s.get_physics_material_override(), Some(Rid::from_u64(2)));
    }

    #[test]
    fn rigid_body_defaults_and_flags() {
        let mut r = RigidBody3DData::new();
        assert_eq!(r.get_mass(), 1.0);
        assert_eq!(r.get_gravity_scale(), 1.0);
        assert_eq!(r.get_linear_damp_mode(), DampMode::Combine);
        assert_eq!(r.get_angular_damp_mode(), DampMode::Combine);
        assert_eq!(r.get_linear_damp(), 0.0);
        assert_eq!(r.get_angular_damp(), 0.0);
        assert!(!r.is_freeze_enabled());
        assert_eq!(r.get_freeze_mode(), FreezeMode::Static);
        assert!(!r.is_continuous_cd_enabled());
        assert!(!r.is_sleeping());
        assert_eq!(r.get_max_contacts_reported(), 0);
        assert_eq!(r.get_contact_count(), 0);
        assert!(!r.is_using_custom_integrator());
        assert_eq!(r.get_center_of_mass_mode(), CenterOfMassMode::Auto);
        assert_eq!(r.get_center_of_mass(), Vector3::ZERO);
        assert_eq!(r.get_linear_velocity(), Vector3::ZERO);
        assert_eq!(r.get_angular_velocity(), Vector3::ZERO);
        assert!(r.get_physics_material_override().is_none());

        r.set_mass(2.5);
        r.set_gravity_scale(0.5);
        r.set_linear_damp_mode(DampMode::Replace);
        r.set_angular_damp_mode(DampMode::Replace);
        r.set_linear_damp(0.1);
        r.set_angular_damp(0.2);
        r.set_freeze_enabled(true);
        r.set_freeze_mode(FreezeMode::Kinematic);
        r.set_continuous_cd_enabled(true);
        r.set_sleeping(true);
        r.set_max_contacts_reported(8);
        r.set_contact_count(3);
        r.set_use_custom_integrator(true);
        r.set_center_of_mass_mode(CenterOfMassMode::Custom);
        r.set_center_of_mass(Vector3::new(0.0, 1.0, 0.0));
        r.set_linear_velocity(Vector3::new(1.0, 0.0, 0.0));
        r.set_angular_velocity(Vector3::new(0.0, 0.0, 2.0));

        assert_eq!(r.get_mass(), 2.5);
        assert_eq!(r.get_gravity_scale(), 0.5);
        assert_eq!(r.get_linear_damp_mode(), DampMode::Replace);
        assert_eq!(r.get_linear_damp(), 0.1);
        assert_eq!(r.get_angular_damp(), 0.2);
        assert!(r.is_freeze_enabled());
        assert_eq!(r.get_freeze_mode(), FreezeMode::Kinematic);
        assert!(r.is_continuous_cd_enabled());
        assert!(r.is_sleeping());
        assert_eq!(r.get_max_contacts_reported(), 8);
        assert_eq!(r.get_contact_count(), 3);
        assert!(r.is_using_custom_integrator());
        assert_eq!(r.get_center_of_mass_mode(), CenterOfMassMode::Custom);
        assert_eq!(r.get_center_of_mass(), Vector3::new(0.0, 1.0, 0.0));

        assert_eq!(FreezeMode::Static as i32, 0);
        assert_eq!(FreezeMode::Kinematic as i32, 1);
        assert_eq!(CenterOfMassMode::Auto as i32, 0);
        assert_eq!(CenterOfMassMode::Custom as i32, 1);
        assert_eq!(DampMode::Combine as i32, 0);
        assert_eq!(DampMode::Replace as i32, 1);
    }
}
