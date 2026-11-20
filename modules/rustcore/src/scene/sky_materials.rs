//! `scene/resources/3d/sky_material.*` (state): the sky material resources.
//!
//! Ported configuration with the engine constructors' defaults:
//!
//! - `ProceduralSkyMaterial`: sky/ground colours, curves, energy multipliers, sky
//!   cover + modulate, sun angle/curve, debanding on, global energy 1.0;
//! - `PanoramaSkyMaterial`: panorama handle, energy 1.0, filtering on;
//! - `PhysicalSkyMaterial`: Rayleigh/Mie coefficients, colours and eccentricity,
//!   turbidity 10, sun disk 1.0, ground colour, energy 1.0, exposure, debanding on
//!   and the night-sky texture.
//!
//! Shader creation and the material RIDs are not ported.

use crate::math::color::Color;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct ProceduralSkyMaterialData {
    sky_top_color: Color,
    sky_horizon_color: Color,
    sky_curve: f32,
    sky_energy_multiplier: f32,
    sky_cover: Option<Rid>,
    sky_cover_modulate: Color,
    ground_bottom_color: Color,
    ground_horizon_color: Color,
    ground_curve: f32,
    ground_energy_multiplier: f32,
    sun_angle_max: f32,
    sun_curve: f32,
    use_debanding: bool,
    global_energy_multiplier: f32,
}

impl Default for ProceduralSkyMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl ProceduralSkyMaterialData {
    pub fn new() -> Self {
        Self {
            sky_top_color: Color::new(0.385, 0.454, 0.55, 1.0),
            sky_horizon_color: Color::new(0.6463, 0.6558, 0.6708, 1.0),
            sky_curve: 0.15,
            sky_energy_multiplier: 1.0,
            sky_cover: None,
            sky_cover_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            ground_bottom_color: Color::new(0.2, 0.169, 0.133, 1.0),
            ground_horizon_color: Color::new(0.6463, 0.6558, 0.6708, 1.0),
            ground_curve: 0.02,
            ground_energy_multiplier: 1.0,
            sun_angle_max: 30.0,
            sun_curve: 0.15,
            use_debanding: true,
            global_energy_multiplier: 1.0,
        }
    }

    pub fn set_sky_top_color(&mut self, color: Color) {
        self.sky_top_color = color;
    }

    pub fn get_sky_top_color(&self) -> Color {
        self.sky_top_color
    }

    pub fn set_sky_horizon_color(&mut self, color: Color) {
        self.sky_horizon_color = color;
    }

    pub fn get_sky_horizon_color(&self) -> Color {
        self.sky_horizon_color
    }

    pub fn set_sky_curve(&mut self, curve: f32) {
        self.sky_curve = curve;
    }

    pub fn get_sky_curve(&self) -> f32 {
        self.sky_curve
    }

    pub fn set_sky_energy_multiplier(&mut self, multiplier: f32) {
        self.sky_energy_multiplier = multiplier;
    }

    pub fn get_sky_energy_multiplier(&self) -> f32 {
        self.sky_energy_multiplier
    }

    pub fn set_sky_cover(&mut self, cover: Option<Rid>) {
        self.sky_cover = cover;
    }

    pub fn get_sky_cover(&self) -> Option<Rid> {
        self.sky_cover
    }

    pub fn set_sky_cover_modulate(&mut self, color: Color) {
        self.sky_cover_modulate = color;
    }

    pub fn get_sky_cover_modulate(&self) -> Color {
        self.sky_cover_modulate
    }

    pub fn set_ground_bottom_color(&mut self, color: Color) {
        self.ground_bottom_color = color;
    }

    pub fn get_ground_bottom_color(&self) -> Color {
        self.ground_bottom_color
    }

    pub fn set_ground_horizon_color(&mut self, color: Color) {
        self.ground_horizon_color = color;
    }

    pub fn get_ground_horizon_color(&self) -> Color {
        self.ground_horizon_color
    }

    pub fn set_ground_curve(&mut self, curve: f32) {
        self.ground_curve = curve;
    }

