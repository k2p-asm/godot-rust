//! `scene/2d/cpu_particles_2d.*`: CPU particle emitter parameters.
//!
//! The constructor enables emission with 8 particles, world coordinates, the
//! engine's per-parameter defaults (`scale` min/max 1, everything else 0), all
//! particle flags off and a white colour. The remaining header defaults are kept
//! too: lifetime 1, pre-process/explosiveness/randomness/lifetime-randomness 0,
//! speed scale 1, fixed FPS 0 with fractional delta on, index draw order,
//! direction `(1,0)`, spread 45, point emission with sphere radius 1, rect
//! extents `(1,1)`, ring inner radius 0.8 and radius 1, and gravity `(0,980)`.
//!
//! `amount >= 1` and `lifetime > 0` are enforced; the parameter tables reject
//! the `PARAM_MAX` sentinel, and curves/gradients are stored by value.
//!
//! The simulation (`_particles_process`) and the multimesh buffer are not ported.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::scene::curve::Curve;
use crate::scene::gradient::Gradient;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawOrder {
    Index,
    Lifetime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Parameter {
    InitialLinearVelocity,
    AngularVelocity,
    OrbitVelocity,
    LinearAccel,
    RadialAccel,
    TangentialAccel,
    Damping,
    Angle,
    Scale,
    HueVariation,
    AnimSpeed,
    AnimOffset,
    Max,
}

pub const PARAM_MAX: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ParticleFlags {
    AlignYToVelocity,
    RotateY,
    DisableZ,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EmissionShape {
    Point,
    Sphere,
    SphereSurface,
    Rectangle,
    Points,
    DirectedPoints,
    Ring,
    Max,
}

#[derive(Clone, Debug)]
pub struct CpuParticles2DData {
    emitting: bool,
    one_shot: bool,
    amount: i32,
    lifetime: f64,
    pre_process_time: f64,
    explosiveness_ratio: f32,
    randomness_ratio: f32,
    lifetime_randomness: f64,
    speed_scale: f64,
    local_coords: bool,
    fixed_fps: i32,
    fractional_delta: bool,
    seed: u32,
    use_fixed_seed: bool,
    draw_order: DrawOrder,
    texture: Option<Rid>,
    direction: Vector2,
    spread: f32,
    params_min: [f32; PARAM_MAX],
    params_max: [f32; PARAM_MAX],
    curves: Vec<Option<Curve>>,
    color: Color,
    color_ramp: Option<Gradient>,
    color_initial_ramp: Option<Gradient>,
    flags: [bool; 3],
    emission_shape: EmissionShape,
    emission_sphere_radius: f32,
    emission_rect_extents: Vector2,
    emission_points: Vec<Vector2>,
    emission_normals: Vec<Vector2>,
    emission_colors: Vec<Color>,
    emission_ring_inner_radius: f32,
    emission_ring_radius: f32,
    scale_curve_x: Option<Curve>,
    scale_curve_y: Option<Curve>,
    split_scale: bool,
    gravity: Vector2,
}

impl Default for CpuParticles2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuParticles2DData {
    pub fn new() -> Self {
        let mut params_min = [0.0f32; PARAM_MAX];
        let mut params_max = [0.0f32; PARAM_MAX];
        params_min[Parameter::Scale as usize] = 1.0;
        params_max[Parameter::Scale as usize] = 1.0;

        Self {
            emitting: true,
            one_shot: false,
            amount: 8,
            lifetime: 1.0,
            pre_process_time: 0.0,
            explosiveness_ratio: 0.0,
            randomness_ratio: 0.0,
            lifetime_randomness: 0.0,
            speed_scale: 1.0,
            local_coords: false,
            fixed_fps: 0,
            fractional_delta: true,
            seed: 0,
            use_fixed_seed: false,
            draw_order: DrawOrder::Index,
            texture: None,
            direction: Vector2::new(1.0, 0.0),
            spread: 45.0,
            params_min,
            params_max,
            curves: vec![None; PARAM_MAX],
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            color_ramp: None,
            color_initial_ramp: None,
            flags: [false; 3],
            emission_shape: EmissionShape::Point,
            emission_sphere_radius: 1.0,
            emission_rect_extents: Vector2::new(1.0, 1.0),
            emission_points: Vec::new(),
            emission_normals: Vec::new(),
            emission_colors: Vec::new(),
            emission_ring_inner_radius: 0.8,
            emission_ring_radius: 1.0,
            scale_curve_x: None,
            scale_curve_y: None,
            split_scale: false,
            gravity: Vector2::new(0.0, 980.0),
        }
    }

    pub fn set_emitting(&mut self, emitting: bool) {
        self.emitting = emitting;
    }

    pub fn is_emitting(&self) -> bool {
        self.emitting
    }

    /// `ERR_FAIL_COND_MSG(amount < 1, ...)`.
    pub fn set_amount(&mut self, amount: i32) -> bool {
        if amount < 1 {
            return false;
        }
        self.amount = amount;
        true
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    /// `ERR_FAIL_COND_MSG(lifetime <= 0, ...)`.
    pub fn set_lifetime(&mut self, lifetime: f64) -> bool {
        if lifetime <= 0.0 {
            return false;
        }
        self.lifetime = lifetime;
        true
    }

    pub fn get_lifetime(&self) -> f64 {
        self.lifetime
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn get_one_shot(&self) -> bool {
        self.one_shot
    }

    pub fn set_pre_process_time(&mut self, time: f64) {
        self.pre_process_time = time;
    }

    pub fn get_pre_process_time(&self) -> f64 {
        self.pre_process_time
    }

    pub fn set_explosiveness_ratio(&mut self, ratio: f32) {
        self.explosiveness_ratio = ratio;
    }

    pub fn get_explosiveness_ratio(&self) -> f32 {
        self.explosiveness_ratio
    }

    pub fn set_randomness_ratio(&mut self, ratio: f32) {
        self.randomness_ratio = ratio;
    }

    pub fn get_randomness_ratio(&self) -> f32 {
        self.randomness_ratio
    }

    pub fn set_lifetime_randomness(&mut self, randomness: f64) {
        self.lifetime_randomness = randomness;
    }

    pub fn get_lifetime_randomness(&self) -> f64 {
        self.lifetime_randomness
    }

    pub fn set_speed_scale(&mut self, scale: f64) {
        self.speed_scale = scale;
    }

    pub fn get_speed_scale(&self) -> f64 {
        self.speed_scale
    }

    pub fn set_use_local_coordinates(&mut self, local: bool) {
        self.local_coords = local;
    }

    pub fn is_using_local_coordinates(&self) -> bool {
        self.local_coords
    }

    pub fn set_fixed_fps(&mut self, fps: i32) {
        self.fixed_fps = fps;
    }

    pub fn get_fixed_fps(&self) -> i32 {
        self.fixed_fps
    }

    pub fn set_fractional_delta(&mut self, enable: bool) {
        self.fractional_delta = enable;
    }

    pub fn get_fractional_delta(&self) -> bool {
        self.fractional_delta
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    pub fn set_use_fixed_seed(&mut self, use_fixed: bool) {
        self.use_fixed_seed = use_fixed;
    }

    pub fn is_using_fixed_seed(&self) -> bool {
        self.use_fixed_seed
    }

    pub fn set_draw_order(&mut self, order: DrawOrder) {
        self.draw_order = order;
    }

    pub fn get_draw_order(&self) -> DrawOrder {
        self.draw_order
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_direction(&mut self, direction: Vector2) {
        self.direction = direction;
    }

    pub fn get_direction(&self) -> Vector2 {
        self.direction
    }

    pub fn set_spread(&mut self, spread: f32) {
        self.spread = spread;
    }

    pub fn get_spread(&self) -> f32 {
        self.spread
    }

    /// Rejects the `PARAM_MAX` sentinel.
    pub fn set_param_min(&mut self, param: Parameter, value: f32) -> bool {
        if param == Parameter::Max {
            return false;
        }
        self.params_min[param as usize] = value;
        true
    }

    pub fn get_param_min(&self, param: Parameter) -> Option<f32> {
        if param == Parameter::Max {
            return None;
        }
        Some(self.params_min[param as usize])
    }

    pub fn set_param_max(&mut self, param: Parameter, value: f32) -> bool {
        if param == Parameter::Max {
            return false;
        }
        self.params_max[param as usize] = value;
        true
    }

    pub fn get_param_max(&self, param: Parameter) -> Option<f32> {
        if param == Parameter::Max {
            return None;
        }
        Some(self.params_max[param as usize])
    }

    pub fn set_param_curve(&mut self, param: Parameter, curve: Option<Curve>) -> bool {
        if param == Parameter::Max {
            return false;
        }
        self.curves[param as usize] = curve;
        true
    }

    pub fn get_param_curve(&self, param: Parameter) -> Option<&Curve> {
        if param == Parameter::Max {
            return None;
        }
        self.curves[param as usize].as_ref()
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_color_ramp(&mut self, ramp: Option<Gradient>) {
        self.color_ramp = ramp;
    }

    pub fn get_color_ramp(&self) -> Option<&Gradient> {
        self.color_ramp.as_ref()
    }

    pub fn set_color_initial_ramp(&mut self, ramp: Option<Gradient>) {
        self.color_initial_ramp = ramp;
    }

    pub fn get_color_initial_ramp(&self) -> Option<&Gradient> {
        self.color_initial_ramp.as_ref()
    }

    pub fn set_particle_flag(&mut self, flag: ParticleFlags, enabled: bool) -> bool {
        if flag == ParticleFlags::Max {
            return false;
        }
        self.flags[flag as i32 as usize] = enabled;
        true
    }

    pub fn get_particle_flag(&self, flag: ParticleFlags) -> bool {
        if flag == ParticleFlags::Max {
            return false;
        }
        self.flags[flag as i32 as usize]
    }

    pub fn set_emission_shape(&mut self, shape: EmissionShape) -> bool {
        if shape == EmissionShape::Max {
            return false;
        }
        self.emission_shape = shape;
        true
    }

    pub fn get_emission_shape(&self) -> EmissionShape {
        self.emission_shape
    }

    pub fn set_emission_sphere_radius(&mut self, radius: f32) {
        self.emission_sphere_radius = radius;
    }

    pub fn get_emission_sphere_radius(&self) -> f32 {
        self.emission_sphere_radius
    }

    pub fn set_emission_rect_extents(&mut self, extents: Vector2) {
        self.emission_rect_extents = extents;
    }

    pub fn get_emission_rect_extents(&self) -> Vector2 {
        self.emission_rect_extents
    }

    pub fn set_emission_points(&mut self, points: Vec<Vector2>) {
        self.emission_points = points;
    }

    pub fn get_emission_points(&self) -> &[Vector2] {
        &self.emission_points
    }

    pub fn set_emission_normals(&mut self, normals: Vec<Vector2>) {
        self.emission_normals = normals;
    }

    pub fn get_emission_normals(&self) -> &[Vector2] {
        &self.emission_normals
    }

    pub fn set_emission_colors(&mut self, colors: Vec<Color>) {
        self.emission_colors = colors;
    }

    pub fn get_emission_colors(&self) -> &[Color] {
        &self.emission_colors
    }

    pub fn set_emission_ring_inner_radius(&mut self, radius: f32) {
        self.emission_ring_inner_radius = radius;
    }

    pub fn get_emission_ring_inner_radius(&self) -> f32 {
        self.emission_ring_inner_radius
    }

    pub fn set_emission_ring_radius(&mut self, radius: f32) {
        self.emission_ring_radius = radius;
    }

    pub fn get_emission_ring_radius(&self) -> f32 {
        self.emission_ring_radius
    }

    pub fn set_scale_curve_x(&mut self, curve: Option<Curve>) {
        self.scale_curve_x = curve;
    }

    pub fn get_scale_curve_x(&self) -> Option<&Curve> {
        self.scale_curve_x.as_ref()
    }

    pub fn set_scale_curve_y(&mut self, curve: Option<Curve>) {
        self.scale_curve_y = curve;
    }

    pub fn get_scale_curve_y(&self) -> Option<&Curve> {
        self.scale_curve_y.as_ref()
    }

    pub fn set_split_scale(&mut self, split: bool) {
        self.split_scale = split;
    }

    pub fn is_scale_split(&self) -> bool {
        self.split_scale
    }

    pub fn set_gravity(&mut self, gravity: Vector2) {
        self.gravity = gravity;
    }

    pub fn get_gravity(&self) -> Vector2 {
        self.gravity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_defaults() {
        let p = CpuParticles2DData::new();
        assert!(p.is_emitting());
        assert_eq!(p.get_amount(), 8);
        assert_eq!(p.get_lifetime(), 1.0);
        assert!(!p.is_using_local_coordinates());
        assert_eq!(p.get_draw_order(), DrawOrder::Index);
        assert_eq!(p.get_direction(), Vector2::new(1.0, 0.0));
        assert_eq!(p.get_spread(), 45.0);
        assert_eq!(p.get_gravity(), Vector2::new(0.0, 980.0));
        assert_eq!(p.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(p.get_emission_shape(), EmissionShape::Point);
        assert_eq!(p.get_emission_sphere_radius(), 1.0);
        assert_eq!(p.get_emission_rect_extents(), Vector2::new(1.0, 1.0));
        assert_eq!(p.get_emission_ring_inner_radius(), 0.8);
        assert_eq!(p.get_emission_ring_radius(), 1.0);
        assert!(!p.is_scale_split());
        assert_eq!(p.get_speed_scale(), 1.0);
        assert_eq!(p.get_fixed_fps(), 0);
        assert!(p.get_fractional_delta());
        assert!(!p.is_using_fixed_seed());

        // Only `scale` starts at 1, everything else at 0.
        assert_eq!(p.get_param_min(Parameter::Scale), Some(1.0));
        assert_eq!(p.get_param_max(Parameter::Scale), Some(1.0));
        for param in [
            Parameter::InitialLinearVelocity,
            Parameter::AngularVelocity,
            Parameter::OrbitVelocity,
            Parameter::LinearAccel,
            Parameter::RadialAccel,
            Parameter::TangentialAccel,
            Parameter::Damping,
            Parameter::Angle,
            Parameter::HueVariation,
            Parameter::AnimSpeed,
            Parameter::AnimOffset,
        ] {
            assert_eq!(p.get_param_min(param), Some(0.0), "{param:?}");
            assert_eq!(p.get_param_max(param), Some(0.0), "{param:?}");
        }
        for flag in [
            ParticleFlags::AlignYToVelocity,
            ParticleFlags::RotateY,
            ParticleFlags::DisableZ,
        ] {
            assert!(!p.get_particle_flag(flag));
        }
    }

    #[test]
    fn amount_and_lifetime_validation() {
        let mut p = CpuParticles2DData::new();
        assert!(!p.set_amount(0));
        assert!(!p.set_amount(-3));
        assert_eq!(p.get_amount(), 8);
        assert!(p.set_amount(1));
        assert_eq!(p.get_amount(), 1);

        assert!(!p.set_lifetime(0.0));
        assert!(!p.set_lifetime(-2.0));
        assert!(p.set_lifetime(0.1));
        assert_eq!(p.get_lifetime(), 0.1);
    }

    #[test]
    fn parameter_tables_and_sentinels() {
        let mut p = CpuParticles2DData::new();
        assert!(p.set_param_min(Parameter::InitialLinearVelocity, 10.0));
        assert!(p.set_param_max(Parameter::InitialLinearVelocity, 20.0));
        assert_eq!(p.get_param_min(Parameter::InitialLinearVelocity), Some(10.0));
        assert_eq!(p.get_param_max(Parameter::InitialLinearVelocity), Some(20.0));

        assert!(!p.set_param_min(Parameter::Max, 1.0));
        assert!(!p.set_param_max(Parameter::Max, 1.0));
        assert_eq!(p.get_param_min(Parameter::Max), None);
        assert_eq!(p.get_param_max(Parameter::Max), None);

        let mut curve = Curve::new();
        curve.add_point(Vector2::new(0.0, 0.0));
        assert!(p.set_param_curve(Parameter::Scale, Some(curve)));
        assert!(p.get_param_curve(Parameter::Scale).is_some());
        assert!(p.get_param_curve(Parameter::Damping).is_none());
        assert!(!p.set_param_curve(Parameter::Max, None));
        assert!(p.get_param_curve(Parameter::Max).is_none());
    }

    #[test]
    fn emission_settings() {
        let mut p = CpuParticles2DData::new();
        assert!(p.set_emission_shape(EmissionShape::Ring));
        assert_eq!(p.get_emission_shape(), EmissionShape::Ring);
        assert!(!p.set_emission_shape(EmissionShape::Max));

        p.set_emission_sphere_radius(2.5);
        p.set_emission_rect_extents(Vector2::new(3.0, 4.0));
        p.set_emission_ring_inner_radius(0.5);
        p.set_emission_ring_radius(1.5);
        assert_eq!(p.get_emission_sphere_radius(), 2.5);
        assert_eq!(p.get_emission_rect_extents(), Vector2::new(3.0, 4.0));
        assert_eq!(p.get_emission_ring_inner_radius(), 0.5);
        assert_eq!(p.get_emission_ring_radius(), 1.5);

        p.set_emission_points(vec![Vector2::ZERO, Vector2::ONE]);
        p.set_emission_normals(vec![Vector2::UP]);
        p.set_emission_colors(vec![Color::new(1.0, 0.0, 0.0, 1.0)]);
        assert_eq!(p.get_emission_points().len(), 2);
        assert_eq!(p.get_emission_normals(), &[Vector2::UP][..]);
        assert_eq!(p.get_emission_colors()[0], Color::new(1.0, 0.0, 0.0, 1.0));

        assert!(p.set_particle_flag(ParticleFlags::AlignYToVelocity, true));
        assert!(p.get_particle_flag(ParticleFlags::AlignYToVelocity));
        assert!(!p.set_particle_flag(ParticleFlags::Max, true));
        assert!(!p.get_particle_flag(ParticleFlags::Max));
    }

    #[test]
    fn curves_ramps_and_remaining_flags() {
        let mut p = CpuParticles2DData::new();
        let mut curve = Curve::new();
        curve.add_point(Vector2::new(0.0, 1.0));
        p.set_scale_curve_x(Some(curve.clone()));
        p.set_scale_curve_y(Some(curve));
        assert!(p.get_scale_curve_x().is_some());
        assert!(p.get_scale_curve_y().is_some());
        p.set_split_scale(true);
        assert!(p.is_scale_split());

        let mut ramp = Gradient::new();
        ramp.add_point(0.0, Color::new(1.0, 0.0, 0.0, 1.0));
        p.set_color_ramp(Some(ramp));
        assert!(p.get_color_ramp().is_some());
        assert!(p.get_color_initial_ramp().is_none());

        p.set_emitting(false);
        p.set_one_shot(true);
        p.set_pre_process_time(1.0);
        p.set_explosiveness_ratio(0.5);
        p.set_randomness_ratio(0.25);
        p.set_lifetime_randomness(0.75);
        p.set_speed_scale(2.0);
        p.set_use_local_coordinates(true);
        p.set_fixed_fps(60);
        p.set_fractional_delta(false);
        p.set_seed(9);
        p.set_use_fixed_seed(true);
        p.set_draw_order(DrawOrder::Lifetime);
        p.set_texture(Some(Rid::from_u64(3)));
        p.set_direction(Vector2::new(0.0, 1.0));
        p.set_spread(10.0);
        p.set_gravity(Vector2::new(0.0, 0.0));

        assert!(!p.is_emitting());
        assert!(p.get_one_shot());
        assert_eq!(p.get_pre_process_time(), 1.0);
        assert_eq!(p.get_explosiveness_ratio(), 0.5);
        assert_eq!(p.get_randomness_ratio(), 0.25);
        assert_eq!(p.get_lifetime_randomness(), 0.75);
        assert_eq!(p.get_speed_scale(), 2.0);
        assert!(p.is_using_local_coordinates());
        assert_eq!(p.get_fixed_fps(), 60);
        assert!(!p.get_fractional_delta());
        assert_eq!(p.get_seed(), 9);
        assert!(p.is_using_fixed_seed());
        assert_eq!(p.get_draw_order(), DrawOrder::Lifetime);
        assert_eq!(p.get_texture(), Some(Rid::from_u64(3)));
        assert_eq!(p.get_direction(), Vector2::new(0.0, 1.0));
        assert_eq!(p.get_spread(), 10.0);
        assert_eq!(p.get_gravity(), Vector2::ZERO);
    }

    #[test]
    fn enum_values() {
        assert_eq!(DrawOrder::Index as i32, 0);
        assert_eq!(DrawOrder::Lifetime as i32, 1);

        assert_eq!(Parameter::InitialLinearVelocity as i32, 0);
        assert_eq!(Parameter::AngularVelocity as i32, 1);
        assert_eq!(Parameter::OrbitVelocity as i32, 2);
        assert_eq!(Parameter::LinearAccel as i32, 3);
        assert_eq!(Parameter::RadialAccel as i32, 4);
        assert_eq!(Parameter::TangentialAccel as i32, 5);
        assert_eq!(Parameter::Damping as i32, 6);
        assert_eq!(Parameter::Angle as i32, 7);
        assert_eq!(Parameter::Scale as i32, 8);
        assert_eq!(Parameter::HueVariation as i32, 9);
        assert_eq!(Parameter::AnimSpeed as i32, 10);
        assert_eq!(Parameter::AnimOffset as i32, 11);
        assert_eq!(Parameter::Max as i32, 12);

        assert_eq!(ParticleFlags::AlignYToVelocity as i32, 0);
        assert_eq!(ParticleFlags::RotateY as i32, 1);
        assert_eq!(ParticleFlags::DisableZ as i32, 2);
        assert_eq!(ParticleFlags::Max as i32, 3);

        assert_eq!(EmissionShape::Point as i32, 0);
        assert_eq!(EmissionShape::Sphere as i32, 1);
        assert_eq!(EmissionShape::SphereSurface as i32, 2);
        assert_eq!(EmissionShape::Rectangle as i32, 3);
        assert_eq!(EmissionShape::Points as i32, 4);
        assert_eq!(EmissionShape::DirectedPoints as i32, 5);
        assert_eq!(EmissionShape::Ring as i32, 6);
        assert_eq!(EmissionShape::Max as i32, 7);
    }
}
