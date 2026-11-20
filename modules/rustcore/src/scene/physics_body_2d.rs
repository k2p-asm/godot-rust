//! `scene/2d/physics/{area_2d,character_body_2d,rigid_body_2d,static_body_2d}.*`
//! (state half).
//!
//! `Area2D`: space-override modes for gravity/linear damp/angular damp (all
//! disabled), gravity 980 with direction `(0,1)` (set by the constructor),
//! point-gravity off with unit distance 0, linear damp 0.1, angular damp 1,
//! priority 0, and monitoring/monitorable both **on** (the constructor enables
//! them, overriding the header's `false`).
//!
//! `CharacterBody2D`: margin 0.08, grounded motion, add-velocity on platform
//! leave, floor stop-on-slope/block-on-wall and slide-on-ceiling on, floor
//! constant speed off, four slides, floor maximum angle 45°, snap length 1, wall
//! minimum slide angle 15°, up `(0,-1)`, platform floor layers all and wall
//! layers none. `set_max_slides` rejects `< 1`; `set_floor_snap_length` rejects
//! negatives.
//!
//! `RigidBody2D` and `StaticBody2D` mirror their 3D counterparts plus the 2D
//! CCD mode.
//!
//! Physics servers, `move_and_slide` and the collision bookkeeping are not ported.

use crate::math::funcs::deg_to_rad;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

// -- Area2D -----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SpaceOverride {
    Disabled,
    Combine,
    CombineReplace,
    Replace,
    ReplaceCombine,
}

#[derive(Clone, Debug)]
pub struct Area2DData {
    gravity_space_override: SpaceOverride,
    linear_damp_space_override: SpaceOverride,
    angular_damp_space_override: SpaceOverride,
    gravity_vec: Vector2,
    gravity: f32,
    gravity_is_point: bool,
    gravity_point_unit_distance: f32,
    linear_damp: f32,
    angular_damp: f32,
    priority: i32,
    monitoring: bool,
    monitorable: bool,
}

impl Default for Area2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Area2DData {
    pub fn new() -> Self {
        Self {
            gravity_space_override: SpaceOverride::Disabled,
            linear_damp_space_override: SpaceOverride::Disabled,
            angular_damp_space_override: SpaceOverride::Disabled,
            gravity_vec: Vector2::new(0.0, 1.0),
            gravity: 980.0,
            gravity_is_point: false,
            gravity_point_unit_distance: 0.0,
            linear_damp: 0.1,
            angular_damp: 1.0,
            priority: 0,
            monitoring: true,
            monitorable: true,
        }
    }

    pub fn set_gravity_space_override_mode(&mut self, mode: SpaceOverride) {
        self.gravity_space_override = mode;
    }

    pub fn get_gravity_space_override_mode(&self) -> SpaceOverride {
        self.gravity_space_override
    }

    pub fn set_linear_damp_space_override_mode(&mut self, mode: SpaceOverride) {
        self.linear_damp_space_override = mode;
    }

    pub fn get_linear_damp_space_override_mode(&self) -> SpaceOverride {
        self.linear_damp_space_override
    }

    pub fn set_angular_damp_space_override_mode(&mut self, mode: SpaceOverride) {
        self.angular_damp_space_override = mode;
    }

    pub fn get_angular_damp_space_override_mode(&self) -> SpaceOverride {
        self.angular_damp_space_override
    }

    pub fn set_gravity_direction(&mut self, direction: Vector2) {
        self.gravity_vec = direction;
    }

    pub fn get_gravity_direction(&self) -> Vector2 {
        self.gravity_vec
    }

    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    pub fn get_gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_gravity_point(&mut self, is_point: bool) {
        self.gravity_is_point = is_point;
    }

    pub fn is_gravity_point(&self) -> bool {
        self.gravity_is_point
    }

    pub fn set_gravity_point_unit_distance(&mut self, distance: f32) {
        self.gravity_point_unit_distance = distance;
    }

