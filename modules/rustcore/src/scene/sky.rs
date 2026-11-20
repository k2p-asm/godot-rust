//! `scene/resources/sky.*` and `sky_material.*` (state half).
//!
//! `Sky` keeps the sky material handle, the radiance size (default 256, with the
//! engine's `32..=2048` ladder and the `RADIANCE_SIZE_MAX` rejection) and the
//! process mode (automatic by default).
//!
//! `ProceduralSkyMaterial` defaults come from its constructor: sky top
//! `(0.385, 0.454, 0.55)`, horizon `(0.6463, 0.6558, 0.6708)`, sky curve 0.15,
//! sky energy 1, white cover modulate, ground bottom `(0.2, 0.169, 0.133)`,
//! ground horizon equal to the sky horizon, ground curve 0.02, ground energy 1,
//! sun angle 30, sun curve 0.15, debanding on and the global energy 1.
//!
//! `PanoramaSkyMaterial` keeps the panorama handle, energy 1 and filtering on.
//!
//! Shader compilation and the material parameters uploaded to the
//! RenderingServer are not ported.

use crate::math::color::Color;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RadianceSize {
    Size32,
    Size64,
    Size128,
    Size256,
    Size512,
    Size1024,
    Size2048,
    Max,
}

impl RadianceSize {
    /// The pixel sizes the engine uploads.
    pub fn pixels(self) -> Option<i32> {
        match self {
            RadianceSize::Size32 => Some(32),
            RadianceSize::Size64 => Some(64),
            RadianceSize::Size128 => Some(128),
            RadianceSize::Size256 => Some(256),
            RadianceSize::Size512 => Some(512),
            RadianceSize::Size1024 => Some(1024),
            RadianceSize::Size2048 => Some(2048),
            RadianceSize::Max => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ProcessMode {
    Automatic,
    Quality,
    Incremental,
    Realtime,
}

#[derive(Clone, Debug)]
pub struct SkyData {
    mode: ProcessMode,
    radiance_size: RadianceSize,
    sky_material: Option<Rid>,
}

impl Default for SkyData {
    fn default() -> Self {
        Self::new()
    }
}

impl SkyData {
    pub fn new() -> Self {
        Self {
            mode: ProcessMode::Automatic,
            radiance_size: RadianceSize::Size256,
            sky_material: None,
        }
    }

    /// `ERR_FAIL_INDEX(size, RADIANCE_SIZE_MAX)`.
    pub fn set_radiance_size(&mut self, size: RadianceSize) -> bool {
        if size == RadianceSize::Max {
            return false;
        }
        self.radiance_size = size;
        true
    }

    pub fn get_radiance_size(&self) -> RadianceSize {
        self.radiance_size
    }

    pub fn set_process_mode(&mut self, mode: ProcessMode) {
        self.mode = mode;
    }

    pub fn get_process_mode(&self) -> ProcessMode {
        self.mode
    }

    pub fn set_material(&mut self, material: Option<Rid>) {
        self.sky_material = material;
    }

    pub fn get_material(&self) -> Option<Rid> {
        self.sky_material
    }
}

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
    energy_multiplier: f32,
}

impl Default for ProceduralSkyMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! sky_accessors {
    ($field:ident, $set:ident, $get:ident, $ty:ty) => {
        pub fn $set(&mut self, value: $ty) {
            self.$field = value;
        }

        pub fn $get(&self) -> $ty {
            self.$field
        }
    };
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
            energy_multiplier: 1.0,
        }
    }

