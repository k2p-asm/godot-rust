//! `scene/resources/camera_attributes.*`: base, practical and physical attributes.
//!
//! Defaults and constructor overrides are copied from the C++:
//!
//! - base: exposure multiplier 1, sensitivity 100 ISO, auto exposure off with
//!   range `0.01..64`, speed 0.5 and scale 0.4; normalization is 1.
//! - practical: sensitivity/3072007 normalization (the physical default at ISO
//!   100), DOF defaults and auto-exposure range `0..800`.
//! - physical: aperture 16, shutter 100, focal 35 mm, focus 10 m, near 0.05,
//!   far 4000; the constructor computes the FOV from the 24 mm sensor height
//!   and sets the auto-exposure range to `-8..10`. Normalization is
//!   `1 / (aperture² · shutter · (100 / sensitivity) · 1.2)`.
//!
//! Not ported: the RenderingServer RID, DOF parameter upload and
//! `use_physical_light_units` gating.

use crate::math::funcs;

/// Base `CameraAttributes`.
#[derive(Clone, Copy, Debug)]
pub struct CameraAttributesData {
    exposure_multiplier: f32,
    exposure_sensitivity: f32,
    auto_exposure_enabled: bool,
    auto_exposure_min: f32,
    auto_exposure_max: f32,
    auto_exposure_speed: f32,
    auto_exposure_scale: f32,
}

impl Default for CameraAttributesData {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraAttributesData {
    pub fn new() -> Self {
        Self {
            exposure_multiplier: 1.0,
            exposure_sensitivity: 100.0,
            auto_exposure_enabled: false,
            auto_exposure_min: 0.01,
            auto_exposure_max: 64.0,
            auto_exposure_speed: 0.5,
            auto_exposure_scale: 0.4,
        }
    }

    pub fn set_exposure_multiplier(&mut self, multiplier: f32) {
        self.exposure_multiplier = multiplier;
    }

    pub fn get_exposure_multiplier(&self) -> f32 {
        self.exposure_multiplier
    }

    pub fn set_exposure_sensitivity(&mut self, sensitivity: f32) {
        self.exposure_sensitivity = sensitivity;
    }

    pub fn get_exposure_sensitivity(&self) -> f32 {
        self.exposure_sensitivity
    }

    pub fn set_auto_exposure_enabled(&mut self, enabled: bool) {
        self.auto_exposure_enabled = enabled;
    }

    pub fn is_auto_exposure_enabled(&self) -> bool {
        self.auto_exposure_enabled
    }

    pub fn set_auto_exposure_min(&mut self, min: f32) {
        self.auto_exposure_min = min;
    }

    pub fn get_auto_exposure_min(&self) -> f32 {
        self.auto_exposure_min
    }

    pub fn set_auto_exposure_max(&mut self, max: f32) {
        self.auto_exposure_max = max;
    }

    pub fn get_auto_exposure_max(&self) -> f32 {
        self.auto_exposure_max
    }

    pub fn set_auto_exposure_speed(&mut self, speed: f32) {
        self.auto_exposure_speed = speed;
    }

    pub fn get_auto_exposure_speed(&self) -> f32 {
        self.auto_exposure_speed
    }

    pub fn set_auto_exposure_scale(&mut self, scale: f32) {
        self.auto_exposure_scale = scale;
    }

    pub fn get_auto_exposure_scale(&self) -> f32 {
        self.auto_exposure_scale
    }

