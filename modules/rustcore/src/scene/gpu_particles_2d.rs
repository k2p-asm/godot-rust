//! `scene/2d/gpu_particles_2d.*` (state): the 2D GPU particle emitter.
//!
//! Ported configuration with the constructor defaults: emitting, one-shot,
//! amount 8, amount ratio 1, lifetime 1 s, pre-process 0, explosiveness and
//! randomness 0, speed scale 1, visibility rect (-100,-100,200,200), local
//! coordinates off, fixed FPS 30, fractional delta and interpolation on, the
//! random seed/fixed-seed flag, process material, draw order (lifetime),
//! texture handle, sub-emitter path, collision base size, and the trail settings
//! (off, 0.3 s, 8 sections, 4 subdivisions) plus the engine's guards.
//!
//! The particles RID, the particle mesh and the rendering-server calls are not
//! ported; the runtime counters (`time`, `emission_time`, `active_time`) are
//! stored but only advanced by the (unported) process loop.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::string::node_path::NodePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawOrder {
    Index = 0,
    Lifetime = 1,
    ReverseLifetime = 2,
}

/// `PARTICLES_EMIT_FLAG_*` bit values.
pub const EMIT_FLAG_POSITION: u32 = 1;
pub const EMIT_FLAG_ROTATION_SCALE: u32 = 2;
pub const EMIT_FLAG_VELOCITY: u32 = 4;
pub const EMIT_FLAG_COLOR: u32 = 8;
pub const EMIT_FLAG_CUSTOM: u32 = 16;

#[derive(Clone, Debug)]
pub struct GpuParticles2DData {
    emitting: bool,
    active: bool,
    signal_canceled: bool,
    one_shot: bool,
    amount: i32,
    amount_ratio: f32,
    lifetime: f64,
    pre_process_time: f64,
    explosiveness_ratio: f32,
    randomness_ratio: f32,
    speed_scale: f64,
    visibility_rect: Rect2,
    local_coords: bool,
    fixed_fps: i32,
    fractional_delta: bool,
    interpolate: bool,
    interp_to_end_factor: f32,
    previous_velocity: Vector3,
    previous_position: Vector2,
    seed: u32,
    use_fixed_seed: bool,
    process_material: Option<Rid>,
    draw_order: DrawOrder,
    texture: Option<Rid>,
    sub_emitter: NodePath,
    collision_base_size: f32,
    trail_enabled: bool,
    trail_lifetime: f64,
    trail_sections: i32,
    trail_section_subdivisions: i32,
    time: f64,
    emission_time: f64,
    active_time: f64,
}