    sky_accessors!(
        sky_top_color,
        set_sky_top_color,
        get_sky_top_color,
        Color
    );
    sky_accessors!(
        sky_horizon_color,
        set_sky_horizon_color,
        get_sky_horizon_color,
        Color
    );
    sky_accessors!(sky_curve, set_sky_curve, get_sky_curve, f32);
    sky_accessors!(
        sky_energy_multiplier,
        set_sky_energy_multiplier,
        get_sky_energy_multiplier,
        f32
    );
    sky_accessors!(sky_cover, set_sky_cover, get_sky_cover, Option<Rid>);
    sky_accessors!(
        sky_cover_modulate,
        set_sky_cover_modulate,
        get_sky_cover_modulate,
        Color
    );
    sky_accessors!(
        ground_bottom_color,
        set_ground_bottom_color,
        get_ground_bottom_color,
        Color
    );
    sky_accessors!(
        ground_horizon_color,
        set_ground_horizon_color,
        get_ground_horizon_color,
        Color
    );
    sky_accessors!(ground_curve, set_ground_curve, get_ground_curve, f32);
    sky_accessors!(
        ground_energy_multiplier,
        set_ground_energy_multiplier,
        get_ground_energy_multiplier,
        f32
    );
    sky_accessors!(sun_angle_max, set_sun_angle_max, get_sun_angle_max, f32);
    sky_accessors!(sun_curve, set_sun_curve, get_sun_curve, f32);
    sky_accessors!(
        use_debanding,
        set_use_debanding,
        get_use_debanding,
        bool
    );
    sky_accessors!(
        energy_multiplier,
        set_energy_multiplier,
        get_energy_multiplier,
        f32
    );

    /// `inv_sky_curve`, the value the engine uploads (`0.6 / curve`).
    pub fn get_inverse_sky_curve(&self) -> f32 {
        0.6 / self.sky_curve
    }

    /// `inv_ground_curve` (`0.6 / curve`).
    pub fn get_inverse_ground_curve(&self) -> f32 {
        0.6 / self.ground_curve
    }

    /// `inv_sun_curve` (`1.6 / curve^1.4`).
    pub fn get_inverse_sun_curve(&self) -> f32 {
        1.6 / self.sun_curve.powf(1.4)
    }

