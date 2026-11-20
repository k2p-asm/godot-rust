//! `scene/3d/camera_3d.*`: camera parameters (no viewport/RenderingServer).
//!
//! Defaults: perspective, FOV 75, size 1, frustum offset zero, near 0.05, far
//! 4000, no h/v offset, `KeepHeight`, cull mask `0xfffff`, Doppler tracking off.
//! Validation matches the C++: FOV `1..=179`, size `> CMP_EPSILON`, cull-mask
//! layer numbers `1..=20`, and only the three known projection modes are
//! accepted.
//!
//! The camera RID, projection matrix (needs the viewport size), environment and
//! attributes are not ported beyond stored handles.

use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

/// `CMP_EPSILON`.
const CMP_EPSILON: f32 = 0.00001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ProjectionType {
    Perspective,
    Orthogonal,
    Frustum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum KeepAspect {
    KeepWidth,
    KeepHeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DopplerTracking {
    Disabled,
    IdleStep,
    PhysicsStep,
}

#[derive(Clone, Debug)]
pub struct Camera3DData {
    mode: ProjectionType,
    fov: f32,
    size: f32,
    frustum_offset: Vector2,
    near: f32,
    far: f32,
    v_offset: f32,
    h_offset: f32,
    keep_aspect: KeepAspect,
    layers: u32,
    environment: Option<Rid>,
    attributes: Option<Rid>,
    compositor: Option<Rid>,
    doppler_tracking: DopplerTracking,
    current: bool,
}

impl Default for Camera3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera3DData {
    pub fn new() -> Self {
        Self {
            mode: ProjectionType::Perspective,
            fov: 75.0,
            size: 1.0,
            frustum_offset: Vector2::ZERO,
            near: 0.05,
            far: 4000.0,
            v_offset: 0.0,
            h_offset: 0.0,
            keep_aspect: KeepAspect::KeepHeight,
            layers: 0xfffff,
            environment: None,
            attributes: None,
            compositor: None,
            doppler_tracking: DopplerTracking::Disabled,
            current: false,
        }
    }

    /// Only the three known modes are accepted (the engine ignores others).
    pub fn set_projection(&mut self, mode: ProjectionType) -> bool {
        match mode {
            ProjectionType::Perspective | ProjectionType::Orthogonal | ProjectionType::Frustum => {
                self.mode = mode;
                true
            }
        }
    }

    pub fn get_projection(&self) -> ProjectionType {
        self.mode
    }

    pub fn set_fov(&mut self, fov: f32) -> bool {
        if !(1.0..=179.0).contains(&fov) {
            return false;
        }
        self.fov = fov;
        true
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn set_size(&mut self, size: f32) -> bool {
        if size <= CMP_EPSILON {
            return false;
        }
        self.size = size;
        true
    }

    pub fn get_size(&self) -> f32 {
        self.size
    }

    pub fn set_frustum_offset(&mut self, offset: Vector2) {
        self.frustum_offset = offset;
    }

    pub fn get_frustum_offset(&self) -> Vector2 {
        self.frustum_offset
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn set_h_offset(&mut self, offset: f32) {
        self.h_offset = offset;
    }

    pub fn get_h_offset(&self) -> f32 {
        self.h_offset
    }

    pub fn set_v_offset(&mut self, offset: f32) {
        self.v_offset = offset;
    }

    pub fn get_v_offset(&self) -> f32 {
        self.v_offset
    }

    pub fn set_keep_aspect_mode(&mut self, keep_aspect: KeepAspect) {
        self.keep_aspect = keep_aspect;
    }

    pub fn get_keep_aspect_mode(&self) -> KeepAspect {
        self.keep_aspect
    }

    pub fn set_cull_mask(&mut self, mask: u32) {
        self.layers = mask;
    }

    pub fn get_cull_mask(&self) -> u32 {
        self.layers
    }

    /// Layer numbers are `1..=20`; other values are rejected.
    pub fn set_cull_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=20).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.layers |= bit;
        } else {
            self.layers &= !bit;
        }
        true
    }

    pub fn get_cull_mask_value(&self, layer: i32) -> Option<bool> {
        if !(1..=20).contains(&layer) {
            return None;
        }
        Some((self.layers & (1u32 << (layer - 1))) != 0)
    }

    pub fn set_environment(&mut self, environment: Option<Rid>) {
        self.environment = environment;
    }

    pub fn get_environment(&self) -> Option<Rid> {
        self.environment
    }

    pub fn set_attributes(&mut self, attributes: Option<Rid>) {
        self.attributes = attributes;
    }

    pub fn get_attributes(&self) -> Option<Rid> {
        self.attributes
    }

    pub fn set_compositor(&mut self, compositor: Option<Rid>) {
        self.compositor = compositor;
    }

    pub fn get_compositor(&self) -> Option<Rid> {
        self.compositor
    }

    pub fn set_doppler_tracking(&mut self, tracking: DopplerTracking) {
        self.doppler_tracking = tracking;
    }

    pub fn get_doppler_tracking(&self) -> DopplerTracking {
        self.doppler_tracking
    }

    pub fn make_current(&mut self) {
        self.current = true;
    }

    pub fn clear_current(&mut self) {
        self.current = false;
    }

    pub fn is_current(&self) -> bool {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = Camera3DData::new();
        assert_eq!(c.get_projection(), ProjectionType::Perspective);
        assert_eq!(c.get_fov(), 75.0);
        assert_eq!(c.get_size(), 1.0);
        assert_eq!(c.get_frustum_offset(), Vector2::ZERO);
        assert_eq!(c.get_near(), 0.05);
        assert_eq!(c.get_far(), 4000.0);
        assert_eq!(c.get_h_offset(), 0.0);
        assert_eq!(c.get_v_offset(), 0.0);
        assert_eq!(c.get_keep_aspect_mode(), KeepAspect::KeepHeight);
        assert_eq!(c.get_cull_mask(), 0xfffff);
        assert!(c.get_environment().is_none());
        assert!(c.get_attributes().is_none());
        assert!(c.get_compositor().is_none());
        assert_eq!(c.get_doppler_tracking(), DopplerTracking::Disabled);
        assert!(!c.is_current());
    }

    #[test]
    fn fov_and_size_validation() {
        let mut c = Camera3DData::new();
        assert!(!c.set_fov(0.999));
        assert!(!c.set_fov(179.001));
        assert_eq!(c.get_fov(), 75.0);
        // Both ends are accepted.
        assert!(c.set_fov(1.0));
        assert!(c.set_fov(179.0));
        assert_eq!(c.get_fov(), 179.0);

        assert!(!c.set_size(0.0));
        assert!(!c.set_size(CMP_EPSILON));
        assert!(c.set_size(2.0));
        assert_eq!(c.get_size(), 2.0);

        c.set_near(0.1);
        c.set_far(100.0);
        c.set_frustum_offset(Vector2::new(0.5, -0.5));
        c.set_h_offset(0.25);
        c.set_v_offset(-0.25);
        assert_eq!(c.get_near(), 0.1);
        assert_eq!(c.get_far(), 100.0);
        assert_eq!(c.get_frustum_offset(), Vector2::new(0.5, -0.5));
        assert_eq!(c.get_h_offset(), 0.25);
        assert_eq!(c.get_v_offset(), -0.25);
    }

    #[test]
    fn cull_mask_layers() {
        let mut c = Camera3DData::new();
        c.set_cull_mask(0);
        assert_eq!(c.get_cull_mask(), 0);

        assert!(c.set_cull_mask_value(1, true));
        assert_eq!(c.get_cull_mask(), 1);
        assert!(c.set_cull_mask_value(20, true));
        assert_eq!(c.get_cull_mask(), 1 | (1 << 19));
        assert_eq!(c.get_cull_mask_value(1), Some(true));
        assert_eq!(c.get_cull_mask_value(2), Some(false));

        assert!(c.set_cull_mask_value(20, false));
        assert_eq!(c.get_cull_mask(), 1);
        assert!(!c.set_cull_mask_value(0, true));
        assert!(!c.set_cull_mask_value(21, true));
        assert_eq!(c.get_cull_mask_value(21), None);
    }

    #[test]
    fn projection_enums_and_switches() {
        let mut c = Camera3DData::new();
        assert!(c.set_projection(ProjectionType::Orthogonal));
        assert_eq!(c.get_projection(), ProjectionType::Orthogonal);
        assert!(c.set_projection(ProjectionType::Frustum));
        assert_eq!(c.get_projection(), ProjectionType::Frustum);
        assert!(c.set_projection(ProjectionType::Perspective));

        c.set_keep_aspect_mode(KeepAspect::KeepWidth);
        assert_eq!(c.get_keep_aspect_mode(), KeepAspect::KeepWidth);
        c.set_doppler_tracking(DopplerTracking::PhysicsStep);
        assert_eq!(c.get_doppler_tracking(), DopplerTracking::PhysicsStep);

        c.set_environment(Some(Rid::from_u64(1)));
        c.set_attributes(Some(Rid::from_u64(2)));
        c.set_compositor(Some(Rid::from_u64(3)));
        assert_eq!(c.get_environment(), Some(Rid::from_u64(1)));
        assert_eq!(c.get_attributes(), Some(Rid::from_u64(2)));
        assert_eq!(c.get_compositor(), Some(Rid::from_u64(3)));

        c.make_current();
        assert!(c.is_current());
        c.clear_current();
        assert!(!c.is_current());

        assert_eq!(ProjectionType::Perspective as i32, 0);
        assert_eq!(ProjectionType::Orthogonal as i32, 1);
        assert_eq!(ProjectionType::Frustum as i32, 2);
        assert_eq!(KeepAspect::KeepWidth as i32, 0);
        assert_eq!(KeepAspect::KeepHeight as i32, 1);
        assert_eq!(DopplerTracking::Disabled as i32, 0);
        assert_eq!(DopplerTracking::IdleStep as i32, 1);
        assert_eq!(DopplerTracking::PhysicsStep as i32, 2);
    }
}
