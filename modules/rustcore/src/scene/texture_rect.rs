//! `scene/gui/texture_rect.*`: the texture rectangle's state.
//!
//! Defaults: no texture, `EXPAND_KEEP_SIZE`, `STRETCH_SCALE`, no flipping.
//! The two enums keep the engine's numeric order.
//!
//! Minimum-size computation, drawing and the deprecated property path are not
//! ported.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExpandMode {
    KeepSize = 0,
    IgnoreSize = 1,
    FitWidth = 2,
    FitWidthProportional = 3,
    FitHeight = 4,
    FitHeightProportional = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StretchMode {
    Scale = 0,
    Tile = 1,
    Keep = 2,
    KeepCentered = 3,
    KeepAspect = 4,
    KeepAspectCentered = 5,
    KeepAspectCovered = 6,
}

#[derive(Clone, Debug)]
pub struct TextureRectData {
    texture: Option<Rid>,
    expand_mode: ExpandMode,
    stretch_mode: StretchMode,
    flip_h: bool,
    flip_v: bool,
}

impl Default for TextureRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureRectData {
    pub fn new() -> Self {
        Self {
            texture: None,
            expand_mode: ExpandMode::KeepSize,
            stretch_mode: StretchMode::Scale,
            flip_h: false,
            flip_v: false,
        }
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn set_expand_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => ExpandMode::KeepSize,
            1 => ExpandMode::IgnoreSize,
            2 => ExpandMode::FitWidth,
            3 => ExpandMode::FitWidthProportional,
            4 => ExpandMode::FitHeight,
            5 => ExpandMode::FitHeightProportional,
            _ => return false,
        };
        self.expand_mode = parsed;
        true
    }

    pub fn get_expand_mode(&self) -> ExpandMode {
        self.expand_mode
    }

    pub fn set_stretch_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => StretchMode::Scale,
            1 => StretchMode::Tile,
            2 => StretchMode::Keep,
            3 => StretchMode::KeepCentered,
            4 => StretchMode::KeepAspect,
            5 => StretchMode::KeepAspectCentered,
            6 => StretchMode::KeepAspectCovered,
            _ => return false,
        };
        self.stretch_mode = parsed;
        true
    }

    pub fn get_stretch_mode(&self) -> StretchMode {
        self.stretch_mode
    }

    pub fn set_flip_h(&mut self, flip: bool) {
        self.flip_h = flip;
    }

    pub fn is_flipped_h(&self) -> bool {
        self.flip_h
    }

    pub fn set_flip_v(&mut self, flip: bool) {
        self.flip_v = flip;
    }

    pub fn is_flipped_v(&self) -> bool {
        self.flip_v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut rect = TextureRectData::new();
        assert!(rect.get_texture().is_none());
        assert_eq!(rect.get_expand_mode(), ExpandMode::KeepSize);
        assert_eq!(rect.get_stretch_mode(), StretchMode::Scale);
        assert!(!rect.is_flipped_h());
        assert!(!rect.is_flipped_v());

        rect.set_texture(Some(Rid::from_u64(3)));
        assert!(rect.set_expand_mode(5));
        assert!(!rect.set_expand_mode(6));
        assert!(rect.set_stretch_mode(6));
        assert!(!rect.set_stretch_mode(7));
        rect.set_flip_h(true);
        rect.set_flip_v(true);

        assert_eq!(rect.get_texture(), Some(Rid::from_u64(3)));
        assert_eq!(rect.get_expand_mode(), ExpandMode::FitHeightProportional);
        assert_eq!(rect.get_stretch_mode(), StretchMode::KeepAspectCovered);
        assert!(rect.is_flipped_h());
        assert!(rect.is_flipped_v());
    }

    #[test]
    fn enum_values() {
        assert_eq!(ExpandMode::KeepSize as i32, 0);
        assert_eq!(ExpandMode::IgnoreSize as i32, 1);
        assert_eq!(ExpandMode::FitWidth as i32, 2);
        assert_eq!(ExpandMode::FitWidthProportional as i32, 3);
        assert_eq!(ExpandMode::FitHeight as i32, 4);
        assert_eq!(ExpandMode::FitHeightProportional as i32, 5);
        assert_eq!(StretchMode::Scale as i32, 0);
        assert_eq!(StretchMode::Tile as i32, 1);
        assert_eq!(StretchMode::Keep as i32, 2);
        assert_eq!(StretchMode::KeepCentered as i32, 3);
        assert_eq!(StretchMode::KeepAspect as i32, 4);
        assert_eq!(StretchMode::KeepAspectCentered as i32, 5);
        assert_eq!(StretchMode::KeepAspectCovered as i32, 6);
    }
}