    /// The cosine the shader receives for `sun_angle_max`.
    pub fn get_sun_angle_max_cos(&self) -> f32 {
        crate::math::funcs::deg_to_rad(self.sun_angle_max).cos()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_defaults_and_radiance_validation() {
        let mut s = SkyData::new();
        assert_eq!(s.get_process_mode(), ProcessMode::Automatic);
        assert_eq!(s.get_radiance_size(), RadianceSize::Size256);
        assert_eq!(s.get_radiance_size().pixels(), Some(256));
        assert!(s.get_material().is_none());

        assert!(!s.set_radiance_size(RadianceSize::Max));
        assert_eq!(s.get_radiance_size(), RadianceSize::Size256);
        assert!(s.set_radiance_size(RadianceSize::Size2048));
        assert_eq!(s.get_radiance_size().pixels(), Some(2048));
        assert!(s.set_radiance_size(RadianceSize::Size32));
        assert_eq!(s.get_radiance_size().pixels(), Some(32));
        assert_eq!(RadianceSize::Max.pixels(), None);

        s.set_process_mode(ProcessMode::Realtime);
        s.set_material(Some(Rid::from_u64(5)));
        assert_eq!(s.get_process_mode(), ProcessMode::Realtime);
        assert_eq!(s.get_material(), Some(Rid::from_u64(5)));

        assert_eq!(RadianceSize::Size32 as i32, 0);
        assert_eq!(RadianceSize::Size2048 as i32, 6);
        assert_eq!(RadianceSize::Max as i32, 7);
        assert_eq!(ProcessMode::Automatic as i32, 0);
        assert_eq!(ProcessMode::Quality as i32, 1);
        assert_eq!(ProcessMode::Incremental as i32, 2);
        assert_eq!(ProcessMode::Realtime as i32, 3);
    }

    #[test]
    fn procedural_defaults() {
        let p = ProceduralSkyMaterialData::new();
        assert_eq!(p.get_sky_top_color(), Color::new(0.385, 0.454, 0.55, 1.0));
        assert_eq!(
            p.get_sky_horizon_color(),
            Color::new(0.6463, 0.6558, 0.6708, 1.0)
        );
        assert_eq!(p.get_sky_curve(), 0.15);
        assert_eq!(p.get_sky_energy_multiplier(), 1.0);
        assert!(p.get_sky_cover().is_none());
        assert_eq!(p.get_sky_cover_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(
            p.get_ground_bottom_color(),
            Color::new(0.2, 0.169, 0.133, 1.0)
        );
        assert_eq!(p.get_ground_horizon_color(), p.get_sky_horizon_color());
        assert_eq!(p.get_ground_curve(), 0.02);
        assert_eq!(p.get_ground_energy_multiplier(), 1.0);
        assert_eq!(p.get_sun_angle_max(), 30.0);
        assert_eq!(p.get_sun_curve(), 0.15);
        assert!(p.get_use_debanding());
        assert_eq!(p.get_energy_multiplier(), 1.0);

        // Derived shader values.
        assert!((p.get_inverse_sky_curve() - 0.6 / 0.15).abs() < 1e-4);
        assert!((p.get_inverse_ground_curve() - 0.6 / 0.02).abs() < 1e-3);
        assert!((p.get_inverse_sun_curve() - 1.6 / 0.15f32.powf(1.4)).abs() < 1e-3);
        assert!((p.get_sun_angle_max_cos() - 30.0f32.to_radians().cos()).abs() < 1e-5);
    }

    #[test]
    fn procedural_setters() {
        let mut p = ProceduralSkyMaterialData::new();
        p.set_sky_top_color(Color::new(0.1, 0.2, 0.3, 1.0));
        p.set_sky_horizon_color(Color::new(0.4, 0.5, 0.6, 1.0));
        p.set_sky_curve(0.5);
        p.set_sky_energy_multiplier(2.0);
        p.set_sky_cover(Some(Rid::from_u64(1)));
        p.set_sky_cover_modulate(Color::new(0.9, 0.8, 0.7, 2.0));
        p.set_ground_bottom_color(Color::new(0.05, 0.05, 0.05, 1.0));
        p.set_ground_horizon_color(Color::new(0.2, 0.2, 0.2, 1.0));
        p.set_ground_curve(0.1);
        p.set_ground_energy_multiplier(0.5);
        p.set_sun_angle_max(45.0);
        p.set_sun_curve(0.25);
        p.set_use_debanding(false);
        p.set_energy_multiplier(1.5);

        assert_eq!(p.get_sky_top_color(), Color::new(0.1, 0.2, 0.3, 1.0));
        assert_eq!(p.get_sky_horizon_color(), Color::new(0.4, 0.5, 0.6, 1.0));
        assert_eq!(p.get_sky_curve(), 0.5);
        assert_eq!(p.get_sky_energy_multiplier(), 2.0);
        assert_eq!(p.get_sky_cover(), Some(Rid::from_u64(1)));
        assert_eq!(p.get_sky_cover_modulate(), Color::new(0.9, 0.8, 0.7, 2.0));
        assert_eq!(p.get_ground_bottom_color(), Color::new(0.05, 0.05, 0.05, 1.0));
        assert_eq!(p.get_ground_horizon_color(), Color::new(0.2, 0.2, 0.2, 1.0));
        assert_eq!(p.get_ground_curve(), 0.1);
        assert_eq!(p.get_ground_energy_multiplier(), 0.5);
        assert_eq!(p.get_sun_angle_max(), 45.0);
        assert_eq!(p.get_sun_curve(), 0.25);
        assert!(!p.get_use_debanding());
        assert_eq!(p.get_energy_multiplier(), 1.5);
    }

    #[test]
    fn panorama_defaults_and_setters() {
        let mut p = PanoramaSkyMaterialData::new();
        assert!(p.get_panorama().is_none());
        assert_eq!(p.get_energy_multiplier(), 1.0);
        assert!(p.is_filtering_enabled());

        p.set_panorama(Some(Rid::from_u64(2)));
        p.set_energy_multiplier(3.0);
        p.set_filtering_enabled(false);
        assert_eq!(p.get_panorama(), Some(Rid::from_u64(2)));
        assert_eq!(p.get_energy_multiplier(), 3.0);
        assert!(!p.is_filtering_enabled());

        p.set_panorama(None);
        assert!(p.get_panorama().is_none());
    }
}
