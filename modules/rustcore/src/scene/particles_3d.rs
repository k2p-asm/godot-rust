//! `scene/3d/gpu_particles_3d.*`: 3D particle emitter state (no rendering).
//!
//! The constructor's own sequence is reproduced: emitting on, one-shot off,
//! amount 8, ratio 1, lifetime 1, fixed FPS **30** (unlike the 2D emitter's 0),
//! fractional delta and interpolation on, pre-process/explosiveness/randomness
//! 0, trail lifetime 0.3, visibility AABB `(-4,-4,-4) + (8,8,8)`, world
//! coordinates, one draw pass, index draw order, speed 1, collision base size
//! 0.01, transform alignment disabled (channel X, axis Y) and a random seed.
//!
//! Validation matches the C++: `amount >= 1`, `lifetime > 0`,
//! `trail_lifetime >= 0.01 - CMP_EPSILON`, `draw_passes >= 1`, and
//! `interp_to_end` clamped to `0..=1`. Everything else assigns directly.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

/// `CMP_EPSILON`.
const CMP_EPSILON: f64 = 0.00001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawOrder {
    Index,
    Lifetime,
    ReverseLifetime,
    ViewDepth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformAlign {
    Disabled,
    ZBillboard,
    YToVelocity,
    ZBillboardYToVelocity,
    LocalBillboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlignChannelFilter {
    Disabled,
    X,
    Y,
    Z,
    W,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlignAxis {
    X,
    Y,
}

#[derive(Clone, Debug)]
pub struct GpuParticles3DData {
    emitting: bool,
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
    interp_to_end_factor: f32,
    sub_emitter: String,
    collision_base_size: f32,
    seed: u32,
    use_fixed_seed: bool,
    trail_enabled: bool,
    trail_lifetime: f64,
    transform_align: TransformAlign,
    align_channel_filter: AlignChannelFilter,
    align_axis: AlignAxis,
    transform_align_use_velocity: bool,
    process_material: Option<Rid>,
    draw_order: DrawOrder,
    draw_passes: Vec<Option<Rid>>,
    skin: Option<Rid>,
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
            one_shot: false,
            amount: 8,
            amount_ratio: 1.0,
            lifetime: 1.0,
            pre_process_time: 0.0,
            explosiveness_ratio: 0.0,
            randomness_ratio: 0.0,
            speed_scale: 1.0,
            visibility_aabb: Aabb::new(Vector3::new(-4.0, -4.0, -4.0), Vector3::new(8.0, 8.0, 8.0)),
            local_coords: false,
            fixed_fps: 30,
            fractional_delta: true,
            interpolate: true,
            interp_to_end_factor: 0.0,
            sub_emitter: String::new(),
            collision_base_size: 0.01,
            seed: 0,
            use_fixed_seed: false,
            trail_enabled: false,
            trail_lifetime: 0.3,
            transform_align: TransformAlign::Disabled,
            align_channel_filter: AlignChannelFilter::X,
            align_axis: AlignAxis::Y,
            transform_align_use_velocity: true,
            process_material: None,
            draw_order: DrawOrder::Index,
            draw_passes: vec![None],
            skin: None,
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

    pub fn set_visibility_aabb(&mut self, aabb: Aabb) {
        self.visibility_aabb = aabb;
    }

    pub fn get_visibility_aabb(&self) -> Aabb {
        self.visibility_aabb
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
    pub fn set_interp_to_end(&mut self, interp: f32) {
        self.interp_to_end_factor = interp.clamp(0.0, 1.0);
    }

    pub fn get_interp_to_end(&self) -> f32 {
        self.interp_to_end_factor
    }

    pub fn set_sub_emitter(&mut self, path: &str) {
        self.sub_emitter = path.to_string();
    }

    pub fn get_sub_emitter(&self) -> &str {
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

    pub fn set_use_fixed_seed(&mut self, use_fixed: bool) {
        self.use_fixed_seed = use_fixed;
    }

    pub fn is_using_fixed_seed(&self) -> bool {
        self.use_fixed_seed
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

    pub fn set_transform_align(&mut self, align: TransformAlign) {
        self.transform_align = align;
    }

    pub fn get_transform_align(&self) -> TransformAlign {
        self.transform_align
    }

    pub fn set_transform_align_channel_filter(&mut self, filter: AlignChannelFilter) {
        self.align_channel_filter = filter;
    }

    pub fn get_transform_align_channel_filter(&self) -> AlignChannelFilter {
        self.align_channel_filter
    }

    pub fn set_transform_align_axis(&mut self, axis: AlignAxis) {
        self.align_axis = axis;
    }

    pub fn get_transform_align_axis(&self) -> AlignAxis {
        self.align_axis
    }

    pub fn set_transform_align_use_velocity(&mut self, use_velocity: bool) {
        self.transform_align_use_velocity = use_velocity;
    }

    pub fn is_transform_align_using_velocity(&self) -> bool {
        self.transform_align_use_velocity
    }

    pub fn set_process_material(&mut self, material: Option<Rid>) {
        self.process_material = material;
    }

    pub fn get_process_material(&self) -> Option<Rid> {
        self.process_material
    }

    pub fn set_draw_order(&mut self, order: DrawOrder) {
        self.draw_order = order;
    }

    pub fn get_draw_order(&self) -> DrawOrder {
        self.draw_order
    }

    /// `ERR_FAIL_COND(p_count < 1)`; shrinking clears the dropped meshes.
    pub fn set_draw_passes(&mut self, count: i32) -> bool {
        if count < 1 {
            return false;
        }
        self.draw_passes.resize(count as usize, None);
        true
    }

    pub fn get_draw_passes(&self) -> i32 {
        self.draw_passes.len() as i32
    }

    pub fn set_draw_pass_mesh(&mut self, pass: i32, mesh: Option<Rid>) -> bool {
        if pass < 0 || pass as usize >= self.draw_passes.len() {
            return false;
        }
        self.draw_passes[pass as usize] = mesh;
        true
    }

    pub fn get_draw_pass_mesh(&self, pass: i32) -> Option<Rid> {
        if pass < 0 {
            return None;
        }
        self.draw_passes.get(pass as usize).copied().flatten()
    }

    pub fn set_skin(&mut self, skin: Option<Rid>) {
        self.skin = skin;
    }

    pub fn get_skin(&self) -> Option<Rid> {
        self.skin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_defaults() {
        let p = GpuParticles3DData::new();
        assert!(p.is_emitting());
        assert!(!p.get_one_shot());
        assert_eq!(p.get_amount(), 8);
        assert_eq!(p.get_amount_ratio(), 1.0);
        assert_eq!(p.get_lifetime(), 1.0);
        // The 3D emitter starts at 30 fixed FPS, unlike the 2D one.
        assert_eq!(p.get_fixed_fps(), 30);
        assert!(p.get_fractional_delta());
        assert!(p.get_interpolate());
        assert_eq!(
            p.get_visibility_aabb(),
            Aabb::new(Vector3::new(-4.0, -4.0, -4.0), Vector3::new(8.0, 8.0, 8.0))
        );
        assert!(!p.is_using_local_coordinates());
        assert_eq!(p.get_draw_passes(), 1);
        assert!(p.get_draw_pass_mesh(0).is_none());
        assert_eq!(p.get_draw_order(), DrawOrder::Index);
        assert_eq!(p.get_speed_scale(), 1.0);
        assert_eq!(p.get_collision_base_size(), 0.01);
        assert_eq!(p.get_transform_align(), TransformAlign::Disabled);
        assert_eq!(p.get_transform_align_channel_filter(), AlignChannelFilter::X);
        assert_eq!(p.get_transform_align_axis(), AlignAxis::Y);
        assert!(p.is_transform_align_using_velocity());
        assert!(!p.is_trail_enabled());
        assert_eq!(p.get_trail_lifetime(), 0.3);
        assert_eq!(p.get_sub_emitter(), "");
        assert!(!p.is_using_fixed_seed());
        assert_eq!(p.get_seed(), 0);
    }

    #[test]
    fn bounds_and_clamps() {
        let mut p = GpuParticles3DData::new();
        assert!(!p.set_amount(0));
        assert!(p.set_amount(64));
        assert_eq!(p.get_amount(), 64);

        assert!(!p.set_lifetime(0.0));
        assert!(p.set_lifetime(0.5));

        assert!(!p.set_trail_lifetime(0.0));
        assert!(p.set_trail_lifetime(0.00999));
        assert!(p.set_trail_lifetime(2.0));

        // interp_to_end clamps instead of rejecting.
        p.set_interp_to_end(-1.0);
        assert_eq!(p.get_interp_to_end(), 0.0);
        p.set_interp_to_end(3.0);
        assert_eq!(p.get_interp_to_end(), 1.0);
        p.set_interp_to_end(0.25);
        assert_eq!(p.get_interp_to_end(), 0.25);

        // Draw passes reject zero and resize.
        assert!(!p.set_draw_passes(0));
        assert_eq!(p.get_draw_passes(), 1);
        assert!(p.set_draw_passes(4));
        assert_eq!(p.get_draw_passes(), 4);
    }

    #[test]
    fn draw_pass_meshes_are_indexed() {
        let mut p = GpuParticles3DData::new();
        assert!(p.set_draw_passes(2));
        assert!(p.set_draw_pass_mesh(0, Some(Rid::from_u64(3))));
        assert!(p.set_draw_pass_mesh(1, Some(Rid::from_u64(4))));
        assert_eq!(p.get_draw_pass_mesh(0), Some(Rid::from_u64(3)));
        assert_eq!(p.get_draw_pass_mesh(1), Some(Rid::from_u64(4)));
        assert!(!p.set_draw_pass_mesh(2, None));
        assert!(!p.set_draw_pass_mesh(-1, None));
        assert!(p.get_draw_pass_mesh(9).is_none());

        // Shrinking drops the trailing meshes.
        assert!(p.set_draw_passes(1));
        assert_eq!(p.get_draw_pass_mesh(0), Some(Rid::from_u64(3)));
        assert!(p.get_draw_pass_mesh(1).is_none());
    }

    #[test]
    fn enums_and_remaining_flags() {
        assert_eq!(DrawOrder::Index as i32, 0);
        assert_eq!(DrawOrder::Lifetime as i32, 1);
        assert_eq!(DrawOrder::ReverseLifetime as i32, 2);
        assert_eq!(DrawOrder::ViewDepth as i32, 3);

        assert_eq!(TransformAlign::Disabled as i32, 0);
        assert_eq!(TransformAlign::ZBillboard as i32, 1);
        assert_eq!(TransformAlign::YToVelocity as i32, 2);
        assert_eq!(TransformAlign::ZBillboardYToVelocity as i32, 3);
        assert_eq!(TransformAlign::LocalBillboard as i32, 4);

        assert_eq!(AlignChannelFilter::Disabled as i32, 0);
        assert_eq!(AlignChannelFilter::X as i32, 1);
        assert_eq!(AlignChannelFilter::W as i32, 4);
        assert_eq!(AlignAxis::X as i32, 0);
        assert_eq!(AlignAxis::Y as i32, 1);

        let mut p = GpuParticles3DData::new();
        p.set_emitting(false);
        p.set_one_shot(true);
        p.set_pre_process_time(1.5);
        p.set_explosiveness_ratio(0.9);
        p.set_randomness_ratio(0.1);
        p.set_speed_scale(-2.0);
        p.set_use_local_coordinates(true);
        p.set_fractional_delta(false);
        p.set_interpolate(false);
        p.set_sub_emitter("../Other");
        p.set_collision_base_size(0.5);
        p.set_seed(7);
        p.set_use_fixed_seed(true);
        p.set_trail_enabled(true);
        p.set_transform_align(TransformAlign::LocalBillboard);
        p.set_transform_align_channel_filter(AlignChannelFilter::Z);
        p.set_transform_align_axis(AlignAxis::X);
        p.set_transform_align_use_velocity(false);
        p.set_process_material(Some(Rid::from_u64(8)));
        p.set_draw_order(DrawOrder::ViewDepth);
        p.set_skin(Some(Rid::from_u64(9)));

        assert!(!p.is_emitting());
        assert!(p.get_one_shot());
        assert_eq!(p.get_pre_process_time(), 1.5);
        assert_eq!(p.get_explosiveness_ratio(), 0.9);
        assert_eq!(p.get_randomness_ratio(), 0.1);
        assert_eq!(p.get_speed_scale(), -2.0);
        assert!(p.is_using_local_coordinates());
        assert!(!p.get_fractional_delta());
        assert!(!p.get_interpolate());
        assert_eq!(p.get_sub_emitter(), "../Other");
        assert_eq!(p.get_collision_base_size(), 0.5);
        assert_eq!(p.get_seed(), 7);
        assert!(p.is_using_fixed_seed());
        assert!(p.is_trail_enabled());
        assert_eq!(p.get_transform_align(), TransformAlign::LocalBillboard);
        assert_eq!(p.get_transform_align_channel_filter(), AlignChannelFilter::Z);
        assert_eq!(p.get_transform_align_axis(), AlignAxis::X);
        assert!(!p.is_transform_align_using_velocity());
        assert_eq!(p.get_process_material(), Some(Rid::from_u64(8)));
        assert_eq!(p.get_draw_order(), DrawOrder::ViewDepth);
        assert_eq!(p.get_skin(), Some(Rid::from_u64(9)));
    }
}
