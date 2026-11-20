//! `scene/2d/parallax_layer.*` and the height-field collider from
//! `scene/3d/gpu_particles_collision_3d.*` (state only).
//!
//! `ParallaxLayer` defaults to motion scale `(1,1)`, zero motion offset and zero
//! mirroring.
//!
//! `GPUParticlesCollisionHeightField3D` defaults to a `(1 << 20) - 1` height-field
//! mask, size `(2,2,2)`, `RESOLUTION_1024`, follow-camera off and
//! `UPDATE_MODE_WHEN_MOVED`.
//!
//! The parallax base offset/scale computation and the height-field bake/render
//! calls are not ported.

use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;

// -- ParallaxLayer ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ParallaxLayerData {
    motion_scale: Vector2,
    motion_offset: Vector2,
    mirroring: Vector2,
}

impl Default for ParallaxLayerData {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallaxLayerData {
    pub fn new() -> Self {
        Self {
            motion_scale: Vector2::new(1.0, 1.0),
            motion_offset: Vector2::ZERO,
            mirroring: Vector2::ZERO,
        }
    }

    pub fn set_motion_offset(&mut self, offset: Vector2) {
        self.motion_offset = offset;
    }

    pub fn get_motion_offset(&self) -> Vector2 {
        self.motion_offset
    }

    pub fn set_motion_scale(&mut self, scale: Vector2) {
        self.motion_scale = scale;
    }

    pub fn get_motion_scale(&self) -> Vector2 {
        self.motion_scale
    }

    pub fn set_mirroring(&mut self, mirroring: Vector2) {
        self.mirroring = mirroring;
    }

    pub fn get_mirroring(&self) -> Vector2 {
        self.mirroring
    }
}

// -- GPUParticlesCollisionHeightField3D -------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HeightFieldResolution {
    Resolution256,
    Resolution512,
    Resolution1024,
    Resolution2048,
    Resolution4096,
    Resolution8192,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HeightFieldUpdateMode {
    WhenMoved,
    Always,
}

#[derive(Clone, Debug)]
pub struct GpuParticlesCollisionHeightField3DData {
    heightfield_mask: u32,
    size: Vector3,
    resolution: HeightFieldResolution,
    follow_camera_mode: bool,
    update_mode: HeightFieldUpdateMode,
}

impl Default for GpuParticlesCollisionHeightField3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuParticlesCollisionHeightField3DData {
    pub fn new() -> Self {
        Self {
            // Only the first 20 bits are set by default (editor layers ignored).
            heightfield_mask: (1 << 20) - 1,
            size: Vector3::new(2.0, 2.0, 2.0),
            resolution: HeightFieldResolution::Resolution1024,
            follow_camera_mode: false,
            update_mode: HeightFieldUpdateMode::WhenMoved,
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    /// `ERR_FAIL_INDEX(resolution, RESOLUTION_MAX)`.
    pub fn set_resolution(&mut self, resolution: i32) -> bool {
        let parsed = match resolution {
            0 => HeightFieldResolution::Resolution256,
            1 => HeightFieldResolution::Resolution512,
            2 => HeightFieldResolution::Resolution1024,
            3 => HeightFieldResolution::Resolution2048,
            4 => HeightFieldResolution::Resolution4096,
            5 => HeightFieldResolution::Resolution8192,
            _ => return false,
        };
        self.resolution = parsed;
        true
    }

    pub fn get_resolution(&self) -> HeightFieldResolution {
        self.resolution
    }

    pub fn set_update_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => HeightFieldUpdateMode::WhenMoved,
            1 => HeightFieldUpdateMode::Always,
            _ => return false,
        };
        self.update_mode = parsed;
        true
    }

    pub fn get_update_mode(&self) -> HeightFieldUpdateMode {
        self.update_mode
    }

    pub fn set_heightfield_mask(&mut self, mask: u32) {
        self.heightfield_mask = mask;
    }

    pub fn get_heightfield_mask(&self) -> u32 {
        self.heightfield_mask
    }

    pub fn set_heightfield_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.heightfield_mask |= bit;
        } else {
            self.heightfield_mask &= !bit;
        }
        true
    }

    pub fn get_heightfield_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        self.heightfield_mask & (1u32 << (layer - 1)) != 0
    }

    pub fn set_follow_camera_enabled(&mut self, enabled: bool) {
        self.follow_camera_mode = enabled;
    }

    pub fn is_follow_camera_enabled(&self) -> bool {
        self.follow_camera_mode
    }

    /// `get_aabb` for the collider: centred on the origin.
    pub fn get_aabb(&self) -> crate::math::aabb::Aabb {
        crate::math::aabb::Aabb::new(self.size * -0.5, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallax_layer_defaults_and_setters() {
        let mut layer = ParallaxLayerData::new();
        assert_eq!(layer.get_motion_scale(), Vector2::new(1.0, 1.0));
        assert_eq!(layer.get_motion_offset(), Vector2::ZERO);
        assert_eq!(layer.get_mirroring(), Vector2::ZERO);

        layer.set_motion_scale(Vector2::new(0.5, 0.25));
        layer.set_motion_offset(Vector2::new(10.0, -5.0));
        layer.set_mirroring(Vector2::new(64.0, 32.0));
        assert_eq!(layer.get_motion_scale(), Vector2::new(0.5, 0.25));
        assert_eq!(layer.get_motion_offset(), Vector2::new(10.0, -5.0));
        assert_eq!(layer.get_mirroring(), Vector2::new(64.0, 32.0));
    }

    #[test]
    fn heightfield_defaults_and_guards() {
        let mut field = GpuParticlesCollisionHeightField3DData::new();
        assert_eq!(field.get_heightfield_mask(), (1 << 20) - 1);
        assert_eq!(field.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(field.get_resolution(), HeightFieldResolution::Resolution1024);
        assert!(!field.is_follow_camera_enabled());
        assert_eq!(field.get_update_mode(), HeightFieldUpdateMode::WhenMoved);
        assert_eq!(
            field.get_aabb(),
            crate::math::aabb::Aabb::new(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(2.0, 2.0, 2.0)
            )
        );

        assert!(field.set_resolution(5));
        assert_eq!(field.get_resolution(), HeightFieldResolution::Resolution8192);
        assert!(!field.set_resolution(6));
        assert!(!field.set_resolution(-1));
        assert!(field.set_update_mode(1));
        assert_eq!(field.get_update_mode(), HeightFieldUpdateMode::Always);
        assert!(!field.set_update_mode(2));

        field.set_size(Vector3::new(4.0, 2.0, 8.0));
        field.set_follow_camera_enabled(true);
        assert_eq!(field.get_aabb().size, Vector3::new(4.0, 2.0, 8.0));
        assert!(field.is_follow_camera_enabled());

        field.set_heightfield_mask(0);
        assert!(field.set_heightfield_mask_value(1, true));
        assert!(field.set_heightfield_mask_value(32, true));
        assert_eq!(field.get_heightfield_mask(), 1 | (1 << 31));
        assert!(field.get_heightfield_mask_value(32));
        assert!(!field.set_heightfield_mask_value(0, true));
        assert!(!field.set_heightfield_mask_value(33, true));
        assert!(!field.get_heightfield_mask_value(33));

        assert_eq!(HeightFieldResolution::Resolution256 as i32, 0);
        assert_eq!(HeightFieldResolution::Max as i32, 6);
        assert_eq!(HeightFieldUpdateMode::Always as i32, 1);
    }
}
