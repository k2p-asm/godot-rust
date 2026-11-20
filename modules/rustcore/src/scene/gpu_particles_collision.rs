//! `scene/3d/gpu_particles_collision_3d.*`: particle collider node state.
//!
//! `GPUParticlesCollision3D` adds the cull mask (`0xFFFFFFFF`) and the collision
//! RID to `Node3D`. `GPUParticlesCollisionSphere3D` starts with radius 1 and
//! `GPUParticlesCollisionBox3D` with size `(2,2,2)`.
//!
//! `GPUParticlesCollisionHeightField3D` and `GPUParticlesCollisionSDF3D` have
//! larger parameter sets plus baking and are marked pending.
//!
//! The RenderingServer collider calls, baking and debug drawing are not ported.

use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

/// `GPUParticlesCollision3D`'s shared state.
#[derive(Clone, Debug)]
pub struct GpuParticlesCollision3DData {
    cull_mask: u32,
    collision: Option<Rid>,
}

impl Default for GpuParticlesCollision3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticlesCollision3DData {
    pub fn new() -> Self {
        Self {
            cull_mask: 0xFFFFFFFF,
            collision: None,
        }
    }

    pub fn set_cull_mask(&mut self, mask: u32) {
        self.cull_mask = mask;
    }

    pub fn get_cull_mask(&self) -> u32 {
        self.cull_mask
    }

    pub fn set_collision(&mut self, collision: Option<Rid>) {
        self.collision = collision;
    }

    pub fn get_collision(&self) -> Option<Rid> {
        self.collision
    }
}

#[derive(Clone, Debug)]
pub struct GpuParticlesCollisionSphere3DData {
    pub base: GpuParticlesCollision3DData,
    radius: f32,
}

impl Default for GpuParticlesCollisionSphere3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticlesCollisionSphere3DData {
    pub fn new() -> Self {
        Self {
            base: GpuParticlesCollision3DData::new(),
            radius: 1.0,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }
}

#[derive(Clone, Debug)]
pub struct GpuParticlesCollisionBox3DData {
    pub base: GpuParticlesCollision3DData,
    size: Vector3,
}

impl Default for GpuParticlesCollisionBox3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticlesCollisionBox3DData {
    pub fn new() -> Self {
        Self {
            base: GpuParticlesCollision3DData::new(),
            size: Vector3::new(2.0, 2.0, 2.0),
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }
}

/// `GPUParticlesCollisionSDF3D`'s parameters, kept without baking.
#[derive(Clone, Debug)]
pub struct GpuParticlesCollisionSdf3DData {
    pub base: GpuParticlesCollision3DData,
    size: Vector3,
    thickness: f32,
    cell_size: f32,
    field_texture: Option<Rid>,
    bake_mask: u32,
    texture_size: i32,
    bake_range: Vector3,
}

impl Default for GpuParticlesCollisionSdf3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticlesCollisionSdf3DData {
    pub fn new() -> Self {
        Self {
            base: GpuParticlesCollision3DData::new(),
            size: Vector3::new(2.0, 2.0, 2.0),
            thickness: 0.5,
            cell_size: 0.5,
            field_texture: None,
            bake_mask: 0xFFFFFFFF,
            texture_size: 64,
            bake_range: Vector3::ZERO,
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }

    pub fn get_thickness(&self) -> f32 {
        self.thickness
    }

    /// `ERR_FAIL_COND(p_value >= 0.01)`.
    pub fn set_cell_size(&mut self, value: f32) -> bool {
        if value < 0.01 {
            return false;
        }
        self.cell_size = value;
        true
    }

    pub fn get_cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn set_field_texture(&mut self, texture: Option<Rid>) {
        self.field_texture = texture;
    }

    pub fn get_field_texture(&self) -> Option<Rid> {
        self.field_texture
    }

    pub fn set_bake_mask(&mut self, mask: u32) {
        self.bake_mask = mask;
    }

    pub fn get_bake_mask(&self) -> u32 {
        self.bake_mask
    }

    pub fn set_bake_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.bake_mask |= bit;
        } else {
            self.bake_mask &= !bit;
        }
        true
    }

    pub fn get_bake_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        (self.bake_mask & (1u32 << (layer - 1))) != 0
    }

    pub fn set_texture_size(&mut self, size: i32) {
        self.texture_size = size;
    }

    pub fn get_texture_size(&self) -> i32 {
        self.texture_size
    }

    /// The baked range (the engine computes it during `bake`).
    pub fn get_bake_range(&self) -> Vector3 {
        self.bake_range
    }

    pub fn set_bake_range(&mut self, range: Vector3) {
        self.bake_range = range;
    }

    /// Present for parity with the engine's debug preview; baking is not ported.
    pub fn get_debug_color(&self) -> Color {
        Color::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_and_sphere() {
        let mut s = GpuParticlesCollisionSphere3DData::new();
        assert_eq!(s.base.get_cull_mask(), 0xFFFFFFFF);
        assert_eq!(s.get_radius(), 1.0);
        assert!(s.base.get_collision().is_none());

        s.base.set_cull_mask(3);
        s.base.set_collision(Some(Rid::from_u64(2)));
        s.set_radius(2.5);
        assert_eq!(s.base.get_cull_mask(), 3);
        assert_eq!(s.base.get_collision(), Some(Rid::from_u64(2)));
        assert_eq!(s.get_radius(), 2.5);
    }

    #[test]
    fn box_collider() {
        let mut b = GpuParticlesCollisionBox3DData::new();
        assert_eq!(b.get_size(), Vector3::new(2.0, 2.0, 2.0));
        b.set_size(Vector3::new(4.0, 1.0, 2.0));
        assert_eq!(b.get_size(), Vector3::new(4.0, 1.0, 2.0));
        assert_eq!(b.base.get_cull_mask(), 0xFFFFFFFF);
    }

    #[test]
    fn sdf_parameters() {
        let mut s = GpuParticlesCollisionSdf3DData::new();
        assert_eq!(s.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(s.get_thickness(), 0.5);
        assert_eq!(s.get_cell_size(), 0.5);
        assert!(s.get_field_texture().is_none());
        assert_eq!(s.get_bake_mask(), 0xFFFFFFFF);
        assert_eq!(s.get_texture_size(), 64);
        assert_eq!(s.get_bake_range(), Vector3::ZERO);

        // The cell size must be at least 0.01.
        assert!(!s.set_cell_size(0.009));
        assert!(s.set_cell_size(0.01));
        assert_eq!(s.get_cell_size(), 0.01);

        s.set_thickness(1.0);
        s.set_texture_size(128);
        s.set_field_texture(Some(Rid::from_u64(3)));
        assert_eq!(s.get_thickness(), 1.0);
        assert_eq!(s.get_texture_size(), 128);
        assert_eq!(s.get_field_texture(), Some(Rid::from_u64(3)));

        s.set_bake_mask(0);
        assert!(s.set_bake_mask_value(1, true));
        assert!(s.set_bake_mask_value(32, true));
        assert_eq!(s.get_bake_mask(), 1 | (1 << 31));
        assert!(s.get_bake_mask_value(32));
        assert!(!s.set_bake_mask_value(0, true));
        assert!(!s.set_bake_mask_value(33, true));
        assert!(!s.get_bake_mask_value(33));

        s.set_bake_range(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(s.get_bake_range(), Vector3::new(1.0, 2.0, 3.0));
    }
}
