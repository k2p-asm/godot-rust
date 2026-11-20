//! `scene/2d/gpu_particles_2d.*`: particle emitter state (no rendering).
//!
//! The constructor's own sequence is reproduced: emitting on, one-shot off,
//! amount 8, ratio 1, lifetime 1, fixed FPS 0, fractional delta and interpolation
//! on, pre-process/explosiveness/randomness 0, visibility rect
//! `(-100,-100,200,200)`, world coordinates, lifetime draw order, speed scale 1.
//! Trails start disabled with lifetime 0.3, 8 sections and 4 subdivisions per
//! section, matching the header.
//!
//! Setter validation matches the C++: `amount >= 1`, `lifetime > 0`,
//! `trail_lifetime >= 0.01 - CMP_EPSILON`, `trail_sections` in `2..=128`,
//! `trail_section_subdivisions` in `1..=1024`. Everything else assigns directly
//! (including `amount_ratio`, which the property hint clamps but the setter does
//! not).
//!
//! Not ported: the RenderingServer particle RID, the mesh/material plumbing,
//! collision sizing and the seed RNG behaviour (the seed is stored as given).

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

/// `CMP_EPSILON`.
const CMP_EPSILON: f64 = 0.00001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawOrder {
    Index,
    Lifetime,
    ReverseLifetime,
}