    /// Base class normalization.
    pub fn calculate_exposure_normalization(&self) -> f32 {
        1.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CameraAttributesPracticalData {
    pub base: CameraAttributesData,
    dof_blur_far_enabled: bool,
    dof_blur_far_distance: f32,
    dof_blur_far_transition: f32,
    dof_blur_near_enabled: bool,
    dof_blur_near_distance: f32,
    dof_blur_near_transition: f32,
    dof_blur_amount: f32,
}

impl Default for CameraAttributesPracticalData {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraAttributesPracticalData {
    pub fn new() -> Self {
        let mut base = CameraAttributesData::new();
        // The constructor overrides the auto-exposure sensitivity range.
        base.set_auto_exposure_min(0.0);
        base.set_auto_exposure_max(800.0);
        Self {
            base,
            dof_blur_far_enabled: false,
            dof_blur_far_distance: 10.0,
            dof_blur_far_transition: 5.0,
            dof_blur_near_enabled: false,
            dof_blur_near_distance: 2.0,
            dof_blur_near_transition: 1.0,
            dof_blur_amount: 0.1,
        }
    }

    pub fn set_dof_blur_far_enabled(&mut self, enabled: bool) {
        self.dof_blur_far_enabled = enabled;
    }

    pub fn is_dof_blur_far_enabled(&self) -> bool {
        self.dof_blur_far_enabled
    }

    pub fn set_dof_blur_far_distance(&mut self, distance: f32) {
        self.dof_blur_far_distance = distance;
    }

    pub fn get_dof_blur_far_distance(&self) -> f32 {
        self.dof_blur_far_distance
    }

    pub fn set_dof_blur_far_transition(&mut self, transition: f32) {
        self.dof_blur_far_transition = transition;
    }

    pub fn get_dof_blur_far_transition(&self) -> f32 {
        self.dof_blur_far_transition
    }

    pub fn set_dof_blur_near_enabled(&mut self, enabled: bool) {
        self.dof_blur_near_enabled = enabled;
    }

    pub fn is_dof_blur_near_enabled(&self) -> bool {
        self.dof_blur_near_enabled
    }

    pub fn set_dof_blur_near_distance(&mut self, distance: f32) {
        self.dof_blur_near_distance = distance;
    }

    pub fn get_dof_blur_near_distance(&self) -> f32 {
        self.dof_blur_near_distance
    }

    pub fn set_dof_blur_near_transition(&mut self, transition: f32) {
        self.dof_blur_near_transition = transition;
    }

    pub fn get_dof_blur_near_transition(&self) -> f32 {
        self.dof_blur_near_transition
    }

    pub fn set_dof_blur_amount(&mut self, amount: f32) {
        self.dof_blur_amount = amount;
    }

    pub fn get_dof_blur_amount(&self) -> f32 {
        self.dof_blur_amount
    }

    /// `exposure_sensitivity / 3072007`.
    pub fn calculate_exposure_normalization(&self) -> f32 {
        self.base.get_exposure_sensitivity() / 3072007.0
    }
}

/// Sensor height used by `_update_frustum` (high-end DSLR, 36x24 mm).
const SENSOR_HEIGHT: f32 = 24.0;

#[derive(Clone, Copy, Debug)]
pub struct CameraAttributesPhysicalData {
    pub base: CameraAttributesData,
    exposure_aperture: f32,
    exposure_shutter_speed: f32,
    frustum_focal_length: f32,
    frustum_focus_distance: f32,
    frustum_near: f32,
    frustum_far: f32,
    frustum_fov: f32,
}

impl Default for CameraAttributesPhysicalData {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraAttributesPhysicalData {
    pub fn new() -> Self {
        let mut base = CameraAttributesData::new();
        base.set_auto_exposure_min(-8.0);
        base.set_auto_exposure_max(10.0);

        let mut attributes = Self {
            base,
            exposure_aperture: 16.0,
            exposure_shutter_speed: 100.0,
            frustum_focal_length: 35.0,
            frustum_focus_distance: 10.0,
            frustum_near: 0.05,
            frustum_far: 4000.0,
            frustum_fov: 75.0,
        };
        // The constructor calls `_update_frustum`.
        attributes.update_frustum_fov();
        attributes
    }

    fn update_frustum_fov(&mut self) {
        self.frustum_fov = funcs::rad_to_deg(2.0
            * (SENSOR_HEIGHT / (2.0 * self.frustum_focal_length)).atan());
    }

    pub fn set_aperture(&mut self, aperture: f32) {
        self.exposure_aperture = aperture;
    }

    pub fn get_aperture(&self) -> f32 {
        self.exposure_aperture
    }

    pub fn set_shutter_speed(&mut self, shutter_speed: f32) {
        self.exposure_shutter_speed = shutter_speed;
    }

    pub fn get_shutter_speed(&self) -> f32 {
        self.exposure_shutter_speed
    }

    pub fn set_focal_length(&mut self, focal_length: f32) {
        self.frustum_focal_length = focal_length;
        self.update_frustum_fov();
    }

    pub fn get_focal_length(&self) -> f32 {
        self.frustum_focal_length
    }

    pub fn set_focus_distance(&mut self, focus_distance: f32) {
        self.frustum_focus_distance = focus_distance;
    }

    pub fn get_focus_distance(&self) -> f32 {
        self.frustum_focus_distance
    }

    pub fn set_near(&mut self, near: f32) {
        self.frustum_near = near;
    }

    pub fn get_near(&self) -> f32 {
        self.frustum_near
    }

    pub fn set_far(&mut self, far: f32) {
        self.frustum_far = far;
    }

    pub fn get_far(&self) -> f32 {
        self.frustum_far
    }

    pub fn get_fov(&self) -> f32 {
        self.frustum_fov
    }

    /// `1 / (aperture² · shutter · (100 / sensitivity) · 1.2)`.
    pub fn calculate_exposure_normalization(&self) -> f32 {
        let e = (self.exposure_aperture * self.exposure_aperture)
            * self.exposure_shutter_speed
            * (100.0 / self.base.get_exposure_sensitivity());
        1.0 / (e * 1.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_defaults() {
        let a = CameraAttributesData::new();
        assert_eq!(a.get_exposure_multiplier(), 1.0);
        assert_eq!(a.get_exposure_sensitivity(), 100.0);
        assert!(!a.is_auto_exposure_enabled());
        assert_eq!(a.get_auto_exposure_min(), 0.01);
        assert_eq!(a.get_auto_exposure_max(), 64.0);
        assert_eq!(a.get_auto_exposure_speed(), 0.5);
        assert_eq!(a.get_auto_exposure_scale(), 0.4);
        assert_eq!(a.calculate_exposure_normalization(), 1.0);
    }

    #[test]
    fn practical_defaults_and_normalization() {
        let mut p = CameraAttributesPracticalData::new();
        assert_eq!(p.base.get_auto_exposure_min(), 0.0);
        assert_eq!(p.base.get_auto_exposure_max(), 800.0);
        assert!(!p.is_dof_blur_far_enabled());
        assert_eq!(p.get_dof_blur_far_distance(), 10.0);
        assert_eq!(p.get_dof_blur_far_transition(), 5.0);
        assert!(!p.is_dof_blur_near_enabled());
        assert_eq!(p.get_dof_blur_near_distance(), 2.0);
        assert_eq!(p.get_dof_blur_near_transition(), 1.0);
        assert_eq!(p.get_dof_blur_amount(), 0.1);

        let n = p.calculate_exposure_normalization();
        assert!((n - 100.0 / 3072007.0).abs() < 1e-12, "{n}");

        p.set_dof_blur_amount(0.5);
        p.set_dof_blur_near_distance(3.0);
        p.set_dof_blur_far_enabled(true);
        assert_eq!(p.get_dof_blur_amount(), 0.5);
        assert_eq!(p.get_dof_blur_near_distance(), 3.0);
        assert!(p.is_dof_blur_far_enabled());

        p.base.set_exposure_sensitivity(200.0);
        assert!((p.calculate_exposure_normalization() - 200.0 / 3072007.0).abs() < 1e-12);
    }

    #[test]
    fn physical_defaults_fov_and_normalization() {
        let mut p = CameraAttributesPhysicalData::new();
        assert_eq!(p.get_aperture(), 16.0);
        assert_eq!(p.get_shutter_speed(), 100.0);
        assert_eq!(p.get_focal_length(), 35.0);
        assert_eq!(p.get_focus_distance(), 10.0);
        assert_eq!(p.get_near(), 0.05);
        assert_eq!(p.get_far(), 4000.0);
        assert_eq!(p.base.get_auto_exposure_min(), -8.0);
        assert_eq!(p.base.get_auto_exposure_max(), 10.0);

        // 2*atan(24 / (2*35)) in degrees.
        let expected = (2.0f32 * (24.0f32 / 70.0).atan()).to_degrees();
        assert!((p.get_fov() - expected).abs() < 1e-4, "{}", p.get_fov());

        // Default normalization equals the practical default at ISO 100.
        let n = p.calculate_exposure_normalization();
        assert!((n - 1.0 / 30720.0).abs() < 1e-9, "{n}");

        // Longer lens narrows the FOV.
        let wide = p.get_fov();
        p.set_focal_length(70.0);
        assert!(p.get_fov() < wide);
        assert!((p.get_fov() - (2.0f32 * (24.0f32 / 140.0).atan()).to_degrees()).abs() < 1e-4);

        // Aperture and shutter feed the normalization directly.
        p.set_aperture(4.0);
        p.set_shutter_speed(250.0);
        let e = 16.0 * 250.0 * 1.0;
        assert!((p.calculate_exposure_normalization() - 1.0 / (e * 1.2)).abs() < 1e-9);
    }
}
