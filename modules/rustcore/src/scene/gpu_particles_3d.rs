//! `scene/3d/gpu_particles_3d.*` (state): the 3D GPU particle emitter.
//!
//! Ported configuration with the constructor defaults: emitting, one-shot,
//! amount 8, amount ratio 1, lifetime 1 s, pre-process 0, explosiveness and
//! randomness 0, speed scale 1, visibility AABB (-4,-4,-4, 8³), local
//! coordinates off, fixed FPS 30, fractional delta and interpolation on, the
//! seed/fixed-seed flag, process material, draw passes (1, max 4), draw order
//! (`INDEX`), transform alignment (`DISABLED`, velocity-driven, X channel
//! filter, Y axis), sub-emitter path, collision base size 0.01 and the trail
//! settings (off, 0.3 s, 8 sections, 4 subdivisions).
//!
//! The particles RID, draw-pass meshes and the server calls are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::string::node_path::NodePath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawOrder {
    Index = 0,
    Lifetime = 1,
    ReverseLifetime = 2,
    ViewDepth = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformAlign {
    Disabled = 0,
    ZBillboard = 1,
    YToVelocity = 2,
    ZBillboardYToVelocity = 3,
    LocalBillboard = 4,
}

pub const MAX_DRAW_PASSES: usize = 4;

pub const EMIT_FLAG_POSITION: u32 = 1;
pub const EMIT_FLAG_ROTATION_SCALE: u32 = 2;
pub const EMIT_FLAG_VELOCITY: u32 = 4;
pub const EMIT_FLAG_COLOR: u32 = 8;
pub const EMIT_FLAG_CUSTOM: u32 = 16;

#[derive(Clone, Debug)]
pub struct GpuParticles3DData {
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
    visibility_aabb: Aabb,
    local_coords: bool,
    fixed_fps: i32,
    fractional_delta: bool,
    interpolate: bool,
    sub_emitter: NodePath,
    collision_base_size: f32,
    seed: u32,
    use_fixed_seed: bool,
    process_material: Option<Rid>,
    draw_passes: i32,
    draw_order: DrawOrder,
    transform_align: TransformAlign,
    transform_align_use_velocity: bool,
    transform_align_channel_filter: i32,
    transform_align_axis: i32,
    trail_enabled: bool,
    trail_lifetime: f64,
    trail_sections: i32,
    trail_section_subdivisions: i32,
    time: f64,
    emission_time: f64,
    active_time: f64,
    interp_to_end_factor: f32,
    previous_velocity: Vector3,
    previous_position: Vector3,
}

impl Default for GpuParticles3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticles3DData {
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
            visibility_aabb: Aabb::new(
                Vector3::new(-4.0, -4.0, -4.0),
                Vector3::new(8.0, 8.0, 8.0),
            ),
            local_coords: false,
            fixed_fps: 30,
            fractional_delta: true,
            interpolate: true,
            sub_emitter: NodePath::new(),
            collision_base_size: 0.01,
            seed: 0,
            use_fixed_seed: false,
            process_material: None,
            draw_passes: 1,
            draw_order: DrawOrder::Index,
            transform_align: TransformAlign::Disabled,
            transform_align_use_velocity: true,
            transform_align_channel_filter: 0,
            transform_align_axis: 1,
            trail_enabled: false,
            trail_lifetime: 0.3,
            trail_sections: 8,
            trail_section_subdivisions: 4,
            time: 0.0,
            emission_time: 0.0,
            active_time: 0.0,
            interp_to_end_factor: 0.0,
            previous_velocity: Vector3::ZERO,
            previous_position: Vector3::ZERO,
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

    pub fn set_visibility_aabb(&mut self, aabb: Aabb) {
        self.visibility_aabb = aabb;
    }