#[derive(Clone, Debug)]
pub struct GpuParticles2DData {
    emitting: bool,
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
    seed: u32,
    use_fixed_seed: bool,
    draw_order: DrawOrder,
    process_material: Option<Rid>,
    texture: Option<Rid>,
    collision_base_size: f32,
    trail_enabled: bool,
    trail_lifetime: f64,
    trail_sections: i32,
    trail_section_subdivisions: i32,
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
            one_shot: false,
            amount: 8,
            amount_ratio: 1.0,
            lifetime: 1.0,
            pre_process_time: 0.0,
            explosiveness_ratio: 0.0,
            randomness_ratio: 0.0,
            speed_scale: 1.0,
            visibility_rect: Rect2::new(
                Vector2::new(-100.0, -100.0),
                Vector2::new(200.0, 200.0),
            ),
            local_coords: false,
            fixed_fps: 0,
            fractional_delta: true,
            interpolate: true,
            interp_to_end_factor: 0.0,
            seed: 0,
            use_fixed_seed: false,
            draw_order: DrawOrder::Lifetime,
            process_material: None,
            texture: None,
            collision_base_size: 1.0,
            trail_enabled: false,
            trail_lifetime: 0.3,
            trail_sections: 8,
            trail_section_subdivisions: 4,
        }
    }

    pub fn set_emitting(&mut self, emitting: bool) {
        self.emitting = emitting;
    }

    pub fn is_emitting(&self) -> bool {
        self.emitting
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn get_one_shot(&self) -> bool {
        self.one_shot
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

    /// No validation in the engine setter.
    pub fn set_amount_ratio(&mut self, ratio: f32) {
        self.amount_ratio = ratio;
    }

    pub fn get_amount_ratio(&self) -> f32 {
        self.amount_ratio
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

    pub fn set_interpolate(&mut self, enable: bool) {
        self.interpolate = enable;
    }

    pub fn get_interpolate(&self) -> bool {
        self.interpolate
    }

    /// `CLAMP(p_interp, 0.0, 1.0)`.
    pub fn set_interp_to_end_factor(&mut self, factor: f32) {
        self.interp_to_end_factor = factor.clamp(0.0, 1.0);
    }

    pub fn get_interp_to_end_factor(&self) -> f32 {
        self.interp_to_end_factor
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

    pub fn set_process_material(&mut self, material: Option<Rid>) {
        self.process_material = material;
    }

    pub fn get_process_material(&self) -> Option<Rid> {
        self.process_material
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
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

    /// `ERR_FAIL_COND(seconds < 0.01 - CMP_EPSILON)`.
    pub fn set_trail_lifetime(&mut self, seconds: f64) -> bool {
        if seconds < 0.01 - CMP_EPSILON {
            return false;
        }
        self.trail_lifetime = seconds;
        true
    }

    pub fn get_trail_lifetime(&self) -> f64 {
        self.trail_lifetime
    }

    /// `ERR_FAIL_COND(sections < 2)` and `> 128`.
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

    /// `ERR_FAIL_COND(subdivisions < 1)` and `> 1024`.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_defaults() {
        let p = GpuParticles2DData::new();
        assert!(p.is_emitting());
        assert!(!p.get_one_shot());
        assert_eq!(p.get_amount(), 8);
        assert_eq!(p.get_amount_ratio(), 1.0);
        assert_eq!(p.get_lifetime(), 1.0);
        assert_eq!(p.get_fixed_fps(), 0);
        assert!(p.get_fractional_delta());
        assert!(p.get_interpolate());
        assert_eq!(p.get_pre_process_time(), 0.0);
        assert_eq!(p.get_explosiveness_ratio(), 0.0);
        assert_eq!(p.get_randomness_ratio(), 0.0);
        assert_eq!(
            p.get_visibility_rect(),
            Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0))
        );
        assert!(!p.is_using_local_coordinates());
        assert_eq!(p.get_draw_order(), DrawOrder::Lifetime);
        assert_eq!(p.get_speed_scale(), 1.0);
        assert!(!p.is_trail_enabled());
        assert_eq!(p.get_trail_lifetime(), 0.3);
        assert_eq!(p.get_trail_sections(), 8);
        assert_eq!(p.get_trail_section_subdivisions(), 4);
        assert_eq!(p.get_collision_base_size(), 1.0);
        assert!(!p.is_using_fixed_seed());
        assert_eq!(p.get_seed(), 0);
    }

    #[test]
    fn amount_and_lifetime_validation() {
        let mut p = GpuParticles2DData::new();
        assert!(!p.set_amount(0));
        assert!(!p.set_amount(-5));
        assert_eq!(p.get_amount(), 8);
        assert!(p.set_amount(1));
        assert_eq!(p.get_amount(), 1);

        assert!(!p.set_lifetime(0.0));
        assert!(!p.set_lifetime(-1.0));
        assert_eq!(p.get_lifetime(), 1.0);
        assert!(p.set_lifetime(0.25));
        assert_eq!(p.get_lifetime(), 0.25);

        // amount_ratio is stored unclamped.
        p.set_amount_ratio(-3.0);
        assert_eq!(p.get_amount_ratio(), -3.0);
    }

    #[test]
    fn trail_validation_and_bounds() {
        let mut p = GpuParticles2DData::new();
        // The lower bound is 0.01 - CMP_EPSILON, so 0.00999 is accepted.
        assert!(p.set_trail_lifetime(0.00999));
        assert!(p.set_trail_lifetime(0.2));
        assert_eq!(p.get_trail_lifetime(), 0.2);

        assert!(!p.set_trail_sections(1));
        assert!(!p.set_trail_sections(129));
        assert_eq!(p.get_trail_sections(), 8);
        assert!(p.set_trail_sections(2));
        assert!(p.set_trail_sections(128));
        assert_eq!(p.get_trail_sections(), 128);

        assert!(!p.set_trail_section_subdivisions(0));
        assert!(!p.set_trail_section_subdivisions(1025));
        assert_eq!(p.get_trail_section_subdivisions(), 4);
        assert!(p.set_trail_section_subdivisions(1));
        assert!(p.set_trail_section_subdivisions(1024));

        p.set_trail_enabled(true);
        assert!(p.is_trail_enabled());
    }

    #[test]
    fn remaining_flags_and_enums() {
        let mut p = GpuParticles2DData::new();
        p.set_emitting(false);
        p.set_one_shot(true);
        p.set_pre_process_time(2.5);
        p.set_explosiveness_ratio(0.75);
        p.set_randomness_ratio(0.25);
        p.set_speed_scale(2.0);
        p.set_use_local_coordinates(true);
        p.set_fixed_fps(30);
        p.set_fractional_delta(false);
        p.set_interpolate(false);
        p.set_interp_to_end_factor(0.5);
        assert_eq!(p.get_interp_to_end_factor(), 0.5);
        // The setter clamps rather than rejecting.
        p.set_interp_to_end_factor(2.0);
        assert_eq!(p.get_interp_to_end_factor(), 1.0);
        p.set_interp_to_end_factor(-2.0);
        assert_eq!(p.get_interp_to_end_factor(), 0.0);
        p.set_interp_to_end_factor(0.5);
        p.set_seed(0xDEAD_BEEF);
        p.set_use_fixed_seed(true);
        p.set_draw_order(DrawOrder::ReverseLifetime);
        p.set_process_material(Some(Rid::from_u64(4)));
        p.set_texture(Some(Rid::from_u64(5)));
        p.set_collision_base_size(0.5);
        p.set_visibility_rect(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));

        assert!(!p.is_emitting());
        assert!(p.get_one_shot());
        assert_eq!(p.get_pre_process_time(), 2.5);
        assert_eq!(p.get_explosiveness_ratio(), 0.75);
        assert_eq!(p.get_randomness_ratio(), 0.25);
        assert_eq!(p.get_speed_scale(), 2.0);
        assert!(p.is_using_local_coordinates());
        assert_eq!(p.get_fixed_fps(), 30);
        assert!(!p.get_fractional_delta());
        assert!(!p.get_interpolate());
        assert_eq!(p.get_interp_to_end_factor(), 0.5);
        assert_eq!(p.get_seed(), 0xDEAD_BEEF);
        assert!(p.is_using_fixed_seed());
        assert_eq!(p.get_draw_order(), DrawOrder::ReverseLifetime);
        assert_eq!(p.get_process_material(), Some(Rid::from_u64(4)));
        assert_eq!(p.get_texture(), Some(Rid::from_u64(5)));
        assert_eq!(p.get_collision_base_size(), 0.5);
        assert_eq!(p.get_visibility_rect().size, Vector2::new(3.0, 4.0));

        assert_eq!(DrawOrder::Index as i32, 0);
        assert_eq!(DrawOrder::Lifetime as i32, 1);
        assert_eq!(DrawOrder::ReverseLifetime as i32, 2);
    }
}
