//! `scene/gui/texture_button.*`: texture set, stretch mode and the click mask.
//!
//! `TextureButton` extends `BaseButton` with five textures (normal, pressed,
//! hover, disabled, focused), a click-mask bitmap, `ignore_texture_size` and the
//! stretch mode (`KEEP` by default, seven values). The hit test is ported exactly:
//!
//! - with a mask and no drawn area the mask's own size is the reference rect;
//! - in tile mode the point is wrapped by `ceil(area / mask)` and `%`;
//! - otherwise the point is offset and scaled back into mask space, with the
//!   aspect-covered region correction;
//! - points outside the resulting rect are misses, and inside they are looked up
//!   in the bitmap.
//!
//! Drawing and the texture-size tracking are not ported, so the layout rects are
//! supplied through `set_layout`.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::scene::bit_map::BitMap;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StretchMode {
    Scale,
    Tile,
    Keep,
    KeepCentered,
    KeepAspect,
    KeepAspectCentered,
    KeepAspectCovered,
}

#[derive(Clone, Debug)]
pub struct TextureButtonData {
    normal: Option<Rid>,
    pressed: Option<Rid>,
    hover: Option<Rid>,
    disabled: Option<Rid>,
    focused: Option<Rid>,
    click_mask: Option<BitMap>,
    ignore_texture_size: bool,
    stretch_mode: StretchMode,
    texture_region: Rect2,
    position_rect: Rect2,
    tile: bool,
    hflip: bool,
    vflip: bool,
}

impl Default for TextureButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureButtonData {
    pub fn new() -> Self {
        Self {
            normal: None,
            pressed: None,
            hover: None,
            disabled: None,
            focused: None,
            click_mask: None,
            ignore_texture_size: false,
            stretch_mode: StretchMode::Keep,
            texture_region: Rect2::default(),
            position_rect: Rect2::default(),
            tile: false,
            hflip: false,
            vflip: false,
        }
    }

    pub fn set_texture_normal(&mut self, texture: Option<Rid>) {
        self.normal = texture;
    }

    pub fn get_texture_normal(&self) -> Option<Rid> {
        self.normal
    }

    pub fn set_texture_pressed(&mut self, texture: Option<Rid>) {
        self.pressed = texture;
    }

    pub fn get_texture_pressed(&self) -> Option<Rid> {
        self.pressed
    }

    pub fn set_texture_hover(&mut self, texture: Option<Rid>) {
        self.hover = texture;
    }

    pub fn get_texture_hover(&self) -> Option<Rid> {
        self.hover
    }

    pub fn set_texture_disabled(&mut self, texture: Option<Rid>) {
        self.disabled = texture;
    }

    pub fn get_texture_disabled(&self) -> Option<Rid> {
        self.disabled
    }

    pub fn set_texture_focused(&mut self, texture: Option<Rid>) {
        self.focused = texture;
    }

    pub fn get_texture_focused(&self) -> Option<Rid> {
        self.focused
    }

    pub fn set_click_mask(&mut self, mask: Option<BitMap>) {
        self.click_mask = mask;
    }

    pub fn get_click_mask(&self) -> Option<&BitMap> {
        self.click_mask.as_ref()
    }

    pub fn set_ignore_texture_size(&mut self, ignore: bool) {
        self.ignore_texture_size = ignore;
    }

    pub fn get_ignore_texture_size(&self) -> bool {
        self.ignore_texture_size
    }

    pub fn set_stretch_mode(&mut self, mode: StretchMode) {
        self.stretch_mode = mode;
    }

    pub fn get_stretch_mode(&self) -> StretchMode {
        self.stretch_mode
    }

    pub fn set_flip_h(&mut self, flip: bool) {
        self.hflip = flip;
    }

    pub fn is_flipped_h(&self) -> bool {
        self.hflip
    }

    pub fn set_flip_v(&mut self, flip: bool) {
        self.vflip = flip;
    }

    pub fn is_flipped_v(&self) -> bool {
        self.vflip
    }

    /// The rects `_draw` computes: where the texture landed and which part of it
    /// is used (`_texture_region`).
    pub fn set_layout(&mut self, position_rect: Rect2, texture_region: Rect2, tile: bool) {
        self.position_rect = position_rect;
        self.texture_region = texture_region;
        self.tile = tile;
    }

    pub fn get_position_rect(&self) -> Rect2 {
        self.position_rect
    }

    pub fn get_texture_region(&self) -> Rect2 {
        self.texture_region
    }