    pub fn get_visibility_aabb(&self) -> Aabb {
        self.visibility_aabb
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

    /// `set_draw_passes`: 1..=`MAX_DRAW_PASSES`.
    pub fn set_draw_passes(&mut self, passes: i32) -> bool {
        if !(1..=MAX_DRAW_PASSES as i32).contains(&passes) {
            return false;
        }
        self.draw_passes = passes;
        true
    }

    pub fn get_draw_passes(&self) -> i32 {
        self.draw_passes
    }

    pub fn set_draw_order(&mut self, order: i32) -> bool {
        let parsed = match order {
            0 => DrawOrder::Index,
            1 => DrawOrder::Lifetime,
            2 => DrawOrder::ReverseLifetime,
            3 => DrawOrder::ViewDepth,
            _ => return false,
        };
        self.draw_order = parsed;
        true
    }

    pub fn get_draw_order(&self) -> DrawOrder {
        self.draw_order
    }

    pub fn set_transform_align(&mut self, align: i32) -> bool {
        let parsed = match align {
            0 => TransformAlign::Disabled,
            1 => TransformAlign::ZBillboard,
            2 => TransformAlign::YToVelocity,
            3 => TransformAlign::ZBillboardYToVelocity,
            4 => TransformAlign::LocalBillboard,
            _ => return false,
        };
        self.transform_align = parsed;
        true
    }

    pub fn get_transform_align(&self) -> TransformAlign {
        self.transform_align
    }

    pub fn set_transform_align_use_velocity(&mut self, enabled: bool) {
        self.transform_align_use_velocity = enabled;
    }

    pub fn is_transform_align_using_velocity(&self) -> bool {
        self.transform_align_use_velocity
    }

    pub fn set_transform_align_channel_filter(&mut self, filter: i32) {
        self.transform_align_channel_filter = filter;
    }

    pub fn get_transform_align_channel_filter(&self) -> i32 {
        self.transform_align_channel_filter
    }

    pub fn set_transform_align_axis(&mut self, axis: i32) {
        self.transform_align_axis = axis;
    }

    pub fn get_transform_align_axis(&self) -> i32 {
        self.transform_align_axis
    }

    pub fn set_trail_enabled(&mut self, enabled: bool) {
        self.trail_enabled = enabled;
    }

    pub fn is_trail_enabled(&self) -> bool {
        self.trail_enabled
    }

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

    /// `restart`.
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

    pub fn set_interp_to_end_factor(&mut self, factor: f32) {
        self.interp_to_end_factor = factor;
    }

    pub fn get_interp_to_end_factor(&self) -> f32 {
        self.interp_to_end_factor
    }

    pub fn get_previous_velocity(&self) -> Vector3 {
        self.previous_velocity
    }

    pub fn get_previous_position(&self) -> Vector3 {
        self.previous_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let particles = GpuParticles3DData::new();
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
            particles.get_visibility_aabb(),
            Aabb::new(Vector3::new(-4.0, -4.0, -4.0), Vector3::new(8.0, 8.0, 8.0))
        );
        assert!(!particles.get_use_local_coordinates());
        assert_eq!(particles.get_fixed_fps(), 30);
        assert!(particles.get_fractional_delta());
        assert!(particles.get_interpolate());
        assert!(particles.get_sub_emitter().is_empty());
        assert_eq!(particles.get_collision_base_size(), 0.01);
        assert_eq!(particles.get_seed(), 0);
        assert!(!particles.get_use_fixed_seed());
        assert!(particles.get_process_material().is_none());
        assert_eq!(particles.get_draw_passes(), 1);
        assert_eq!(particles.get_draw_order(), DrawOrder::Index);
        assert_eq!(particles.get_transform_align(), TransformAlign::Disabled);
        assert!(particles.is_transform_align_using_velocity());
        assert_eq!(particles.get_transform_align_channel_filter(), 0);
        assert_eq!(particles.get_transform_align_axis(), 1);
        assert!(!particles.is_trail_enabled());
        assert_eq!(particles.get_trail_lifetime(), 0.3);
        assert_eq!(particles.get_trail_sections(), 8);
        assert_eq!(particles.get_trail_section_subdivisions(), 4);
        assert_eq!(MAX_DRAW_PASSES, 4);
        assert_eq!(DrawOrder::ViewDepth as i32, 3);
        assert_eq!(TransformAlign::LocalBillboard as i32, 4);
    }

    #[test]
    fn guards_and_setters() {
        let mut particles = GpuParticles3DData::new();
        assert!(!particles.set_amount(0));
        assert!(particles.set_amount(16));
        assert!(!particles.set_lifetime(0.0));
        assert!(particles.set_lifetime(0.75));
        assert!(!particles.set_draw_passes(0));
        assert!(!particles.set_draw_passes(5));
        assert!(particles.set_draw_passes(4));
        assert!(!particles.set_draw_order(4));
        assert!(particles.set_draw_order(3));
        assert!(!particles.set_transform_align(5));
        assert!(particles.set_transform_align(3));
        assert!(!particles.set_trail_lifetime(0.0));
        assert!(particles.set_trail_lifetime(1.0));
        assert!(!particles.set_trail_sections(1));
        assert!(particles.set_trail_sections(32));
        assert!(!particles.set_trail_section_subdivisions(0));
        assert!(particles.set_trail_section_subdivisions(16));

        particles.set_sub_emitter(NodePath::parse("Emitter").expect("path"));
        particles.set_collision_base_size(0.5);
        particles.set_process_material(Some(Rid::from_u64(8)));
        particles.set_transform_align_use_velocity(false);
        particles.set_transform_align_channel_filter(2);
        particles.set_transform_align_axis(0);
        particles.set_interp_to_end_factor(0.25);

        assert_eq!(particles.get_amount(), 16);
        assert_eq!(particles.get_lifetime(), 0.75);
        assert_eq!(particles.get_draw_passes(), 4);
        assert_eq!(particles.get_draw_order(), DrawOrder::ViewDepth);
        assert_eq!(
            particles.get_transform_align(),
            TransformAlign::ZBillboardYToVelocity
        );
        assert!(!particles.is_transform_align_using_velocity());
        assert_eq!(particles.get_transform_align_channel_filter(), 2);
        assert_eq!(particles.get_transform_align_axis(), 0);
        assert_eq!(particles.get_sub_emitter().as_string(), "Emitter");
        assert_eq!(particles.get_collision_base_size(), 0.5);
        assert_eq!(particles.get_process_material(), Some(Rid::from_u64(8)));
        assert_eq!(particles.get_interp_to_end_factor(), 0.25);
    }

    #[test]
    fn restart() {
        let mut particles = GpuParticles3DData::new();
        particles.set_seed(7);
        particles.restart(false);
        assert_eq!(particles.get_seed(), 8);
        particles.restart(true);
        assert_eq!(particles.get_seed(), 8);
        assert_eq!(particles.get_time(), 0.0);
        assert_eq!(particles.get_emission_time(), 0.0);
        assert_eq!(particles.get_active_time(), 0.0);
        assert_eq!(EMIT_FLAG_ROTATION_SCALE, 2);
        assert_eq!(EMIT_FLAG_COLOR, 8);
    }
}
