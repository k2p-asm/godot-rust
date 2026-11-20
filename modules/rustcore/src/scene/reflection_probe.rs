//! `scene/3d/reflection_probe.*`: probe parameters.
//!
//! Defaults: intensity 1, blend distance 1, max distance 0, size `(20,20,20)`,
//! origin offset 0, box projection/shadows/interior off, ambient mode
//! environment, black ambient colour with energy 1, mesh LOD threshold 1, cull
//! and reflection masks `(1 << 20) - 1`, update once.
//!
//! `set_size` and `set_origin_offset` clamp the offset so it stays within
//! `half_size - 0.01` on each axis (with `half_size` floored at 0.01 in
//! `set_size`), mirroring the engine. RIDs are not ported.

use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::scene::visual_instance_3d::VisualInstance3DData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum UpdateMode {
    Once,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AmbientMode {
    Disabled,
    Environment,
    Color,
}

#[derive(Clone, Debug)]
pub struct ReflectionProbeData {
    pub visual: VisualInstance3DData,
    intensity: f32,
    blend_distance: f32,
    max_distance: f32,
    size: Vector3,
    origin_offset: Vector3,
    box_projection: bool,
    enable_shadows: bool,
    interior: bool,
    ambient_mode: AmbientMode,
    ambient_color: Color,
    ambient_color_energy: f32,
    mesh_lod_threshold: f32,
    cull_mask: u32,
    reflection_mask: u32,
    update_mode: UpdateMode,
}

impl Default for ReflectionProbeData {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionProbeData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            intensity: 1.0,
            blend_distance: 1.0,
            max_distance: 0.0,
            size: Vector3::new(20.0, 20.0, 20.0),
            origin_offset: Vector3::ZERO,
            box_projection: false,
            enable_shadows: false,
            interior: false,
            ambient_mode: AmbientMode::Environment,
            ambient_color: Color::rgb(0.0, 0.0, 0.0),
            ambient_color_energy: 1.0,
            mesh_lod_threshold: 1.0,
            cull_mask: (1 << 20) - 1,
            reflection_mask: (1 << 20) - 1,
            update_mode: UpdateMode::Once,
        }
    }

    fn clamp_axis(offset: f32, limit: f32) -> f32 {
        if limit < offset.abs() {
            let sign = if offset > 0.0 {
                1.0
            } else if offset < 0.0 {
                -1.0
            } else {
                0.0
            };
            sign * limit
        } else {
            offset
        }
    }

    /// The stored size is unclamped, but the origin offset is pulled inside
    /// `max(size/2, 0.01) - 0.01` on each axis.
    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
        let axes = [size.x, size.y, size.z];
        let mut offset = [self.origin_offset.x, self.origin_offset.y, self.origin_offset.z];
        for i in 0..3 {
            let half_size = (axes[i] / 2.0).max(0.01);
            offset[i] = Self::clamp_axis(offset[i], half_size - 0.01);
        }
        self.origin_offset = Vector3::new(offset[0], offset[1], offset[2]);
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    /// Clamps the offset against the raw `size/2 - 0.01` limit.
    pub fn set_origin_offset(&mut self, offset: Vector3) {
        let half = [
            self.size.x / 2.0,
            self.size.y / 2.0,
            self.size.z / 2.0,
        ];
        let mut values = [offset.x, offset.y, offset.z];
        for i in 0..3 {
            values[i] = Self::clamp_axis(values[i], half[i] - 0.01);
        }
        self.origin_offset = Vector3::new(values[0], values[1], values[2]);
    }

    pub fn get_origin_offset(&self) -> Vector3 {
        self.origin_offset
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    pub fn set_blend_distance(&mut self, distance: f32) {
        self.blend_distance = distance;
    }

    pub fn get_blend_distance(&self) -> f32 {
        self.blend_distance
    }

    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance;
    }

    pub fn get_max_distance(&self) -> f32 {
        self.max_distance
    }

    pub fn set_enable_box_projection(&mut self, enable: bool) {
        self.box_projection = enable;
    }

    pub fn is_box_projection_enabled(&self) -> bool {
        self.box_projection
    }

    pub fn set_enable_shadows(&mut self, enable: bool) {
        self.enable_shadows = enable;
    }

    pub fn are_shadows_enabled(&self) -> bool {
        self.enable_shadows
    }

    pub fn set_as_interior(&mut self, enable: bool) {
        self.interior = enable;
    }

    pub fn is_set_as_interior(&self) -> bool {
        self.interior
    }

    pub fn set_ambient_mode(&mut self, mode: AmbientMode) {
        self.ambient_mode = mode;
    }

    pub fn get_ambient_mode(&self) -> AmbientMode {
        self.ambient_mode
    }

    pub fn set_ambient_color(&mut self, color: Color) {
        self.ambient_color = color;
    }

    pub fn get_ambient_color(&self) -> Color {
        self.ambient_color
    }

    pub fn set_ambient_color_energy(&mut self, energy: f32) {
        self.ambient_color_energy = energy;
    }

    pub fn get_ambient_color_energy(&self) -> f32 {
        self.ambient_color_energy
    }

    pub fn set_mesh_lod_threshold(&mut self, threshold: f32) {
        self.mesh_lod_threshold = threshold;
    }

    pub fn get_mesh_lod_threshold(&self) -> f32 {
        self.mesh_lod_threshold
    }

    pub fn set_cull_mask(&mut self, mask: u32) {
        self.cull_mask = mask;
    }

    pub fn get_cull_mask(&self) -> u32 {
        self.cull_mask
    }

    pub fn set_reflection_mask(&mut self, mask: u32) {
        self.reflection_mask = mask;
    }

    pub fn get_reflection_mask(&self) -> u32 {
        self.reflection_mask
    }

    pub fn set_update_mode(&mut self, mode: UpdateMode) {
        self.update_mode = mode;
    }

    pub fn get_update_mode(&self) -> UpdateMode {
        self.update_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = ReflectionProbeData::new();
        assert_eq!(p.get_intensity(), 1.0);
        assert_eq!(p.get_blend_distance(), 1.0);
        assert_eq!(p.get_max_distance(), 0.0);
        assert_eq!(p.get_size(), Vector3::new(20.0, 20.0, 20.0));
        assert_eq!(p.get_origin_offset(), Vector3::ZERO);
        assert!(!p.is_box_projection_enabled());
        assert!(!p.are_shadows_enabled());
        assert!(!p.is_set_as_interior());
        assert_eq!(p.get_ambient_mode(), AmbientMode::Environment);
        assert_eq!(p.get_ambient_color(), Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(p.get_ambient_color_energy(), 1.0);
        assert_eq!(p.get_mesh_lod_threshold(), 1.0);
        assert_eq!(p.get_cull_mask(), (1 << 20) - 1);
        assert_eq!(p.get_reflection_mask(), (1 << 20) - 1);
        assert_eq!(p.get_update_mode(), UpdateMode::Once);
    }

    #[test]
    fn origin_offset_is_clamped_by_size() {
        let mut p = ReflectionProbeData::new();
        // size 2 -> half 1, limit 0.99.
        p.set_size(Vector3::new(2.0, 2.0, 2.0));
        p.set_origin_offset(Vector3::new(5.0, -5.0, 0.5));
        let o = p.get_origin_offset();
        assert!((o.x - 0.99).abs() < 1e-5, "{o:?}");
        assert!((o.y + 0.99).abs() < 1e-5, "{o:?}");
        assert!((o.z - 0.5).abs() < 1e-5, "{o:?}");

        // Growing the size keeps the offset; shrinking clamps it again.
        p.set_size(Vector3::new(100.0, 100.0, 100.0));
        assert!((p.get_origin_offset().x - 0.99).abs() < 1e-5);
        p.set_size(Vector3::new(1.0, 1.0, 1.0));
        let o = p.get_origin_offset();
        assert!((o.x - 0.49).abs() < 1e-5, "{o:?}");
        assert!((o.y + 0.49).abs() < 1e-5, "{o:?}");

        // Tiny sizes floor the half size at 0.01, so the limit is zero.
        p.set_size(Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(p.get_origin_offset(), Vector3::ZERO);
        assert_eq!(p.get_size(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn other_parameters_and_enums() {
        let mut p = ReflectionProbeData::new();
        p.set_intensity(2.0);
        p.set_blend_distance(0.5);
        p.set_max_distance(30.0);
        p.set_enable_box_projection(true);
        p.set_enable_shadows(true);
        p.set_as_interior(true);
        p.set_ambient_mode(AmbientMode::Color);
        p.set_ambient_color(Color::new(0.1, 0.2, 0.3, 1.0));
        p.set_ambient_color_energy(2.0);
        p.set_mesh_lod_threshold(4.0);
        p.set_cull_mask(3);
        p.set_reflection_mask(5);
        p.set_update_mode(UpdateMode::Always);

        assert_eq!(p.get_intensity(), 2.0);
        assert_eq!(p.get_blend_distance(), 0.5);
        assert_eq!(p.get_max_distance(), 30.0);
        assert!(p.is_box_projection_enabled());
        assert!(p.are_shadows_enabled());
        assert!(p.is_set_as_interior());
        assert_eq!(p.get_ambient_mode(), AmbientMode::Color);
        assert_eq!(p.get_ambient_color(), Color::new(0.1, 0.2, 0.3, 1.0));
        assert_eq!(p.get_ambient_color_energy(), 2.0);
        assert_eq!(p.get_mesh_lod_threshold(), 4.0);
        assert_eq!(p.get_cull_mask(), 3);
        assert_eq!(p.get_reflection_mask(), 5);
        assert_eq!(p.get_update_mode(), UpdateMode::Always);

        assert_eq!(UpdateMode::Once as i32, 0);
        assert_eq!(UpdateMode::Always as i32, 1);
        assert_eq!(AmbientMode::Disabled as i32, 0);
        assert_eq!(AmbientMode::Environment as i32, 1);
        assert_eq!(AmbientMode::Color as i32, 2);
    }
}