    pub fn get_ground_curve(&self) -> f32 {
        self.ground_curve
    }

    pub fn set_ground_energy_multiplier(&mut self, multiplier: f32) {
        self.ground_energy_multiplier = multiplier;
    }

    pub fn get_ground_energy_multiplier(&self) -> f32 {
        self.ground_energy_multiplier
    }

    pub fn set_sun_angle_max(&mut self, angle: f32) {
        self.sun_angle_max = angle;
    }

    pub fn get_sun_angle_max(&self) -> f32 {
        self.sun_angle_max
    }

    pub fn set_sun_curve(&mut self, curve: f32) {
        self.sun_curve = curve;
    }

    pub fn get_sun_curve(&self) -> f32 {
        self.sun_curve
    }

    pub fn set_use_debanding(&mut self, enabled: bool) {
        self.use_debanding = enabled;
    }

    pub fn get_use_debanding(&self) -> bool {
        self.use_debanding
    }

    pub fn set_energy_multiplier(&mut self, multiplier: f32) {
        self.global_energy_multiplier = multiplier;
    }

    pub fn get_energy_multiplier(&self) -> f32 {
        self.global_energy_multiplier
    }
}

#[derive(Clone, Debug)]
pub struct PanoramaSkyMaterialData {
    panorama: Option<Rid>,
    energy_multiplier: f32,
    filter: bool,
}

impl Default for PanoramaSkyMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl PanoramaSkyMaterialData {
    pub fn new() -> Self {
        Self {
            panorama: None,
            energy_multiplier: 1.0,
            filter: true,
        }
    }

    pub fn set_panorama(&mut self, panorama: Option<Rid>) {
        self.panorama = panorama;
    }

    pub fn get_panorama(&self) -> Option<Rid> {
        self.panorama
    }

    pub fn set_filtering_enabled(&mut self, enabled: bool) {
        self.filter = enabled;
    }

    pub fn is_filtering_enabled(&self) -> bool {
        self.filter
    }

    pub fn set_energy_multiplier(&mut self, multiplier: f32) {
        self.energy_multiplier = multiplier;
    }

