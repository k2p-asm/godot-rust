//! `servers/rendering/rendering_server_enums.h`: the numeric contract.
//!
//! This is the first `servers/` unit: the enum values the engine hands to the
//! renderer. They are part of the wire format between scenes and the server, so
//! their numbers matter as much as their names. Values below are copied from
//! the header and asserted in the tests.

/// `CanvasItemZMin` / `CanvasItemZMax`.
pub const CANVAS_ITEM_Z_MIN: i32 = -4096;
pub const CANVAS_ITEM_Z_MAX: i32 = 4096;
/// `CanvasLayerMin` / `CanvasLayerMax`.
pub const CANVAS_LAYER_MIN: i32 = i32::MIN;
pub const CANVAS_LAYER_MAX: i32 = i32::MAX;

pub const MAX_GLOW_LEVELS: i32 = 7;
pub const MAX_CURSORS: i32 = 8;
pub const MAX_2D_DIRECTIONAL_LIGHTS: i32 = 8;
pub const MAX_MESH_SURFACES: i32 = 256;

/// `CanvasItemTextureFilter`, in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CanvasItemTextureFilter {
    Default,
    Nearest,
    Linear,
    NearestWithMipmaps,
    LinearWithMipmaps,
    NearestWithMipmapsAnisotropic,
    LinearWithMipmapsAnisotropic,
    Max,
}

impl CanvasItemTextureFilter {
    pub const COUNT: i32 = 7;

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// `CanvasItemTextureRepeat`, in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CanvasItemTextureRepeat {
    Default,
    Disabled,
    Enabled,
    Mirror,
    Max,
}

impl CanvasItemTextureRepeat {
    pub const COUNT: i32 = 4;

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// `CanvasLightBlendMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CanvasLightBlendMode {
    Add,
    Sub,
    Mix,
}

/// `CanvasLightShadowFilter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CanvasLightShadowFilter {
    None,
    Pcf5,
    Pcf13,
    Max,
}

impl CanvasLightShadowFilter {
    pub const COUNT: i32 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_limits_match_the_header() {
        assert_eq!(CANVAS_ITEM_Z_MIN, -4096);
        assert_eq!(CANVAS_ITEM_Z_MAX, 4096);
        assert_eq!(CANVAS_LAYER_MIN, i32::MIN);
        assert_eq!(CANVAS_LAYER_MAX, i32::MAX);
        assert_eq!(MAX_GLOW_LEVELS, 7);
        assert_eq!(MAX_CURSORS, 8);
        assert_eq!(MAX_2D_DIRECTIONAL_LIGHTS, 8);
        assert_eq!(MAX_MESH_SURFACES, 256);
    }

    #[test]
    fn texture_filter_values_follow_declaration_order() {
        assert_eq!(CanvasItemTextureFilter::Default.as_i32(), 0);
        assert_eq!(CanvasItemTextureFilter::Nearest.as_i32(), 1);
        assert_eq!(CanvasItemTextureFilter::Linear.as_i32(), 2);
        assert_eq!(CanvasItemTextureFilter::NearestWithMipmaps.as_i32(), 3);
        assert_eq!(CanvasItemTextureFilter::LinearWithMipmaps.as_i32(), 4);
        assert_eq!(CanvasItemTextureFilter::NearestWithMipmapsAnisotropic.as_i32(), 5);
        assert_eq!(CanvasItemTextureFilter::LinearWithMipmapsAnisotropic.as_i32(), 6);
        // MAX is the count, and the enum's last member.
        assert_eq!(CanvasItemTextureFilter::Max.as_i32(), CanvasItemTextureFilter::COUNT);
    }

    #[test]
    fn texture_repeat_values_follow_declaration_order() {
        assert_eq!(CanvasItemTextureRepeat::Default.as_i32(), 0);
        assert_eq!(CanvasItemTextureRepeat::Disabled.as_i32(), 1);
        assert_eq!(CanvasItemTextureRepeat::Enabled.as_i32(), 2);
        assert_eq!(CanvasItemTextureRepeat::Mirror.as_i32(), 3);
        assert_eq!(CanvasItemTextureRepeat::Max.as_i32(), CanvasItemTextureRepeat::COUNT);
    }

    #[test]
    fn light_enums_match_the_header() {
        assert_eq!(CanvasLightBlendMode::Add as i32, 0);
        assert_eq!(CanvasLightBlendMode::Sub as i32, 1);
        assert_eq!(CanvasLightBlendMode::Mix as i32, 2);

        assert_eq!(CanvasLightShadowFilter::None as i32, 0);
        assert_eq!(CanvasLightShadowFilter::Pcf5 as i32, 1);
        assert_eq!(CanvasLightShadowFilter::Pcf13 as i32, 2);
        assert_eq!(CanvasLightShadowFilter::Max as i32, CanvasLightShadowFilter::COUNT);
    }

    #[test]
    fn canvas_item_limits_agree_with_the_scene_module() {
        // The scene layer duplicates these constants because it has no
        // dependency on servers; keep them in sync.
        assert_eq!(CANVAS_ITEM_Z_MIN, crate::scene::canvas_item::CANVAS_ITEM_Z_MIN);
        assert_eq!(CANVAS_ITEM_Z_MAX, crate::scene::canvas_item::CANVAS_ITEM_Z_MAX);
    }
}
