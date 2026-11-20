//! `scene/resources/environment.*`: every world-environment parameter.
//!
//! All defaults are the header's in-class initialisers or the constructor body
//! (`glow_levels = [0, 0.8, 0.4, 0.1, 0, 0, 0]`). Only two setters validate:
//! `set_sdfgi_cascades` (`1..=8`) and `set_glow_level` (`< MAX_GLOW_LEVELS`).
//! Everything else assigns as given, matching `environment.cpp`.
//!
//! The SDFGI distances are derived, exactly as in the engine:
//! `cascade0 = min_cell_size * 64`, `max = cascade0 * 2^cascades`.
//!
//! Sky/glow-map/colour-correction textures are `Rid` handles. Rendering-server
//! updates and the `_update_*` staging are not ported.

use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

/// `RenderingServer::MAX_GLOW_LEVELS`.
pub const MAX_GLOW_LEVELS: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BgMode {
    ClearColor,
    Color,
    Sky,
    Canvas,
    Keep,
    CameraFeed,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AmbientSource {
    Bg,
    Disabled,
    Color,
    Sky,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ReflectionSource {
    Bg,
    Disabled,
    Sky,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ToneMapper {
    Linear,
    Reinhardt,
    Filmic,
    Aces,
    Agx,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SdfgiYScale {
    Scale50,
    Scale75,
    Scale100,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FogMode {
    Exponential,
    Depth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GlowBlendMode {
    Additive,
    Screen,
    Softlight,
    Replace,
    Mix,
}

#[derive(Clone, Debug)]
pub struct EnvironmentData {
    // Background.
    bg_mode: BgMode,
    bg_sky: Option<Rid>,
    bg_sky_custom_fov: f32,
    bg_sky_rotation: Vector3,
    bg_color: Color,
    bg_canvas_max_layer: i32,
    bg_camera_feed_id: i32,
    bg_energy_multiplier: f32,
    bg_intensity: f32,
    // Ambient.
    ambient_color: Color,
    ambient_source: AmbientSource,
    ambient_energy: f32,
    ambient_sky_contribution: f32,
    reflection_source: ReflectionSource,
    // Tonemap.
    tone_mapper: ToneMapper,
    tonemap_exposure: f32,
    tonemap_white: f32,
    tonemap_agx_white: f32,
    tonemap_agx_contrast: f32,
    // SSR.
    ssr_enabled: bool,
    ssr_max_steps: i32,
    ssr_fade_in: f32,
    ssr_fade_out: f32,
    ssr_depth_tolerance: f32,
    // SSAO.
    ssao_enabled: bool,
    ssao_radius: f32,
    ssao_intensity: f32,
    ssao_power: f32,
    ssao_detail: f32,
    ssao_horizon: f32,
    ssao_sharpness: f32,
    ssao_direct_light_affect: f32,
    ssao_ao_channel_affect: f32,
    // SSIL.
    ssil_enabled: bool,
    ssil_radius: f32,
    ssil_intensity: f32,
    ssil_sharpness: f32,
    ssil_normal_rejection: f32,
    // SDFGI.
    sdfgi_enabled: bool,
    sdfgi_cascades: i32,
    sdfgi_min_cell_size: f32,
    sdfgi_y_scale: SdfgiYScale,
    sdfgi_use_occlusion: bool,
    sdfgi_bounce_feedback: f32,
    sdfgi_read_sky_light: bool,
    sdfgi_energy: f32,
    sdfgi_normal_bias: f32,
    sdfgi_probe_bias: f32,
    // Glow.
    glow_enabled: bool,
    glow_levels: [f32; MAX_GLOW_LEVELS],
    glow_normalize_levels: bool,
    glow_intensity: f32,
    glow_strength: f32,
    glow_mix: f32,
    glow_bloom: f32,
    glow_blend_mode: GlowBlendMode,
    glow_hdr_bleed_threshold: f32,
    glow_hdr_bleed_scale: f32,
    glow_hdr_luminance_cap: f32,
    glow_map_strength: f32,
    glow_map: Option<Rid>,
    // Fog.
    fog_enabled: bool,
    fog_mode: FogMode,
    fog_light_color: Color,
    fog_light_energy: f32,
    fog_sun_scatter: f32,
    fog_density: f32,
    fog_height: f32,
    fog_height_density: f32,
    fog_aerial_perspective: f32,
    fog_sky_affect: f32,
    fog_depth_curve: f32,
    fog_depth_begin: f32,
    fog_depth_end: f32,
    // Volumetric fog.
    volumetric_fog_enabled: bool,
    volumetric_fog_density: f32,
    volumetric_fog_albedo: Color,
    volumetric_fog_emission: Color,
    volumetric_fog_emission_energy: f32,
    volumetric_fog_anisotropy: f32,
    volumetric_fog_length: f32,
    volumetric_fog_detail_spread: f32,
    volumetric_fog_gi_inject: f32,
    volumetric_fog_ambient_inject: f32,
    volumetric_fog_sky_affect: f32,
    volumetric_fog_temporal_reproject: bool,
    volumetric_fog_temporal_reproject_amount: f32,
    // Adjustment.
    adjustment_enabled: bool,
    adjustment_brightness: f32,
    adjustment_contrast: f32,
    adjustment_saturation: f32,
    use_1d_color_correction: bool,
    adjustment_color_correction: Option<Rid>,
}

impl Default for EnvironmentData {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! accessors {
    ($field:ident, $set:ident, $get:ident, $ty:ty) => {
        pub fn $set(&mut self, value: $ty) {
            self.$field = value;
        }

        pub fn $get(&self) -> $ty {
            self.$field
        }
    };
}

impl EnvironmentData {
    pub fn new() -> Self {
        Self {
            bg_mode: BgMode::ClearColor,
            bg_sky: None,
            bg_sky_custom_fov: 0.0,
            bg_sky_rotation: Vector3::ZERO,
            bg_color: Color::new(0.0, 0.0, 0.0, 1.0),
            bg_canvas_max_layer: 0,
            bg_camera_feed_id: 1,
            bg_energy_multiplier: 1.0,
            bg_intensity: 30000.0,
            ambient_color: Color::new(0.0, 0.0, 0.0, 1.0),
            ambient_source: AmbientSource::Bg,
            ambient_energy: 1.0,
            ambient_sky_contribution: 1.0,
            reflection_source: ReflectionSource::Bg,
            tone_mapper: ToneMapper::Linear,
            tonemap_exposure: 1.0,
            tonemap_white: 1.0,
            tonemap_agx_white: 16.29,
            tonemap_agx_contrast: 1.25,
            ssr_enabled: false,
            ssr_max_steps: 64,
            ssr_fade_in: 0.15,
            ssr_fade_out: 2.0,
            ssr_depth_tolerance: 0.5,
            ssao_enabled: false,
            ssao_radius: 1.0,
            ssao_intensity: 2.0,
            ssao_power: 1.5,
            ssao_detail: 0.5,
            ssao_horizon: 0.06,
            ssao_sharpness: 0.98,
            ssao_direct_light_affect: 0.0,
            ssao_ao_channel_affect: 0.0,
            ssil_enabled: false,
            ssil_radius: 5.0,
            ssil_intensity: 1.0,
            ssil_sharpness: 0.98,
            ssil_normal_rejection: 1.0,
            sdfgi_enabled: false,
            sdfgi_cascades: 4,
            sdfgi_min_cell_size: 0.2,
            sdfgi_y_scale: SdfgiYScale::Scale75,
            sdfgi_use_occlusion: false,
            sdfgi_bounce_feedback: 0.5,
            sdfgi_read_sky_light: true,
            sdfgi_energy: 1.0,
            sdfgi_normal_bias: 1.1,
            sdfgi_probe_bias: 1.1,
            glow_enabled: false,
            glow_levels: [0.0, 0.8, 0.4, 0.1, 0.0, 0.0, 0.0],
            glow_normalize_levels: false,
            glow_intensity: 0.3,
            glow_strength: 1.0,
            glow_mix: 0.05,
            glow_bloom: 0.0,
            glow_blend_mode: GlowBlendMode::Screen,
            glow_hdr_bleed_threshold: 1.0,
            glow_hdr_bleed_scale: 2.0,
            glow_hdr_luminance_cap: 12.0,
            glow_map_strength: 0.8,
            glow_map: None,
            fog_enabled: false,
            fog_mode: FogMode::Exponential,
            fog_light_color: Color::new(0.518, 0.553, 0.608, 1.0),
            fog_light_energy: 1.0,
            fog_sun_scatter: 0.0,
            fog_density: 0.01,
            fog_height: 0.0,
            fog_height_density: 0.0,
            fog_aerial_perspective: 0.0,
            fog_sky_affect: 1.0,
            fog_depth_curve: 1.0,
            fog_depth_begin: 10.0,
            fog_depth_end: 100.0,
            volumetric_fog_enabled: false,
            volumetric_fog_density: 0.05,
            volumetric_fog_albedo: Color::new(1.0, 1.0, 1.0, 1.0),
            volumetric_fog_emission: Color::new(0.0, 0.0, 0.0, 1.0),
            volumetric_fog_emission_energy: 1.0,
            volumetric_fog_anisotropy: 0.2,
            volumetric_fog_length: 64.0,
            volumetric_fog_detail_spread: 2.0,
            volumetric_fog_gi_inject: 1.0,
            volumetric_fog_ambient_inject: 0.0,
            volumetric_fog_sky_affect: 1.0,
            volumetric_fog_temporal_reproject: true,
            volumetric_fog_temporal_reproject_amount: 0.9,
            adjustment_enabled: false,
            adjustment_brightness: 1.0,
            adjustment_contrast: 1.0,
            adjustment_saturation: 1.0,
            use_1d_color_correction: true,
            adjustment_color_correction: None,
        }
    }

    // -- background ---------------------------------------------------------

    pub fn set_background_mode(&mut self, mode: BgMode) {
        self.bg_mode = mode;
    }

    pub fn get_background_mode(&self) -> BgMode {
        self.bg_mode
    }

    pub fn set_sky(&mut self, sky: Option<Rid>) {
        self.bg_sky = sky;
    }

    pub fn get_sky(&self) -> Option<Rid> {
        self.bg_sky
    }

    accessors!(bg_sky_custom_fov, set_sky_custom_fov, get_sky_custom_fov, f32);
    accessors!(
        bg_sky_rotation,
        set_sky_rotation,
        get_sky_rotation,
        Vector3
    );
    accessors!(bg_color, set_bg_color, get_bg_color, Color);
    accessors!(
        bg_canvas_max_layer,
        set_canvas_max_layer,
        get_canvas_max_layer,
        i32
    );
    accessors!(
        bg_camera_feed_id,
        set_camera_feed_id,
        get_camera_feed_id,
        i32
    );
    accessors!(
        bg_energy_multiplier,
        set_bg_energy_multiplier,
        get_bg_energy_multiplier,
        f32
    );
    accessors!(bg_intensity, set_bg_intensity, get_bg_intensity, f32);

    // -- ambient ------------------------------------------------------------

    accessors!(ambient_color, set_ambient_color, get_ambient_color, Color);
    accessors!(
        ambient_source,
        set_ambient_source,
        get_ambient_source,
        AmbientSource
    );
    accessors!(ambient_energy, set_ambient_energy, get_ambient_energy, f32);
    accessors!(
        ambient_sky_contribution,
        set_ambient_sky_contribution,
        get_ambient_sky_contribution,
        f32
    );
    accessors!(
        reflection_source,
        set_reflection_source,
        get_reflection_source,
        ReflectionSource
    );

    // -- tonemap ------------------------------------------------------------

    accessors!(tone_mapper, set_tone_mapper, get_tone_mapper, ToneMapper);
    accessors!(
        tonemap_exposure,
        set_tonemap_exposure,
        get_tonemap_exposure,
        f32
    );
    accessors!(tonemap_white, set_tonemap_white, get_tonemap_white, f32);
    accessors!(
        tonemap_agx_white,
        set_tonemap_agx_white,
        get_tonemap_agx_white,
        f32
    );
    accessors!(
        tonemap_agx_contrast,
        set_tonemap_agx_contrast,
        get_tonemap_agx_contrast,
        f32
    );

    // -- SSR ----------------------------------------------------------------

    accessors!(ssr_enabled, set_ssr_enabled, is_ssr_enabled, bool);
    accessors!(ssr_max_steps, set_ssr_max_steps, get_ssr_max_steps, i32);
    accessors!(ssr_fade_in, set_ssr_fade_in, get_ssr_fade_in, f32);
    accessors!(ssr_fade_out, set_ssr_fade_out, get_ssr_fade_out, f32);
    accessors!(
        ssr_depth_tolerance,
        set_ssr_depth_tolerance,
        get_ssr_depth_tolerance,
        f32
    );

    // -- SSAO ---------------------------------------------------------------

    accessors!(ssao_enabled, set_ssao_enabled, is_ssao_enabled, bool);
    accessors!(ssao_radius, set_ssao_radius, get_ssao_radius, f32);
    accessors!(ssao_intensity, set_ssao_intensity, get_ssao_intensity, f32);
    accessors!(ssao_power, set_ssao_power, get_ssao_power, f32);
    accessors!(ssao_detail, set_ssao_detail, get_ssao_detail, f32);
    accessors!(ssao_horizon, set_ssao_horizon, get_ssao_horizon, f32);
    accessors!(ssao_sharpness, set_ssao_sharpness, get_ssao_sharpness, f32);
    accessors!(
        ssao_direct_light_affect,
        set_ssao_direct_light_affect,
        get_ssao_direct_light_affect,
        f32
    );
    accessors!(
        ssao_ao_channel_affect,
        set_ssao_ao_channel_affect,
        get_ssao_ao_channel_affect,
        f32
    );

    // -- SSIL ---------------------------------------------------------------

    accessors!(ssil_enabled, set_ssil_enabled, is_ssil_enabled, bool);
    accessors!(ssil_radius, set_ssil_radius, get_ssil_radius, f32);
    accessors!(ssil_intensity, set_ssil_intensity, get_ssil_intensity, f32);
    accessors!(ssil_sharpness, set_ssil_sharpness, get_ssil_sharpness, f32);
    accessors!(
        ssil_normal_rejection,
        set_ssil_normal_rejection,
        get_ssil_normal_rejection,
        f32
    );

    // -- SDFGI --------------------------------------------------------------

    accessors!(sdfgi_enabled, set_sdfgi_enabled, is_sdfgi_enabled, bool);

    /// `ERR_FAIL_COND_MSG(cascades < 1 || cascades > 8, ...)`.
    pub fn set_sdfgi_cascades(&mut self, cascades: i32) -> bool {
        if !(1..=8).contains(&cascades) {
            return false;
        }
        self.sdfgi_cascades = cascades;
        true
    }

    pub fn get_sdfgi_cascades(&self) -> i32 {
        self.sdfgi_cascades
    }

    accessors!(
        sdfgi_min_cell_size,
        set_sdfgi_min_cell_size,
        get_sdfgi_min_cell_size,
        f32
    );

    /// `min_cell_size * 64`.
    pub fn get_sdfgi_cascade0_distance(&self) -> f32 {
        self.sdfgi_min_cell_size * 64.0
    }

    /// `cascade0 * 2^cascades`.
    pub fn get_sdfgi_max_distance(&self) -> f32 {
        let mut distance = self.sdfgi_min_cell_size * 64.0;
        for _ in 0..self.sdfgi_cascades {
            distance *= 2.0;
        }
        distance
    }

    accessors!(
        sdfgi_y_scale,
        set_sdfgi_y_scale,
        get_sdfgi_y_scale,
        SdfgiYScale
    );
    accessors!(
        sdfgi_use_occlusion,
        set_sdfgi_use_occlusion,
        is_sdfgi_using_occlusion,
        bool
    );
    accessors!(
        sdfgi_bounce_feedback,
        set_sdfgi_bounce_feedback,
        get_sdfgi_bounce_feedback,
        f32
    );
    accessors!(
        sdfgi_read_sky_light,
        set_sdfgi_read_sky_light,
        is_sdfgi_reading_sky_light,
        bool
    );
    accessors!(sdfgi_energy, set_sdfgi_energy, get_sdfgi_energy, f32);
    accessors!(
        sdfgi_normal_bias,
        set_sdfgi_normal_bias,
        get_sdfgi_normal_bias,
        f32
    );
    accessors!(
        sdfgi_probe_bias,
        set_sdfgi_probe_bias,
        get_sdfgi_probe_bias,
        f32
    );

    // -- glow ---------------------------------------------------------------

    accessors!(glow_enabled, set_glow_enabled, is_glow_enabled, bool);

    /// `ERR_FAIL_INDEX(level, MAX_GLOW_LEVELS)`.
    pub fn set_glow_level(&mut self, level: usize, intensity: f32) -> bool {
        if level >= MAX_GLOW_LEVELS {
            return false;
        }
        self.glow_levels[level] = intensity;
        true
    }

    /// Invalid levels read as 0 (`ERR_FAIL_INDEX_V`).
    pub fn get_glow_level(&self, level: usize) -> f32 {
        self.glow_levels.get(level).copied().unwrap_or(0.0)
    }

    accessors!(
        glow_normalize_levels,
        set_glow_normalized,
        is_glow_normalized,
        bool
    );
    accessors!(glow_intensity, set_glow_intensity, get_glow_intensity, f32);
    accessors!(glow_strength, set_glow_strength, get_glow_strength, f32);
    accessors!(glow_mix, set_glow_mix, get_glow_mix, f32);
    accessors!(glow_bloom, set_glow_bloom, get_glow_bloom, f32);
    accessors!(
        glow_blend_mode,
        set_glow_blend_mode,
        get_glow_blend_mode,
        GlowBlendMode
    );
    accessors!(
        glow_hdr_bleed_threshold,
        set_glow_hdr_bleed_threshold,
        get_glow_hdr_bleed_threshold,
        f32
    );
    accessors!(
        glow_hdr_bleed_scale,
        set_glow_hdr_bleed_scale,
        get_glow_hdr_bleed_scale,
        f32
    );
    accessors!(
        glow_hdr_luminance_cap,
        set_glow_hdr_luminance_cap,
        get_glow_hdr_luminance_cap,
        f32
    );
    accessors!(
        glow_map_strength,
        set_glow_map_strength,
        get_glow_map_strength,
        f32
    );
    accessors!(glow_map, set_glow_map, get_glow_map, Option<Rid>);

    // -- fog ----------------------------------------------------------------

    accessors!(fog_enabled, set_fog_enabled, is_fog_enabled, bool);
    accessors!(fog_mode, set_fog_mode, get_fog_mode, FogMode);
    accessors!(
        fog_light_color,
        set_fog_light_color,
        get_fog_light_color,
        Color
    );
    accessors!(
        fog_light_energy,
        set_fog_light_energy,
        get_fog_light_energy,
        f32
    );
    accessors!(fog_sun_scatter, set_fog_sun_scatter, get_fog_sun_scatter, f32);
    accessors!(fog_density, set_fog_density, get_fog_density, f32);
    accessors!(fog_height, set_fog_height, get_fog_height, f32);
    accessors!(
        fog_height_density,
        set_fog_height_density,
        get_fog_height_density,
        f32
    );
    accessors!(
        fog_aerial_perspective,
        set_fog_aerial_perspective,
        get_fog_aerial_perspective,
        f32
    );
    accessors!(fog_sky_affect, set_fog_sky_affect, get_fog_sky_affect, f32);
    accessors!(
        fog_depth_curve,
        set_fog_depth_curve,
        get_fog_depth_curve,
        f32
    );
    accessors!(
        fog_depth_begin,
        set_fog_depth_begin,
        get_fog_depth_begin,
        f32
    );
    accessors!(fog_depth_end, set_fog_depth_end, get_fog_depth_end, f32);

    // -- volumetric fog -----------------------------------------------------

    accessors!(
        volumetric_fog_enabled,
        set_volumetric_fog_enabled,
        is_volumetric_fog_enabled,
        bool
    );
    accessors!(
        volumetric_fog_density,
        set_volumetric_fog_density,
        get_volumetric_fog_density,
        f32
    );
    accessors!(
        volumetric_fog_albedo,
        set_volumetric_fog_albedo,
        get_volumetric_fog_albedo,
        Color
    );
    accessors!(
        volumetric_fog_emission,
        set_volumetric_fog_emission,
        get_volumetric_fog_emission,
        Color
    );
    accessors!(
        volumetric_fog_emission_energy,
        set_volumetric_fog_emission_energy,
        get_volumetric_fog_emission_energy,
        f32
    );
    accessors!(
        volumetric_fog_anisotropy,
        set_volumetric_fog_anisotropy,
        get_volumetric_fog_anisotropy,
        f32
    );
    accessors!(
        volumetric_fog_length,
        set_volumetric_fog_length,
        get_volumetric_fog_length,
        f32
    );
    accessors!(
        volumetric_fog_detail_spread,
        set_volumetric_fog_detail_spread,
        get_volumetric_fog_detail_spread,
        f32
    );
    accessors!(
        volumetric_fog_gi_inject,
        set_volumetric_fog_gi_inject,
        get_volumetric_fog_gi_inject,
        f32
    );
    accessors!(
        volumetric_fog_ambient_inject,
        set_volumetric_fog_ambient_inject,
        get_volumetric_fog_ambient_inject,
        f32
    );
    accessors!(
        volumetric_fog_sky_affect,
        set_volumetric_fog_sky_affect,
        get_volumetric_fog_sky_affect,
        f32
    );
    accessors!(
        volumetric_fog_temporal_reproject,
        set_volumetric_fog_temporal_reprojection_enabled,
        is_volumetric_fog_temporal_reprojection_enabled,
        bool
    );
    accessors!(
        volumetric_fog_temporal_reproject_amount,
        set_volumetric_fog_temporal_reprojection_amount,
        get_volumetric_fog_temporal_reprojection_amount,
        f32
    );

    // -- adjustment ---------------------------------------------------------

    accessors!(
        adjustment_enabled,
        set_adjustment_enabled,
        is_adjustment_enabled,
        bool
    );
    accessors!(
        adjustment_brightness,
        set_adjustment_brightness,
        get_adjustment_brightness,
        f32
    );
    accessors!(
        adjustment_contrast,
        set_adjustment_contrast,
        get_adjustment_contrast,
        f32
    );
    accessors!(
        adjustment_saturation,
        set_adjustment_saturation,
        get_adjustment_saturation,
        f32
    );
    accessors!(
        use_1d_color_correction,
        set_use_1d_color_correction,
        is_using_1d_color_correction,
        bool
    );
    accessors!(
        adjustment_color_correction,
        set_adjustment_color_correction,
        get_adjustment_color_correction,
        Option<Rid>
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_and_ambient_defaults() {
        let e = EnvironmentData::new();
        assert_eq!(e.get_background_mode(), BgMode::ClearColor);
        assert!(e.get_sky().is_none());
        assert_eq!(e.get_sky_custom_fov(), 0.0);
        assert_eq!(e.get_sky_rotation(), Vector3::ZERO);
        assert_eq!(e.get_bg_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(e.get_canvas_max_layer(), 0);
        assert_eq!(e.get_camera_feed_id(), 1);
        assert_eq!(e.get_bg_energy_multiplier(), 1.0);
        assert_eq!(e.get_bg_intensity(), 30000.0);

        assert_eq!(e.get_ambient_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(e.get_ambient_source(), AmbientSource::Bg);
        assert_eq!(e.get_ambient_energy(), 1.0);
        assert_eq!(e.get_ambient_sky_contribution(), 1.0);
        assert_eq!(e.get_reflection_source(), ReflectionSource::Bg);
    }

    #[test]
    fn tonemap_ssr_ssao_ssil_defaults() {
        let e = EnvironmentData::new();
        assert_eq!(e.get_tone_mapper(), ToneMapper::Linear);
        assert_eq!(e.get_tonemap_exposure(), 1.0);
        assert_eq!(e.get_tonemap_white(), 1.0);
        assert_eq!(e.get_tonemap_agx_white(), 16.29);
        assert_eq!(e.get_tonemap_agx_contrast(), 1.25);

        assert!(!e.is_ssr_enabled());
        assert_eq!(e.get_ssr_max_steps(), 64);
        assert_eq!(e.get_ssr_fade_in(), 0.15);
        assert_eq!(e.get_ssr_fade_out(), 2.0);
        assert_eq!(e.get_ssr_depth_tolerance(), 0.5);

        assert!(!e.is_ssao_enabled());
        assert_eq!(e.get_ssao_radius(), 1.0);
        assert_eq!(e.get_ssao_intensity(), 2.0);
        assert_eq!(e.get_ssao_power(), 1.5);
        assert_eq!(e.get_ssao_detail(), 0.5);
        assert_eq!(e.get_ssao_horizon(), 0.06);
        assert_eq!(e.get_ssao_sharpness(), 0.98);
        assert_eq!(e.get_ssao_direct_light_affect(), 0.0);
        assert_eq!(e.get_ssao_ao_channel_affect(), 0.0);

        assert!(!e.is_ssil_enabled());
        assert_eq!(e.get_ssil_radius(), 5.0);
        assert_eq!(e.get_ssil_intensity(), 1.0);
        assert_eq!(e.get_ssil_sharpness(), 0.98);
        assert_eq!(e.get_ssil_normal_rejection(), 1.0);
    }

    #[test]
    fn sdfgi_defaults_and_derived_distances() {
        let mut e = EnvironmentData::new();
        assert!(!e.is_sdfgi_enabled());
        assert_eq!(e.get_sdfgi_cascades(), 4);
        assert_eq!(e.get_sdfgi_min_cell_size(), 0.2);
        assert_eq!(e.get_sdfgi_y_scale(), SdfgiYScale::Scale75);
        assert!(!e.is_sdfgi_using_occlusion());
        assert_eq!(e.get_sdfgi_bounce_feedback(), 0.5);
        assert!(e.is_sdfgi_reading_sky_light());
        assert_eq!(e.get_sdfgi_energy(), 1.0);
        assert_eq!(e.get_sdfgi_normal_bias(), 1.1);
        assert_eq!(e.get_sdfgi_probe_bias(), 1.1);

        // 0.2 * 64 = 12.8, times 2^4 = 204.8.
        assert!((e.get_sdfgi_cascade0_distance() - 12.8).abs() < 1e-4);
        assert!((e.get_sdfgi_max_distance() - 204.8).abs() < 1e-3);

        // Cascades are validated into 1..=8.
        assert!(!e.set_sdfgi_cascades(0));
        assert!(!e.set_sdfgi_cascades(9));
        assert_eq!(e.get_sdfgi_cascades(), 4);
        assert!(e.set_sdfgi_cascades(1));
        assert!((e.get_sdfgi_max_distance() - 25.6).abs() < 1e-4);
        assert!(e.set_sdfgi_cascades(8));
        assert!((e.get_sdfgi_max_distance() - 12.8 * 256.0).abs() < 1e-2);
    }

    #[test]
    fn glow_defaults_and_levels() {
        let mut e = EnvironmentData::new();
        assert!(!e.is_glow_enabled());
        assert_eq!(
            [0, 1, 2, 3, 4, 5, 6].map(|i| e.get_glow_level(i)),
            [0.0, 0.8, 0.4, 0.1, 0.0, 0.0, 0.0]
        );
        assert!(!e.is_glow_normalized());
        assert_eq!(e.get_glow_intensity(), 0.3);
        assert_eq!(e.get_glow_strength(), 1.0);
        assert_eq!(e.get_glow_mix(), 0.05);
        assert_eq!(e.get_glow_bloom(), 0.0);
        assert_eq!(e.get_glow_blend_mode(), GlowBlendMode::Screen);
        assert_eq!(e.get_glow_hdr_bleed_threshold(), 1.0);
        assert_eq!(e.get_glow_hdr_bleed_scale(), 2.0);
        assert_eq!(e.get_glow_hdr_luminance_cap(), 12.0);
        assert_eq!(e.get_glow_map_strength(), 0.8);
        assert!(e.get_glow_map().is_none());

        assert!(e.set_glow_level(6, 0.6));
        assert_eq!(e.get_glow_level(6), 0.6);
        assert!(!e.set_glow_level(MAX_GLOW_LEVELS, 1.0));
        assert_eq!(e.get_glow_level(MAX_GLOW_LEVELS), 0.0);
    }

    #[test]
    fn fog_defaults() {
        let e = EnvironmentData::new();
        assert!(!e.is_fog_enabled());
        assert_eq!(e.get_fog_mode(), FogMode::Exponential);
        assert_eq!(e.get_fog_light_color(), Color::new(0.518, 0.553, 0.608, 1.0));
        assert_eq!(e.get_fog_light_energy(), 1.0);
        assert_eq!(e.get_fog_sun_scatter(), 0.0);
        assert_eq!(e.get_fog_density(), 0.01);
        assert_eq!(e.get_fog_height(), 0.0);
        assert_eq!(e.get_fog_height_density(), 0.0);
        assert_eq!(e.get_fog_aerial_perspective(), 0.0);
        assert_eq!(e.get_fog_sky_affect(), 1.0);
        assert_eq!(e.get_fog_depth_curve(), 1.0);
        assert_eq!(e.get_fog_depth_begin(), 10.0);
        assert_eq!(e.get_fog_depth_end(), 100.0);
    }

    #[test]
    fn volumetric_fog_and_adjustment_defaults() {
        let e = EnvironmentData::new();
        assert!(!e.is_volumetric_fog_enabled());
        assert_eq!(e.get_volumetric_fog_density(), 0.05);
        assert_eq!(
            e.get_volumetric_fog_albedo(),
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
        assert_eq!(
            e.get_volumetric_fog_emission(),
            Color::new(0.0, 0.0, 0.0, 1.0)
        );
        assert_eq!(e.get_volumetric_fog_emission_energy(), 1.0);
        assert_eq!(e.get_volumetric_fog_anisotropy(), 0.2);
        assert_eq!(e.get_volumetric_fog_length(), 64.0);
        assert_eq!(e.get_volumetric_fog_detail_spread(), 2.0);
        assert_eq!(e.get_volumetric_fog_gi_inject(), 1.0);
        assert_eq!(e.get_volumetric_fog_ambient_inject(), 0.0);
        assert_eq!(e.get_volumetric_fog_sky_affect(), 1.0);
        assert!(e.is_volumetric_fog_temporal_reprojection_enabled());
        assert_eq!(e.get_volumetric_fog_temporal_reprojection_amount(), 0.9);

        assert!(!e.is_adjustment_enabled());
        assert_eq!(e.get_adjustment_brightness(), 1.0);
        assert_eq!(e.get_adjustment_contrast(), 1.0);
        assert_eq!(e.get_adjustment_saturation(), 1.0);
        assert!(e.is_using_1d_color_correction());
        assert!(e.get_adjustment_color_correction().is_none());
    }

    #[test]
    fn setters_round_trip_and_enum_values() {
        let mut e = EnvironmentData::new();
        e.set_background_mode(BgMode::Sky);
        e.set_sky(Some(Rid::from_u64(1)));
        e.set_sky_custom_fov(70.0);
        e.set_sky_rotation(Vector3::new(0.0, 1.0, 0.0));
        e.set_bg_color(Color::new(0.1, 0.2, 0.3, 1.0));
        e.set_canvas_max_layer(3);
        e.set_camera_feed_id(2);
        e.set_bg_energy_multiplier(2.0);
        e.set_bg_intensity(1000.0);
        e.set_ambient_color(Color::new(1.0, 1.0, 1.0, 1.0));
        e.set_ambient_source(AmbientSource::Sky);
        e.set_ambient_energy(2.0);
        e.set_ambient_sky_contribution(0.5);
        e.set_reflection_source(ReflectionSource::Sky);
        e.set_tone_mapper(ToneMapper::Agx);
        e.set_tonemap_exposure(2.0);
        e.set_tonemap_white(6.0);
        e.set_tonemap_agx_white(8.0);
        e.set_tonemap_agx_contrast(1.0);
        e.set_ssr_enabled(true);
        e.set_ssr_max_steps(128);
        e.set_ssr_fade_in(0.5);
        e.set_ssr_fade_out(1.0);
        e.set_ssr_depth_tolerance(1.0);
        e.set_ssao_enabled(true);
        e.set_ssao_radius(2.0);
        e.set_ssao_intensity(3.0);
        e.set_ssao_power(2.0);
        e.set_ssao_detail(0.25);
        e.set_ssao_horizon(0.1);
        e.set_ssao_sharpness(0.9);
        e.set_ssao_direct_light_affect(0.5);
        e.set_ssao_ao_channel_affect(0.25);
        e.set_ssil_enabled(true);
        e.set_ssil_radius(10.0);
        e.set_ssil_intensity(2.0);
        e.set_ssil_sharpness(0.5);
        e.set_ssil_normal_rejection(0.75);
        e.set_sdfgi_enabled(true);
        e.set_sdfgi_min_cell_size(0.5);
        e.set_sdfgi_y_scale(SdfgiYScale::Scale100);
        e.set_sdfgi_use_occlusion(true);
        e.set_sdfgi_bounce_feedback(1.0);
        e.set_sdfgi_read_sky_light(false);
        e.set_sdfgi_energy(2.0);
        e.set_sdfgi_normal_bias(0.5);
        e.set_sdfgi_probe_bias(0.25);
        e.set_glow_enabled(true);
        e.set_glow_normalized(true);
        e.set_glow_intensity(1.0);
        e.set_glow_strength(2.0);
        e.set_glow_mix(0.1);
        e.set_glow_bloom(0.25);
        e.set_glow_blend_mode(GlowBlendMode::Additive);
        e.set_glow_hdr_bleed_threshold(2.0);
        e.set_glow_hdr_bleed_scale(3.0);
        e.set_glow_hdr_luminance_cap(20.0);
        e.set_glow_map_strength(0.5);
        e.set_glow_map(Some(Rid::from_u64(2)));
        e.set_fog_enabled(true);
        e.set_fog_mode(FogMode::Depth);
        e.set_fog_light_color(Color::new(0.5, 0.5, 0.5, 1.0));
        e.set_fog_light_energy(2.0);
        e.set_fog_sun_scatter(0.5);
        e.set_fog_density(0.02);
        e.set_fog_height(2.0);
        e.set_fog_height_density(-0.5);
        e.set_fog_aerial_perspective(0.25);
        e.set_fog_sky_affect(0.75);
        e.set_fog_depth_curve(2.0);
        e.set_fog_depth_begin(5.0);
        e.set_fog_depth_end(50.0);
        e.set_volumetric_fog_enabled(true);
        e.set_volumetric_fog_density(0.1);
        e.set_volumetric_fog_albedo(Color::new(0.9, 0.9, 0.9, 1.0));
        e.set_volumetric_fog_emission(Color::new(0.1, 0.0, 0.0, 1.0));
        e.set_volumetric_fog_emission_energy(2.0);
        e.set_volumetric_fog_anisotropy(-0.5);
        e.set_volumetric_fog_length(32.0);
        e.set_volumetric_fog_detail_spread(1.0);
        e.set_volumetric_fog_gi_inject(0.5);
        e.set_volumetric_fog_ambient_inject(0.25);
        e.set_volumetric_fog_sky_affect(0.5);
        e.set_volumetric_fog_temporal_reprojection_enabled(false);
        e.set_volumetric_fog_temporal_reprojection_amount(0.5);
        e.set_adjustment_enabled(true);
        e.set_adjustment_brightness(1.5);
        e.set_adjustment_contrast(0.75);
        e.set_adjustment_saturation(1.25);
        e.set_use_1d_color_correction(false);
        e.set_adjustment_color_correction(Some(Rid::from_u64(3)));

        assert_eq!(e.get_background_mode(), BgMode::Sky);
        assert_eq!(e.get_sky(), Some(Rid::from_u64(1)));
        assert_eq!(e.get_sky_custom_fov(), 70.0);
        assert_eq!(e.get_sky_rotation(), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(e.get_canvas_max_layer(), 3);
        assert_eq!(e.get_camera_feed_id(), 2);
        assert_eq!(e.get_ambient_source(), AmbientSource::Sky);
        assert_eq!(e.get_reflection_source(), ReflectionSource::Sky);
        assert_eq!(e.get_tone_mapper(), ToneMapper::Agx);
        assert!(e.is_ssr_enabled());
        assert_eq!(e.get_ssr_max_steps(), 128);
        assert!(e.is_ssao_enabled());
        assert_eq!(e.get_ssao_direct_light_affect(), 0.5);
        assert!(e.is_ssil_enabled());
        assert!(e.is_sdfgi_enabled());
        assert_eq!(e.get_sdfgi_y_scale(), SdfgiYScale::Scale100);
        assert!(!e.is_sdfgi_reading_sky_light());
        assert!(e.is_glow_enabled());
        assert_eq!(e.get_glow_blend_mode(), GlowBlendMode::Additive);
        assert_eq!(e.get_glow_map(), Some(Rid::from_u64(2)));
        assert_eq!(e.get_fog_mode(), FogMode::Depth);
        assert_eq!(e.get_fog_height_density(), -0.5);
        assert!(e.is_volumetric_fog_enabled());
        assert!(!e.is_volumetric_fog_temporal_reprojection_enabled());
        assert!(e.is_adjustment_enabled());
        assert!(!e.is_using_1d_color_correction());
        assert_eq!(e.get_adjustment_color_correction(), Some(Rid::from_u64(3)));

        assert_eq!(BgMode::ClearColor as i32, 0);
        assert_eq!(BgMode::CameraFeed as i32, 5);
        assert_eq!(BgMode::Max as i32, 6);
        assert_eq!(AmbientSource::Bg as i32, 0);
        assert_eq!(AmbientSource::Sky as i32, 3);
        assert_eq!(ReflectionSource::Bg as i32, 0);
        assert_eq!(ReflectionSource::Sky as i32, 2);
        assert_eq!(ToneMapper::Linear as i32, 0);
        assert_eq!(ToneMapper::Agx as i32, 4);
        assert_eq!(SdfgiYScale::Scale50 as i32, 0);
        assert_eq!(SdfgiYScale::Scale100 as i32, 2);
        assert_eq!(FogMode::Exponential as i32, 0);
        assert_eq!(FogMode::Depth as i32, 1);
        assert_eq!(GlowBlendMode::Additive as i32, 0);
        assert_eq!(GlowBlendMode::Mix as i32, 4);
    }
}