    pub fn get_energy_multiplier(&self) -> f32 {
        self.energy_multiplier
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalSkyMaterialData {
    rayleigh: f32,
    rayleigh_color: Color,
    mie: f32,
    mie_eccentricity: f32,
    mie_color: Color,
    turbidity: f32,
    sun_disk_scale: f32,
    ground_color: Color,
    energy_multiplier: f32,
    exposure: f32,
    use_debanding: bool,
    night_sky: Option<Rid>,
}

impl Default for PhysicalSkyMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalSkyMaterialData {
    pub fn new() -> Self {
        Self {
            rayleigh: 2.0,
            rayleigh_color: Color::new(0.3, 0.405, 0.6, 1.0),
            mie: 0.005,
            mie_eccentricity: 0.8,
            mie_color: Color::new(0.69, 0.729, 0.812, 1.0),
            turbidity: 10.0,
            sun_disk_scale: 1.0,
            ground_color: Color::new(0.1, 0.07, 0.034, 1.0),
            energy_multiplier: 1.0,
            exposure: 0.0,
            use_debanding: true,
            night_sky: None,
        }
    }

    pub fn set_rayleigh_coefficient(&mut self, value: f32) {
        self.rayleigh = value;
    }

    pub fn get_rayleigh_coefficient(&self) -> f32 {
        self.rayleigh
    }

    pub fn set_rayleigh_color(&mut self, color: Color) {
        self.rayleigh_color = color;
    }

    pub fn get_rayleigh_color(&self) -> Color {
        self.rayleigh_color
    }

    pub fn set_mie_coefficient(&mut self, value: f32) {
        self.mie = value;
    }

    pub fn get_mie_coefficient(&self) -> f32 {
        self.mie
    }

    pub fn set_mie_eccentricity(&mut self, value: f32) {
        self.mie_eccentricity = value;
    }

    pub fn get_mie_eccentricity(&self) -> f32 {
        self.mie_eccentricity
    }

    pub fn set_mie_color(&mut self, color: Color) {
        self.mie_color = color;
    }

    pub fn get_mie_color(&self) -> Color {
        self.mie_color
    }

    pub fn set_turbidity(&mut self, value: f32) {
        self.turbidity = value;
    }

    pub fn get_turbidity(&self) -> f32 {
        self.turbidity
    }

    pub fn set_sun_disk_scale(&mut self, value: f32) {
        self.sun_disk_scale = value;
    }

    pub fn get_sun_disk_scale(&self) -> f32 {
        self.sun_disk_scale
    }

    pub fn set_ground_color(&mut self, color: Color) {
        self.ground_color = color;
    }

    pub fn get_ground_color(&self) -> Color {
        self.ground_color
    }

    pub fn set_energy_multiplier(&mut self, value: f32) {
        self.energy_multiplier = value;
    }

    pub fn get_energy_multiplier(&self) -> f32 {
        self.energy_multiplier
    }

    pub fn set_exposure_value(&mut self, exposure: f32) {
        self.exposure = exposure;
    }

    pub fn get_exposure_value(&self) -> f32 {
        self.exposure
    }

    pub fn set_use_debanding(&mut self, enabled: bool) {
        self.use_debanding = enabled;
    }

    pub fn get_use_debanding(&self) -> bool {
        self.use_debanding
    }

    pub fn set_night_sky(&mut self, texture: Option<Rid>) {
        self.night_sky = texture;
    }

    pub fn get_night_sky(&self) -> Option<Rid> {
        self.night_sky
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procedural_defaults_and_setters() {
        let mut sky = ProceduralSkyMaterialData::new();
        assert_eq!(sky.get_sky_top_color(), Color::new(0.385, 0.454, 0.55, 1.0));
        assert_eq!(
            sky.get_sky_horizon_color(),
            Color::new(0.6463, 0.6558, 0.6708, 1.0)
        );
        assert_eq!(sky.get_sky_curve(), 0.15);
        assert_eq!(sky.get_sky_energy_multiplier(), 1.0);
        assert!(sky.get_sky_cover().is_none());
        assert_eq!(sky.get_sky_cover_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(
            sky.get_ground_bottom_color(),
            Color::new(0.2, 0.169, 0.133, 1.0)
        );
        assert_eq!(sky.get_ground_curve(), 0.02);
        assert_eq!(sky.get_sun_angle_max(), 30.0);
        assert_eq!(sky.get_sun_curve(), 0.15);
        assert!(sky.get_use_debanding());
        assert_eq!(sky.get_energy_multiplier(), 1.0);

        sky.set_sky_top_color(Color::new(0.1, 0.2, 0.3, 1.0));
        sky.set_sky_horizon_color(Color::new(0.4, 0.5, 0.6, 1.0));
        sky.set_sky_curve(0.5);
        sky.set_sky_energy_multiplier(2.0);
        sky.set_sky_cover(Some(Rid::from_u64(1)));
        sky.set_sky_cover_modulate(Color::new(0.5, 0.5, 0.5, 1.0));
        sky.set_ground_bottom_color(Color::new(0.2, 0.1, 0.0, 1.0));
        sky.set_ground_horizon_color(Color::new(0.7, 0.7, 0.7, 1.0));
        sky.set_ground_curve(0.1);
        sky.set_ground_energy_multiplier(3.0);
        sky.set_sun_angle_max(45.0);
        sky.set_sun_curve(0.3);
        sky.set_use_debanding(false);
        sky.set_energy_multiplier(1.5);

        assert_eq!(sky.get_sky_top_color(), Color::new(0.1, 0.2, 0.3, 1.0));
        assert_eq!(sky.get_sky_curve(), 0.5);
        assert_eq!(sky.get_sky_energy_multiplier(), 2.0);
        assert_eq!(sky.get_sky_cover(), Some(Rid::from_u64(1)));
        assert_eq!(sky.get_sky_cover_modulate(), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(sky.get_ground_bottom_color(), Color::new(0.2, 0.1, 0.0, 1.0));
        assert_eq!(sky.get_ground_horizon_color(), Color::new(0.7, 0.7, 0.7, 1.0));
        assert_eq!(sky.get_ground_curve(), 0.1);
        assert_eq!(sky.get_ground_energy_multiplier(), 3.0);
        assert_eq!(sky.get_sun_angle_max(), 45.0);
        assert_eq!(sky.get_sun_curve(), 0.3);
        assert!(!sky.get_use_debanding());
        assert_eq!(sky.get_energy_multiplier(), 1.5);
    }

    #[test]
    fn panorama_defaults_and_setters() {
        let mut sky = PanoramaSkyMaterialData::new();
        assert!(sky.get_panorama().is_none());
        assert_eq!(sky.get_energy_multiplier(), 1.0);
        assert!(sky.is_filtering_enabled());
        sky.set_panorama(Some(Rid::from_u64(2)));
        sky.set_energy_multiplier(4.0);
        sky.set_filtering_enabled(false);
        assert_eq!(sky.get_panorama(), Some(Rid::from_u64(2)));
        assert_eq!(sky.get_energy_multiplier(), 4.0);
        assert!(!sky.is_filtering_enabled());
    }

    #[test]
    fn physical_defaults_and_setters() {
        let mut sky = PhysicalSkyMaterialData::new();
        assert_eq!(sky.get_rayleigh_coefficient(), 2.0);
        assert_eq!(sky.get_rayleigh_color(), Color::new(0.3, 0.405, 0.6, 1.0));
        assert_eq!(sky.get_mie_coefficient(), 0.005);
        assert_eq!(sky.get_mie_eccentricity(), 0.8);
        assert_eq!(sky.get_mie_color(), Color::new(0.69, 0.729, 0.812, 1.0));
        assert_eq!(sky.get_turbidity(), 10.0);
        assert_eq!(sky.get_sun_disk_scale(), 1.0);
        assert_eq!(sky.get_ground_color(), Color::new(0.1, 0.07, 0.034, 1.0));
        assert_eq!(sky.get_energy_multiplier(), 1.0);
        assert_eq!(sky.get_exposure_value(), 0.0);
        assert!(sky.get_use_debanding());
        assert!(sky.get_night_sky().is_none());

        sky.set_rayleigh_coefficient(3.0);
        sky.set_rayleigh_color(Color::new(0.1, 0.1, 0.5, 1.0));
        sky.set_mie_coefficient(0.01);
        sky.set_mie_eccentricity(0.5);
        sky.set_mie_color(Color::new(0.5, 0.5, 0.5, 1.0));
        sky.set_turbidity(20.0);
        sky.set_sun_disk_scale(2.0);
        sky.set_ground_color(Color::new(0.0, 0.1, 0.0, 1.0));
        sky.set_energy_multiplier(2.0);
        sky.set_exposure_value(1.25);
        sky.set_use_debanding(false);
        sky.set_night_sky(Some(Rid::from_u64(7)));

        assert_eq!(sky.get_rayleigh_coefficient(), 3.0);
        assert_eq!(sky.get_rayleigh_color(), Color::new(0.1, 0.1, 0.5, 1.0));
        assert_eq!(sky.get_mie_coefficient(), 0.01);
        assert_eq!(sky.get_mie_eccentricity(), 0.5);
        assert_eq!(sky.get_mie_color(), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(sky.get_turbidity(), 20.0);
        assert_eq!(sky.get_sun_disk_scale(), 2.0);
        assert_eq!(sky.get_ground_color(), Color::new(0.0, 0.1, 0.0, 1.0));
        assert_eq!(sky.get_energy_multiplier(), 2.0);
        assert_eq!(sky.get_exposure_value(), 1.25);
        assert!(!sky.get_use_debanding());
        assert_eq!(sky.get_night_sky(), Some(Rid::from_u64(7)));
    }
}