impl Default for GpuParticles2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticles2DData {
    pub fn new() -> Self {
        Self {
            emitting: true,
            active: false,
            signal_canceled: false,
            one_shot: false,
            amount: 8,
            amount_ratio: 1.0,
            lifetime: 1.0,
            pre_process_time: 0.0,
            explosiveness_ratio: 0.0,
            randomness_ratio: 0.0,
            speed_scale: 1.0,
            visibility_rect: Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0)),
            local_coords: false,
            // The constructor calls `set_fixed_fps(0)` and later `(30)`.
            fixed_fps: 30,
            fractional_delta: true,
            interpolate: true,
            interp_to_end_factor: 0.0,
            previous_velocity: Vector3::ZERO,
            previous_position: Vector2::ZERO,
            seed: 0,
            use_fixed_seed: false,
            process_material: None,
            draw_order: DrawOrder::Lifetime,
            texture: None,
            sub_emitter: NodePath::new(),
            collision_base_size: 1.0,
            trail_enabled: false,
            trail_lifetime: 0.3,
            trail_sections: 8,
            trail_section_subdivisions: 4,
            time: 0.0,
            emission_time: 0.0,
            active_time: 0.0,
        }
    }

    pub fn set_emitting(&mut self, emitting: bool) {
        self.emitting = emitting;
    }

    pub fn is_emitting(&self) -> bool {
        self.emitting
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn get_one_shot(&self) -> bool {
        self.one_shot
    }

    /// `set_amount`: values below 1 are refused.
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

    pub fn set_amount_ratio(&mut self, ratio: f32) {
        self.amount_ratio = ratio;
    }

    pub fn get_amount_ratio(&self) -> f32 {
        self.amount_ratio
    }

    /// `set_lifetime`: values at or below zero are refused.
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

    pub fn set_speed_scale(&mut self, scale: f64) {
        self.speed_scale = scale;
    }

    pub fn get_speed_scale(&self) -> f64 {
        self.speed_scale
    }

    pub fn set_visibility_rect(&mut self, rect: Rect2) {
        self.visibility_rect = rect;
    }

    pub fn get_visibility_rect(&self) -> Rect2 {
        self.visibility_rect
    }

    pub fn set_use_local_coordinates(&mut self, enabled: bool) {
        self.local_coords = enabled;
    }

    pub fn get_use_local_coordinates(&self) -> bool {
        self.local_coords
    }

    pub fn set_fixed_fps(&mut self, fps: i32) {
        self.fixed_fps = fps;
    }

    pub fn get_fixed_fps(&self) -> i32 {
        self.fixed_fps
    }

    pub fn set_fractional_delta(&mut self, enabled: bool) {
        self.fractional_delta = enabled;
    }

    pub fn get_fractional_delta(&self) -> bool {
        self.fractional_delta
    }

    pub fn set_interpolate(&mut self, enabled: bool) {
        self.interpolate = enabled;
    }

    pub fn get_interpolate(&self) -> bool {
        self.interpolate
    }

    pub fn set_interp_to_end_factor(&mut self, factor: f32) {
        self.interp_to_end_factor = factor;
    }

    pub fn get_interp_to_end_factor(&self) -> f32 {
        self.interp_to_end_factor
    }

    pub fn get_previous_velocity(&self) -> Vector3 {
        self.previous_velocity
    }

    pub fn get_previous_position(&self) -> Vector2 {
        self.previous_position
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    pub fn set_use_fixed_seed(&mut self, enabled: bool) {
        self.use_fixed_seed = enabled;
    }

    pub fn get_use_fixed_seed(&self) -> bool {
        self.use_fixed_seed
    }

    pub fn set_process_material(&mut self, material: Option<Rid>) {
        self.process_material = material;
    }

    pub fn get_process_material(&self) -> Option<Rid> {
        self.process_material
    }

    pub fn set_draw_order(&mut self, order: i32) -> bool {
        let parsed = match order {
            0 => DrawOrder::Index,
            1 => DrawOrder::Lifetime,
            2 => DrawOrder::ReverseLifetime,
            _ => return false,
        };
        self.draw_order = parsed;
        true
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

    pub fn set_sub_emitter(&mut self, path: NodePath) {
        self.sub_emitter = path;
    }

    pub fn get_sub_emitter(&self) -> &NodePath {
        &self.sub_emitter
    }

    pub fn set_collision_base_size(&mut self, size: f32) {
        self.collision_base_size = size;
    }

    pub fn get_collision_base_size(&self) -> f32 {
        self.collision_base_size
    }

    pub fn set_trail_enabled(&mut self, enabled: bool) {
        self.trail_enabled = enabled;
    }

    pub fn is_trail_enabled(&self) -> bool {
        self.trail_enabled
    }

    /// `set_trail_lifetime`: below 0.01 s is refused.
    pub fn set_trail_lifetime(&mut self, seconds: f64) -> bool {
        if seconds < 0.01 - 1e-6 {
            return false;
        }
        self.trail_lifetime = seconds;
        true
    }

    pub fn get_trail_lifetime(&self) -> f64 {
        self.trail_lifetime
    }

    /// `set_trail_sections`: 2..=128.
    pub fn set_trail_sections(&mut self, sections: i32) -> bool {
        if !(2..=128).contains(&sections) {
            return false;
        }
        self.trail_sections = sections;
        true
    }

    pub fn get_trail_sections(&self) -> i32 {
        self.trail_sections
    }

    /// `set_trail_section_subdivisions`: 1..=1024.
    pub fn set_trail_section_subdivisions(&mut self, subdivisions: i32) -> bool {
        if !(1..=1024).contains(&subdivisions) {
            return false;
        }
        self.trail_section_subdivisions = subdivisions;
        true
    }

    pub fn get_trail_section_subdivisions(&self) -> i32 {
        self.trail_section_subdivisions
    }

    /// `restart`: requests a one-shot restart.
    pub fn restart(&mut self, keep_seed: bool) {
        if !keep_seed {
            self.seed = self.seed.wrapping_add(1);
        }
        self.time = 0.0;
        self.emission_time = 0.0;
        self.active_time = 0.0;
        self.signal_canceled = true;
    }

    pub fn get_time(&self) -> f64 {
        self.time
    }

    pub fn get_emission_time(&self) -> f64 {
        self.emission_time
    }

    pub fn get_active_time(&self) -> f64 {
        self.active_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let particles = GpuParticles2DData::new();
        assert!(particles.is_emitting());
        assert!(!particles.is_active());
        assert!(!particles.get_one_shot());
        assert_eq!(particles.get_amount(), 8);
        assert_eq!(particles.get_amount_ratio(), 1.0);
        assert_eq!(particles.get_lifetime(), 1.0);
        assert_eq!(particles.get_pre_process_time(), 0.0);
        assert_eq!(particles.get_explosiveness_ratio(), 0.0);
        assert_eq!(particles.get_randomness_ratio(), 0.0);
        assert_eq!(particles.get_speed_scale(), 1.0);
        assert_eq!(
            particles.get_visibility_rect(),
            Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0))
        );
        assert!(!particles.get_use_local_coordinates());
        assert_eq!(particles.get_fixed_fps(), 30);
        assert!(particles.get_fractional_delta());
        assert!(particles.get_interpolate());
        assert_eq!(particles.get_interp_to_end_factor(), 0.0);
        assert_eq!(particles.get_seed(), 0);
        assert!(!particles.get_use_fixed_seed());
        assert!(particles.get_process_material().is_none());
        assert_eq!(particles.get_draw_order(), DrawOrder::Lifetime);
        assert!(particles.get_texture().is_none());
        assert!(particles.get_sub_emitter().is_empty());
        assert_eq!(particles.get_collision_base_size(), 1.0);
        assert!(!particles.is_trail_enabled());
        assert_eq!(particles.get_trail_lifetime(), 0.3);
        assert_eq!(particles.get_trail_sections(), 8);
        assert_eq!(particles.get_trail_section_subdivisions(), 4);
        assert_eq!(particles.get_time(), 0.0);
        assert_eq!(DrawOrder::Index as i32, 0);
        assert_eq!(DrawOrder::ReverseLifetime as i32, 2);
    }

    #[test]
    fn guards() {
        let mut particles = GpuParticles2DData::new();
        assert!(!particles.set_amount(0));
        assert_eq!(particles.get_amount(), 8);
        assert!(particles.set_amount(64));
        assert_eq!(particles.get_amount(), 64);

        assert!(!particles.set_lifetime(0.0));
        assert!(!particles.set_lifetime(-1.0));
        assert!(particles.set_lifetime(2.5));
        assert_eq!(particles.get_lifetime(), 2.5);

        assert!(!particles.set_trail_lifetime(0.0));
        assert!(particles.set_trail_lifetime(0.5));
        assert_eq!(particles.get_trail_lifetime(), 0.5);
        assert!(!particles.set_trail_sections(1));
        assert!(!particles.set_trail_sections(129));
        assert!(particles.set_trail_sections(16));
        assert_eq!(particles.get_trail_sections(), 16);
        assert!(!particles.set_trail_section_subdivisions(0));
        assert!(!particles.set_trail_section_subdivisions(1025));
        assert!(particles.set_trail_section_subdivisions(8));

        assert!(!particles.set_draw_order(3));
        assert!(particles.set_draw_order(0));
        assert_eq!(particles.get_draw_order(), DrawOrder::Index);
        // The emit flags keep the render-server bit values.
        assert_eq!(EMIT_FLAG_POSITION, 1);
        assert_eq!(EMIT_FLAG_VELOCITY, 4);
        assert_eq!(EMIT_FLAG_CUSTOM, 16);
    }

    #[test]
    fn setters_and_restart() {
        let mut particles = GpuParticles2DData::new();
        particles.set_emitting(false);
        particles.set_one_shot(true);
        particles.set_amount_ratio(0.5);
        particles.set_pre_process_time(1.5);
        particles.set_explosiveness_ratio(0.75);
        particles.set_randomness_ratio(0.25);
        particles.set_speed_scale(2.0);
        particles.set_visibility_rect(Rect2::new(Vector2::ZERO, Vector2::new(10.0, 20.0)));
        particles.set_use_local_coordinates(true);
        particles.set_fixed_fps(60);
        particles.set_fractional_delta(false);
        particles.set_interpolate(false);
        particles.set_interp_to_end_factor(0.5);
        particles.set_seed(1234);
        particles.set_use_fixed_seed(true);
        particles.set_process_material(Some(Rid::from_u64(3)));
        particles.set_texture(Some(Rid::from_u64(4)));
        particles.set_sub_emitter(NodePath::parse("SubEmitter").expect("path"));
        particles.set_collision_base_size(2.0);
        particles.set_trail_enabled(true);

        assert!(!particles.is_emitting());
        assert!(particles.get_one_shot());
        assert_eq!(particles.get_amount_ratio(), 0.5);
        assert_eq!(particles.get_pre_process_time(), 1.5);
        assert_eq!(particles.get_explosiveness_ratio(), 0.75);
        assert_eq!(particles.get_randomness_ratio(), 0.25);
        assert_eq!(particles.get_speed_scale(), 2.0);
        assert_eq!(
            particles.get_visibility_rect(),
            Rect2::new(Vector2::ZERO, Vector2::new(10.0, 20.0))
        );
        assert!(particles.get_use_local_coordinates());
        assert_eq!(particles.get_fixed_fps(), 60);
        assert!(!particles.get_fractional_delta());
        assert!(!particles.get_interpolate());
        assert_eq!(particles.get_interp_to_end_factor(), 0.5);
        assert_eq!(particles.get_seed(), 1234);
        assert!(particles.get_use_fixed_seed());
        assert_eq!(particles.get_process_material(), Some(Rid::from_u64(3)));
        assert_eq!(particles.get_texture(), Some(Rid::from_u64(4)));
        assert_eq!(particles.get_sub_emitter().as_string(), "SubEmitter");
        assert_eq!(particles.get_collision_base_size(), 2.0);
        assert!(particles.is_trail_enabled());

        // A restart with a new seed advances the seed and resets the clocks.
        particles.restart(false);
        assert_eq!(particles.get_seed(), 1235);
        assert_eq!(particles.get_time(), 0.0);
        particles.restart(true);
        assert_eq!(particles.get_seed(), 1235, "keep_seed leaves the seed");
    }
}