    pub fn get_gravity_point_unit_distance(&self) -> f32 {
        self.gravity_point_unit_distance
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

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    pub fn get_priority(&self) -> i32 {
        self.priority
    }

    pub fn set_monitoring(&mut self, monitoring: bool) {
        self.monitoring = monitoring;
    }

    pub fn is_monitoring(&self) -> bool {
        self.monitoring
    }

    pub fn set_monitorable(&mut self, monitorable: bool) {
        self.monitorable = monitorable;
    }

    pub fn is_monitorable(&self) -> bool {
        self.monitorable
    }
}

// -- CharacterBody2D --------------------------------------------------------

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
pub struct CharacterBody2DData {
    margin: f32,
    motion_mode: MotionMode,
    platform_on_leave: PlatformOnLeave,
    floor_constant_speed: bool,
    floor_stop_on_slope: bool,
    floor_block_on_wall: bool,
    slide_on_ceiling: bool,
    max_slides: i32,
    floor_max_angle: f32,
    floor_snap_length: f32,
    wall_min_slide_angle: f32,
    up_direction: Vector2,
    platform_floor_layers: u32,
    platform_wall_layers: u32,
    velocity: Vector2,
    floor_normal: Vector2,
    wall_normal: Vector2,
    last_motion: Vector2,
    platform_velocity: Vector2,
    real_velocity: Vector2,
}

impl Default for CharacterBody2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterBody2DData {
    pub fn new() -> Self {
        Self {
            margin: 0.08,
            motion_mode: MotionMode::Grounded,
            platform_on_leave: PlatformOnLeave::AddVelocity,
            floor_constant_speed: false,
            floor_stop_on_slope: true,
            floor_block_on_wall: true,
            slide_on_ceiling: true,
            max_slides: 4,
            floor_max_angle: deg_to_rad(45.0),
            floor_snap_length: 1.0,
            wall_min_slide_angle: deg_to_rad(15.0),
            up_direction: Vector2::new(0.0, -1.0),
            platform_floor_layers: u32::MAX,
            platform_wall_layers: 0,
            velocity: Vector2::ZERO,
            floor_normal: Vector2::ZERO,
            wall_normal: Vector2::ZERO,
            last_motion: Vector2::ZERO,
            platform_velocity: Vector2::ZERO,
            real_velocity: Vector2::ZERO,
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

    pub fn set_floor_max_angle(&mut self, radians: f32) {
        self.floor_max_angle = radians;
    }

    pub fn get_floor_max_angle(&self) -> f32 {
        self.floor_max_angle
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

    pub fn set_wall_min_slide_angle(&mut self, radians: f32) {
        self.wall_min_slide_angle = radians;
    }

    pub fn get_wall_min_slide_angle(&self) -> f32 {
        self.wall_min_slide_angle
    }

    pub fn set_up_direction(&mut self, up: Vector2) {
        self.up_direction = up;
    }

    pub fn get_up_direction(&self) -> Vector2 {
        self.up_direction
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

    pub fn set_safe_margin(&mut self, margin: f32) {
        self.margin = margin;
    }

    pub fn get_safe_margin(&self) -> f32 {
        self.margin
    }

    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    pub fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    /// `get_floor_angle`: angle between the floor normal and `up`.
    pub fn get_floor_angle(&self, up_direction: Vector2) -> f32 {
        self.floor_normal.angle_to(up_direction)
    }

    pub fn set_motion_state(
        &mut self,
        floor_normal: Vector2,
        wall_normal: Vector2,
        last_motion: Vector2,
        real_velocity: Vector2,
    ) {
        self.floor_normal = floor_normal;
        self.wall_normal = wall_normal;
        self.last_motion = last_motion;
        self.real_velocity = real_velocity;
    }

    pub fn get_floor_normal(&self) -> Vector2 {
        self.floor_normal
    }

    pub fn get_wall_normal(&self) -> Vector2 {
        self.wall_normal
    }

    pub fn get_last_motion(&self) -> Vector2 {
        self.last_motion
    }

    pub fn get_real_velocity(&self) -> Vector2 {
        self.real_velocity
    }

    pub fn set_platform_velocity(&mut self, velocity: Vector2) {
        self.platform_velocity = velocity;
    }

    pub fn get_platform_velocity(&self) -> Vector2 {
        self.platform_velocity
    }
}

// -- StaticBody2D -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StaticBody2DData {
    constant_linear_velocity: Vector2,
    constant_angular_velocity: f32,
    physics_material_override: Option<Rid>,
}

impl Default for StaticBody2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticBody2DData {
    pub fn new() -> Self {
        Self {
            constant_linear_velocity: Vector2::ZERO,
            constant_angular_velocity: 0.0,
            physics_material_override: None,
        }
    }

    pub fn set_constant_linear_velocity(&mut self, velocity: Vector2) {
        self.constant_linear_velocity = velocity;
    }

    pub fn get_constant_linear_velocity(&self) -> Vector2 {
        self.constant_linear_velocity
    }

    pub fn set_constant_angular_velocity(&mut self, velocity: f32) {
        self.constant_angular_velocity = velocity;
    }

    pub fn get_constant_angular_velocity(&self) -> f32 {
        self.constant_angular_velocity
    }

    pub fn set_physics_material_override(&mut self, material: Option<Rid>) {
        self.physics_material_override = material;
    }

    pub fn get_physics_material_override(&self) -> Option<Rid> {
        self.physics_material_override
    }
}

// -- RigidBody2D ------------------------------------------------------------

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CcdMode {
    Disabled,
    CastRay,
    CastShape,
    Max,
}

#[derive(Clone, Debug)]
pub struct RigidBody2DData {
    mass: f32,
    gravity_scale: f32,
    linear_damp_mode: DampMode,
    angular_damp_mode: DampMode,
    linear_damp: f32,
    angular_damp: f32,
    freeze: bool,
    freeze_mode: FreezeMode,
    ccd_mode: CcdMode,
    sleeping: bool,
    max_contacts_reported: i32,
    contact_count: i32,
    custom_integrator: bool,
    center_of_mass_mode: CenterOfMassMode,
    center_of_mass: Vector2,
    linear_velocity: Vector2,
    angular_velocity: f32,
    physics_material_override: Option<Rid>,
}

impl Default for RigidBody2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RigidBody2DData {
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
            ccd_mode: CcdMode::Disabled,
            sleeping: false,
            max_contacts_reported: 0,
            contact_count: 0,
            custom_integrator: false,
            center_of_mass_mode: CenterOfMassMode::Auto,
            center_of_mass: Vector2::ZERO,
            linear_velocity: Vector2::ZERO,
            angular_velocity: 0.0,
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

    /// `ERR_FAIL_INDEX(mode, CCD_MODE_MAX)` for values past the table.
    pub fn set_continuous_collision_detection_mode(&mut self, mode: CcdMode) -> bool {
        if mode == CcdMode::Max {
            return false;
        }
        self.ccd_mode = mode;
        true
    }

    pub fn get_continuous_collision_detection_mode(&self) -> CcdMode {
        self.ccd_mode
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

    pub fn set_center_of_mass(&mut self, center: Vector2) {
        self.center_of_mass = center;
    }

    pub fn get_center_of_mass(&self) -> Vector2 {
        self.center_of_mass
    }

    pub fn set_linear_velocity(&mut self, velocity: Vector2) {
        self.linear_velocity = velocity;
    }

    pub fn get_linear_velocity(&self) -> Vector2 {
        self.linear_velocity
    }

    pub fn set_angular_velocity(&mut self, velocity: f32) {
        self.angular_velocity = velocity;
    }

    pub fn get_angular_velocity(&self) -> f32 {
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
    fn area_defaults() {
        let mut a = Area2DData::new();
        assert_eq!(a.get_gravity_space_override_mode(), SpaceOverride::Disabled);
        assert_eq!(a.get_linear_damp_space_override_mode(), SpaceOverride::Disabled);
        assert_eq!(a.get_angular_damp_space_override_mode(), SpaceOverride::Disabled);
        assert_eq!(a.get_gravity(), 980.0, "the constructor sets 980");
        assert_eq!(a.get_gravity_direction(), Vector2::new(0.0, 1.0));
        assert!(!a.is_gravity_point());
        assert_eq!(a.get_gravity_point_unit_distance(), 0.0);
        assert_eq!(a.get_linear_damp(), 0.1);
        assert_eq!(a.get_angular_damp(), 1.0);
        assert_eq!(a.get_priority(), 0);
        assert!(a.is_monitoring(), "the constructor enables monitoring");
        assert!(a.is_monitorable(), "the constructor enables monitorable");

        a.set_gravity_space_override_mode(SpaceOverride::ReplaceCombine);
        a.set_linear_damp_space_override_mode(SpaceOverride::Combine);
        a.set_angular_damp_space_override_mode(SpaceOverride::Replace);
        a.set_gravity_direction(Vector2::DOWN);
        a.set_gravity(10.0);
        a.set_gravity_point(true);
        a.set_gravity_point_unit_distance(2.0);
        a.set_linear_damp(0.5);
        a.set_angular_damp(0.25);
        a.set_priority(3);
        a.set_monitoring(false);
        a.set_monitorable(false);

        assert_eq!(a.get_gravity_space_override_mode(), SpaceOverride::ReplaceCombine);
        assert_eq!(a.get_linear_damp_space_override_mode(), SpaceOverride::Combine);
        assert_eq!(a.get_angular_damp_space_override_mode(), SpaceOverride::Replace);
        assert_eq!(a.get_gravity_direction(), Vector2::DOWN);
        assert_eq!(a.get_gravity(), 10.0);
        assert!(a.is_gravity_point());
        assert_eq!(a.get_gravity_point_unit_distance(), 2.0);
        assert_eq!(a.get_linear_damp(), 0.5);
        assert_eq!(a.get_angular_damp(), 0.25);
        assert_eq!(a.get_priority(), 3);
        assert!(!a.is_monitoring());
        assert!(!a.is_monitorable());

        assert_eq!(SpaceOverride::Disabled as i32, 0);
        assert_eq!(SpaceOverride::Combine as i32, 1);
        assert_eq!(SpaceOverride::CombineReplace as i32, 2);
        assert_eq!(SpaceOverride::Replace as i32, 3);
        assert_eq!(SpaceOverride::ReplaceCombine as i32, 4);
    }

    #[test]
    fn character_defaults() {
        let c = CharacterBody2DData::new();
        assert_eq!(c.get_safe_margin(), 0.08);
        assert_eq!(c.get_motion_mode(), MotionMode::Grounded);
        assert_eq!(c.get_platform_on_leave(), PlatformOnLeave::AddVelocity);
        assert!(!c.is_floor_constant_speed_enabled());
        assert!(c.is_floor_stop_on_slope_enabled());
        assert!(c.is_floor_block_on_wall_enabled());
        assert!(c.is_slide_on_ceiling_enabled());
        assert_eq!(c.get_max_slides(), 4);
        assert!((c.get_floor_max_angle() - deg_to_rad(45.0)).abs() < 1e-6);
        assert_eq!(c.get_floor_snap_length(), 1.0);
        assert!((c.get_wall_min_slide_angle() - deg_to_rad(15.0)).abs() < 1e-6);
        assert_eq!(c.get_up_direction(), Vector2::new(0.0, -1.0));
        assert_eq!(c.get_platform_floor_layers(), u32::MAX);
        assert_eq!(c.get_platform_wall_layers(), 0);
        assert_eq!(c.get_velocity(), Vector2::ZERO);
    }

    #[test]
    fn character_validation_and_motion_state() {
        let mut c = CharacterBody2DData::new();
        assert!(!c.set_max_slides(0));
        assert_eq!(c.get_max_slides(), 4);
        assert!(c.set_max_slides(8));
        assert!(!c.set_floor_snap_length(-0.1));
        assert!(c.set_floor_snap_length(0.0));

        c.set_motion_mode(MotionMode::Floating);
        c.set_platform_on_leave(PlatformOnLeave::DoNothing);
        c.set_up_direction(Vector2::UP);
        c.set_velocity(Vector2::new(3.0, 0.0));
        c.set_motion_state(
            Vector2::UP,
            Vector2::RIGHT,
            Vector2::new(0.0, -1.0),
            Vector2::new(3.0, 0.0),
        );
        assert_eq!(c.get_motion_mode(), MotionMode::Floating);
        assert_eq!(c.get_platform_on_leave(), PlatformOnLeave::DoNothing);
        assert_eq!(c.get_up_direction(), Vector2::UP);
        assert_eq!(c.get_velocity(), Vector2::new(3.0, 0.0));
        assert_eq!(c.get_floor_normal(), Vector2::UP);
        assert_eq!(c.get_wall_normal(), Vector2::RIGHT);
        assert_eq!(c.get_last_motion(), Vector2::new(0.0, -1.0));
        assert_eq!(c.get_real_velocity(), Vector2::new(3.0, 0.0));
        assert!(c.get_floor_angle(Vector2::UP).abs() < 1e-6);

        c.set_platform_velocity(Vector2::new(0.0, 1.0));
        assert_eq!(c.get_platform_velocity(), Vector2::new(0.0, 1.0));
        assert_eq!(MotionMode::Grounded as i32, 0);
        assert_eq!(PlatformOnLeave::DoNothing as i32, 2);
    }

    #[test]
    fn static_and_rigid_defaults() {
        let mut s = StaticBody2DData::new();
        assert_eq!(s.get_constant_linear_velocity(), Vector2::ZERO);
        assert_eq!(s.get_constant_angular_velocity(), 0.0);
        assert!(s.get_physics_material_override().is_none());
        s.set_constant_linear_velocity(Vector2::new(1.0, 2.0));
        s.set_constant_angular_velocity(1.5);
        assert_eq!(s.get_constant_linear_velocity(), Vector2::new(1.0, 2.0));
        assert_eq!(s.get_constant_angular_velocity(), 1.5);

        let mut r = RigidBody2DData::new();
        assert_eq!(r.get_mass(), 1.0);
        assert_eq!(r.get_gravity_scale(), 1.0);
        assert_eq!(r.get_linear_damp_mode(), DampMode::Combine);
        assert_eq!(r.get_angular_damp_mode(), DampMode::Combine);
        assert_eq!(r.get_linear_damp(), 0.0);
        assert_eq!(r.get_angular_damp(), 0.0);
        assert!(!r.is_freeze_enabled());
        assert_eq!(r.get_freeze_mode(), FreezeMode::Static);
        assert_eq!(r.get_continuous_collision_detection_mode(), CcdMode::Disabled);
        assert!(!r.is_sleeping());
        assert_eq!(r.get_max_contacts_reported(), 0);
        assert_eq!(r.get_contact_count(), 0);
        assert!(!r.is_using_custom_integrator());
        assert_eq!(r.get_center_of_mass_mode(), CenterOfMassMode::Auto);
        assert_eq!(r.get_center_of_mass(), Vector2::ZERO);
        assert_eq!(r.get_linear_velocity(), Vector2::ZERO);
        assert_eq!(r.get_angular_velocity(), 0.0);
        assert!(r.get_physics_material_override().is_none());
    }

    #[test]
    fn rigid_setters_and_ccd() {
        let mut r = RigidBody2DData::new();
        r.set_mass(3.0);
        r.set_gravity_scale(-1.0);
        r.set_linear_damp_mode(DampMode::Replace);
        r.set_angular_damp_mode(DampMode::Replace);
        r.set_linear_damp(0.5);
        r.set_angular_damp(0.75);
        r.set_freeze_enabled(true);
        r.set_freeze_mode(FreezeMode::Kinematic);
        assert!(r.set_continuous_collision_detection_mode(CcdMode::CastShape));
        assert!(!r.set_continuous_collision_detection_mode(CcdMode::Max));
        assert_eq!(r.get_continuous_collision_detection_mode(), CcdMode::CastShape);
        r.set_sleeping(true);
        r.set_max_contacts_reported(4);
        r.set_contact_count(2);
        r.set_use_custom_integrator(true);
        r.set_center_of_mass_mode(CenterOfMassMode::Custom);
        r.set_center_of_mass(Vector2::new(1.0, 1.0));
        r.set_linear_velocity(Vector2::new(2.0, 0.0));
        r.set_angular_velocity(1.25);
        r.set_physics_material_override(Some(Rid::from_u64(7)));

        assert_eq!(r.get_mass(), 3.0);
        assert_eq!(r.get_gravity_scale(), -1.0);
        assert_eq!(r.get_linear_damp_mode(), DampMode::Replace);
        assert_eq!(r.get_angular_damp_mode(), DampMode::Replace);
        assert_eq!(r.get_linear_damp(), 0.5);
        assert_eq!(r.get_angular_damp(), 0.75);
        assert!(r.is_freeze_enabled());
        assert_eq!(r.get_freeze_mode(), FreezeMode::Kinematic);
        assert!(r.is_sleeping());
        assert_eq!(r.get_max_contacts_reported(), 4);
        assert_eq!(r.get_contact_count(), 2);
        assert!(r.is_using_custom_integrator());
        assert_eq!(r.get_center_of_mass_mode(), CenterOfMassMode::Custom);
        assert_eq!(r.get_center_of_mass(), Vector2::new(1.0, 1.0));
        assert_eq!(r.get_linear_velocity(), Vector2::new(2.0, 0.0));
        assert_eq!(r.get_angular_velocity(), 1.25);
        assert_eq!(r.get_physics_material_override(), Some(Rid::from_u64(7)));

        assert_eq!(CcdMode::Disabled as i32, 0);
        assert_eq!(CcdMode::CastRay as i32, 1);
        assert_eq!(CcdMode::CastShape as i32, 2);
        assert_eq!(CcdMode::Max as i32, 3);
    }
}