    /// `get_minimum_size`: the largest of the available textures unless
    /// `ignore_texture_size` is set. Texture sizes are arguments because the
    /// port stores textures as handles.
    pub fn get_minimum_size(
        &self,
        control_min: Vector2,
        normal_size: Option<Vector2>,
        pressed_size: Option<Vector2>,
        hover_size: Option<Vector2>,
        mask_size: Option<Vector2>,
    ) -> Vector2 {
        let mut size = control_min;
        if !self.ignore_texture_size {
            size = if let Some(normal) = normal_size {
                normal
            } else if let Some(pressed) = pressed_size {
                pressed
            } else if let Some(hover) = hover_size {
                hover
            } else if let Some(mask) = mask_size {
                mask
            } else {
                Vector2::ZERO
            };
        }
        Vector2::new(size.x.abs(), size.y.abs())
    }

    /// `has_point` for the click mask; without a mask this reports a hit (the
    /// caller then falls back to `Control::has_point`).
    pub fn has_point(&self, point: Vector2) -> Option<bool> {
        let Some(mask) = &self.click_mask else {
            return None;
        };
        let mask_size = {
            let s = mask.get_size();
            Vector2::new(s.x as f32, s.y as f32)
        };
        let mut point = point;
        let rect;

        if !self.position_rect.has_area() {
            rect = Rect2::new(Vector2::ZERO, mask_size);
        } else if self.tile {
            // Keep the point inside the mask by wrapping the tile indices.
            let mut r = Rect2::new(Vector2::ZERO, mask_size);
            if self.position_rect.has_point(point) {
                let cols = (self.position_rect.size.x / mask_size.x).ceil() as i32;
                let rows = (self.position_rect.size.y / mask_size.y).ceil() as i32;
                // The engine truncates the tile index before the modulo.
                let col = ((point.x / mask_size.x) as i32).rem_euclid(cols.max(1));
                let row = ((point.y / mask_size.y) as i32).rem_euclid(rows.max(1));
                point.x -= mask_size.x * col as f32;
                point.y -= mask_size.y * row as f32;
            }
            r.position = Vector2::ZERO;
            rect = r;
        } else {
            // Map the point from the drawn image back into mask space.
            let mut ofs = self.position_rect.position;
            let mut scale = Vector2::new(
                mask_size.x / self.position_rect.size.x,
                mask_size.y / self.position_rect.size.y,
            );
            if self.stretch_mode == StretchMode::KeepAspectCovered {
                let min = scale.x.min(scale.y);
                scale = Vector2::new(min, min);
                ofs -= self.texture_region.position / min;
            }
            point -= ofs;
            point *= scale;

            rect = Rect2::new(
                Vector2::new(
                    self.texture_region.position.x.max(0.0),
                    self.texture_region.position.y.max(0.0),
                ),
                Vector2::new(
                    mask_size.x.min(self.texture_region.size.x),
                    mask_size.y.min(self.texture_region.size.y),
                ),
            );
        }

        if !rect.has_point(point) {
            return Some(false);
        }
        let p = Vector2i::new(point.x as i32, point.y as i32);
        Some(mask.get_bit(p.x, p.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(width: i32, height: i32) -> BitMap {
        let mut b = BitMap::new();
        b.create(width, height);
        b
    }

    #[test]
    fn defaults() {
        let b = TextureButtonData::new();
        assert!(b.get_texture_normal().is_none());
        assert!(b.get_texture_pressed().is_none());
        assert!(b.get_texture_hover().is_none());
        assert!(b.get_texture_disabled().is_none());
        assert!(b.get_texture_focused().is_none());
        assert!(b.get_click_mask().is_none());
        assert!(!b.get_ignore_texture_size());
        assert_eq!(b.get_stretch_mode(), StretchMode::Keep);
        assert!(!b.is_flipped_h());
        assert!(!b.is_flipped_v());
    }

    #[test]
    fn minimum_size_prefers_normal_then_pressed_then_hover_then_mask() {
        let mut b = TextureButtonData::new();
        let n = Vector2::new(10.0, 20.0);
        let p = Vector2::new(30.0, 40.0);
        let h = Vector2::new(50.0, 60.0);
        let m = Vector2::new(70.0, 80.0);
        let base = Vector2::new(1.0, 2.0);

        assert_eq!(b.get_minimum_size(base, Some(n), Some(p), Some(h), Some(m)), n);
        assert_eq!(b.get_minimum_size(base, None, Some(p), Some(h), Some(m)), p);
        assert_eq!(b.get_minimum_size(base, None, None, Some(h), Some(m)), h);
        assert_eq!(b.get_minimum_size(base, None, None, None, Some(m)), m);
        assert_eq!(b.get_minimum_size(base, None, None, None, None), Vector2::ZERO);

        // Ignoring the texture size keeps the control minimum, and the result is
        // absolute (negative control minimums become positive).
        b.set_ignore_texture_size(true);
        assert_eq!(
            b.get_minimum_size(Vector2::new(-4.0, 3.0), Some(n), None, None, None),
            Vector2::new(4.0, 3.0)
        );

        // Negative texture sizes are made absolute too.
        b.set_ignore_texture_size(false);
        assert_eq!(
            b.get_minimum_size(base, Some(Vector2::new(-8.0, -9.0)), None, None, None),
            Vector2::new(8.0, 9.0)
        );
    }

    #[test]
    fn click_mask_without_layout_uses_the_mask_size() {
        let mut b = TextureButtonData::new();
        let mut m = mask(4, 4);
        m.set_bit(1, 1, true);
        b.set_click_mask(Some(m));

        assert_eq!(b.has_point(Vector2::new(1.5, 1.5)), Some(true));
        assert_eq!(b.has_point(Vector2::new(3.5, 3.5)), Some(false));
        assert_eq!(b.has_point(Vector2::new(4.0, 0.0)), Some(false));
        // Without a mask the caller falls back to Control::has_point.
        let empty = TextureButtonData::new();
        assert_eq!(empty.has_point(Vector2::ZERO), None);
    }

    #[test]
    fn click_mask_maps_scaled_points() {
        let mut b = TextureButtonData::new();
        let mut m = mask(2, 2);
        m.set_bit(0, 0, true);
        b.set_click_mask(Some(m));

        // A 20x20 image on a 2x2 mask: each mask pixel covers 10 screen pixels.
        b.set_layout(
            Rect2::new(Vector2::ZERO, Vector2::new(20.0, 20.0)),
            Rect2::new(Vector2::ZERO, Vector2::new(2.0, 2.0)),
            false,
        );
        assert_eq!(b.has_point(Vector2::new(1.0, 1.0)), Some(true));
        assert_eq!(b.has_point(Vector2::new(15.0, 15.0)), Some(false));
        // Outside the drawn rect is a miss before the mask lookup.
        assert_eq!(b.has_point(Vector2::new(25.0, 5.0)), Some(false));

        // With an offset the mask follows the image position.
        b.set_layout(
            Rect2::new(Vector2::new(10.0, 10.0), Vector2::new(20.0, 20.0)),
            Rect2::new(Vector2::ZERO, Vector2::new(2.0, 2.0)),
            false,
        );
        assert_eq!(b.has_point(Vector2::new(11.0, 11.0)), Some(true));
        assert_eq!(b.has_point(Vector2::new(1.0, 1.0)), Some(false));
    }

    #[test]
    fn tiled_click_mask_wraps_the_point() {
        let mut b = TextureButtonData::new();
        let mut m = mask(2, 2);
        m.set_bit(0, 0, true);
        b.set_click_mask(Some(m));
        b.set_layout(
            Rect2::new(Vector2::ZERO, Vector2::new(40.0, 40.0)),
            Rect2::new(Vector2::ZERO, Vector2::new(2.0, 2.0)),
            true,
        );
        // Tiles repeat every 2 px, so every even coordinate lands on (0,0).
        assert_eq!(b.has_point(Vector2::new(0.5, 0.5)), Some(true));
        assert_eq!(b.has_point(Vector2::new(20.5, 20.5)), Some(true));
        assert_eq!(b.has_point(Vector2::new(1.5, 0.5)), Some(false));
    }

    #[test]
    fn toggle_and_stretch_setters() {
        let mut b = TextureButtonData::new();
        b.set_texture_normal(Some(Rid::from_u64(1)));
        b.set_texture_pressed(Some(Rid::from_u64(2)));
        b.set_texture_hover(Some(Rid::from_u64(3)));
        b.set_texture_disabled(Some(Rid::from_u64(4)));
        b.set_texture_focused(Some(Rid::from_u64(5)));
        b.set_stretch_mode(StretchMode::KeepAspectCovered);
        b.set_flip_h(true);
        b.set_flip_v(true);

        assert_eq!(b.get_texture_normal(), Some(Rid::from_u64(1)));
        assert_eq!(b.get_texture_pressed(), Some(Rid::from_u64(2)));
        assert_eq!(b.get_texture_hover(), Some(Rid::from_u64(3)));
        assert_eq!(b.get_texture_disabled(), Some(Rid::from_u64(4)));
        assert_eq!(b.get_texture_focused(), Some(Rid::from_u64(5)));
        assert_eq!(b.get_stretch_mode(), StretchMode::KeepAspectCovered);
        assert!(b.is_flipped_h());
        assert!(b.is_flipped_v());

        assert_eq!(StretchMode::Scale as i32, 0);
        assert_eq!(StretchMode::Tile as i32, 1);
        assert_eq!(StretchMode::Keep as i32, 2);
        assert_eq!(StretchMode::KeepCentered as i32, 3);
        assert_eq!(StretchMode::KeepAspect as i32, 4);
        assert_eq!(StretchMode::KeepAspectCentered as i32, 5);
        assert_eq!(StretchMode::KeepAspectCovered as i32, 6);
    }
}
