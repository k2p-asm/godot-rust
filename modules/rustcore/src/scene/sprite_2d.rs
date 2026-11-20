//! `scene/2d/sprite_2d.*`: frame slicing and rect math.
//!
//! The engine's `Sprite2D` holds a `Texture2D`; here the texture is reduced to
//! its size (`set_texture_size`), which is all the geometry needs. Everything
//! else follows the engine: frames are indexed row-major over
//! `hframes x vframes`, changing `hframes` keeps the current column when it
//! still exists, and `get_rect` divides the region/texture by the frame grid.
//!
//! Not ported: drawing, `is_pixel_opaque` (needs pixel access), and the
//! pixel-snap path (`get_viewport()->is_snap_2d_transforms_to_pixel_enabled()`).

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;

#[derive(Clone, Copy, Debug)]
pub struct Sprite2DData {
    texture_size: Option<Vector2i>,
    centered: bool,
    offset: Vector2,
    flip_h: bool,
    flip_v: bool,
    region_enabled: bool,
    region_rect: Rect2,
    region_filter_clip: bool,
    hframes: i32,
    vframes: i32,
    frame: i32,
}

impl Default for Sprite2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Sprite2DData {
    pub fn new() -> Self {
        Self {
            texture_size: None,
            centered: true,
            offset: Vector2::ZERO,
            flip_h: false,
            flip_v: false,
            region_enabled: false,
            region_rect: Rect2::new(Vector2::ZERO, Vector2::new(1.0, 1.0)),
            region_filter_clip: false,
            hframes: 1,
            vframes: 1,
            frame: 0,
        }
    }

    pub fn set_texture_size(&mut self, size: Option<Vector2i>) {
        self.texture_size = size;
    }

    pub fn set_centered(&mut self, centered: bool) {
        self.centered = centered;
    }

    pub fn is_centered(&self) -> bool {
        self.centered
    }

    pub fn set_offset(&mut self, offset: Vector2) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> Vector2 {
        self.offset
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

    pub fn set_region_enabled(&mut self, enabled: bool) {
        self.region_enabled = enabled;
    }

    pub fn is_region_enabled(&self) -> bool {
        self.region_enabled
    }

    pub fn set_region_rect(&mut self, rect: Rect2) {
        self.region_rect = rect;
    }

    pub fn get_region_rect(&self) -> Rect2 {
        self.region_rect
    }

    pub fn set_region_filter_clip_enabled(&mut self, enabled: bool) {
        self.region_filter_clip = enabled;
    }

    pub fn is_region_filter_clip_enabled(&self) -> bool {
        self.region_filter_clip
    }

    /// Rejected when outside `hframes * vframes`; the previous frame is kept.
    pub fn set_frame(&mut self, frame: i32) -> bool {
        if frame < 0 || frame >= self.vframes * self.hframes {
            return false;
        }
        self.frame = frame;
        true
    }

    pub fn get_frame(&self) -> i32 {
        self.frame
    }

    /// Rejected when outside the grid; the previous frame is kept.
    pub fn set_frame_coords(&mut self, coords: Vector2i) -> bool {
        if coords.x < 0 || coords.x >= self.hframes || coords.y < 0 || coords.y >= self.vframes {
            return false;
        }
        self.set_frame(coords.y * self.hframes + coords.x)
    }

    pub fn get_frame_coords(&self) -> Vector2i {
        Vector2i::new(self.frame % self.hframes, self.frame / self.hframes)
    }

    /// `set_vframes`: below 1 is rejected; the frame resets when it no longer
    /// fits.
    pub fn set_vframes(&mut self, amount: i32) -> bool {
        if amount < 1 {
            return false;
        }
        if self.vframes == amount {
            return true;
        }
        self.vframes = amount;
        if self.frame >= self.vframes * self.hframes {
            self.frame = 0;
        }
        true
    }

    pub fn get_vframes(&self) -> i32 {
        self.vframes
    }

    /// `set_hframes`: with multiple rows the frame follows its row when its
    /// column still exists; otherwise (or without rows) it resets to 0.
    pub fn set_hframes(&mut self, amount: i32) -> bool {
        if amount < 1 {
            return false;
        }
        if self.hframes == amount {
            return true;
        }

        if self.vframes > 1 {
            let original_column = self.frame % self.hframes;
            if original_column >= amount {
                self.frame = 0;
            } else {
                let original_row = self.frame / self.hframes;
                self.frame = original_row * amount + original_column;
            }
        }
        self.hframes = amount;
        if self.frame >= self.vframes * self.hframes {
            self.frame = 0;
        }
        true
    }

    pub fn get_hframes(&self) -> i32 {
        self.hframes
    }

    /// `_get_rect_offset`, minus the viewport pixel snap.
    pub fn get_rect_offset(&self, size: Vector2i) -> Vector2 {
        let mut ofs = self.offset;
        if self.centered {
            ofs -= Vector2::new(size.x as f32, size.y as f32) / 2.0;
        }
        ofs
    }

    /// `get_rect`: the frame's destination rect in local space.
    pub fn get_rect(&self) -> Rect2 {
        let size = match self.texture_size {
            None => return Rect2::new(Vector2::ZERO, Vector2::ONE),
            Some(s) => {
                if self.region_enabled {
                    Vector2i::new(self.region_rect.size.x as i32, self.region_rect.size.y as i32)
                } else {
                    s
                }
            }
        };

        let frame_size = Vector2i::new(size.x / self.hframes, size.y / self.vframes);
        let ofs = self.get_rect_offset(frame_size);

        let size = if frame_size.x == 0 || frame_size.y == 0 {
            Vector2i::new(1, 1)
        } else {
            frame_size
        };

        Rect2::new(ofs, Vector2::new(size.x as f32, size.y as f32))
    }

    /// `_get_rects`: source rect inside the texture (or region) and destination
    /// rect, with flips expressed as negative sizes.
    pub fn get_rects(&self) -> Option<(Rect2, Rect2, bool)> {
        let base_rect = if self.region_enabled {
            self.region_rect
        } else {
            let s = self.texture_size?;
            Rect2::new(Vector2::ZERO, Vector2::new(s.x as f32, s.y as f32))
        };
        let filter_clip = if self.region_enabled {
            self.region_filter_clip
        } else {
            false
        };

        let frame_size = Vector2::new(
            base_rect.size.x / self.hframes as f32,
            base_rect.size.y / self.vframes as f32,
        );
        let frame_offset = Vector2::new(
            (self.frame % self.hframes) as f32,
            (self.frame / self.hframes) as f32,
        ) * frame_size;

        let src = Rect2::new(base_rect.position + frame_offset, frame_size);

        let mut dest_offset = self.offset;
        if self.centered {
            dest_offset -= frame_size / 2.0;
        }
        let mut dest_size = frame_size;
        if self.flip_h {
            dest_size.x = -dest_size.x;
        }
        if self.flip_v {
            dest_size.y = -dest_size.y;
        }

        Some((src, Rect2::new(dest_offset, dest_size), filter_clip))
    }

    pub fn is_editor_region_rect_draggable(&self) -> bool {
        self.hframes <= 1 && self.vframes <= 1 && self.region_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite(w: i32, h: i32) -> Sprite2DData {
        let mut s = Sprite2DData::new();
        s.set_texture_size(Some(Vector2i::new(w, h)));
        s
    }

    #[test]
    fn defaults() {
        let s = Sprite2DData::new();
        assert!(s.is_centered());
        assert_eq!(s.get_hframes(), 1);
        assert_eq!(s.get_vframes(), 1);
        assert_eq!(s.get_frame(), 0);
        assert!(!s.is_flipped_h() && !s.is_flipped_v());
        // No texture: 1x1 at the origin.
        assert_eq!(s.get_rect(), Rect2::new(Vector2::ZERO, Vector2::ONE));
        assert!(s.get_rects().is_none());
    }

    #[test]
    fn centred_sprite_rect() {
        let mut s = sprite(64, 32);
        // Centered: the rect starts at -half size.
        let r = s.get_rect();
        assert_eq!(r.size, Vector2::new(64.0, 32.0));
        assert_eq!(r.position, Vector2::new(-32.0, -16.0));

        s.set_centered(false);
        assert_eq!(s.get_rect().position, Vector2::ZERO);
        s.set_offset(Vector2::new(5.0, -3.0));
        assert_eq!(s.get_rect().position, Vector2::new(5.0, -3.0));
    }

    #[test]
    fn frame_grid_divides_the_texture() {
        let mut s = sprite(64, 32);
        s.set_hframes(4);
        s.set_vframes(2);
        assert_eq!(s.get_rect().size, Vector2::new(16.0, 16.0));

        s.set_centered(false);
        s.set_frame_coords(Vector2i::new(2, 1));
        assert_eq!(s.get_frame(), 6);
        let (src, _, _) = s.get_rects().unwrap();
        assert_eq!(src.position, Vector2::new(32.0, 16.0));
        assert_eq!(src.size, Vector2::new(16.0, 16.0));
    }

    #[test]
    fn frame_bounds_are_rejected() {
        let mut s = sprite(20, 20);
        s.set_hframes(2);
        s.set_vframes(2);
        assert!(s.set_frame(3));
        assert!(!s.set_frame(4));
        assert_eq!(s.get_frame(), 3);
        assert!(!s.set_frame(-1));

        assert!(s.set_frame_coords(Vector2i::new(1, 1)));
        assert_eq!(s.get_frame(), 3);
        assert!(!s.set_frame_coords(Vector2i::new(2, 0)));
        assert_eq!(s.get_frame(), 3);
        assert_eq!(s.get_frame_coords(), Vector2i::new(1, 1));
    }

    #[test]
    fn changing_columns_keeps_the_row_when_possible() {
        let mut s = sprite(100, 100);
        s.set_hframes(4);
        s.set_vframes(2);
        // Frame 5 = row 1, column 1.
        s.set_frame(5);
        s.set_hframes(3);
        assert_eq!(s.get_frame(), 4, "row 1, column 1 with 3 columns");

        // The column no longer exists: reset.
        let mut t = sprite(100, 100);
        t.set_hframes(4);
        t.set_vframes(1);
        t.set_frame(3);
        t.set_hframes(2);
        assert_eq!(t.get_frame(), 0);
    }

    #[test]
    fn range_setters_reject_zero() {
        let mut s = Sprite2DData::new();
        assert!(!s.set_hframes(0));
        assert!(!s.set_vframes(0));
        assert!(s.set_hframes(1));
        assert!(s.set_vframes(1));
    }

    #[test]
    fn flips_negate_the_destination_size() {
        let mut s = sprite(10, 10);
        s.set_flip_h(true);
        let (_, dst, _) = s.get_rects().unwrap();
        assert_eq!(dst.size, Vector2::new(-10.0, 10.0));
        s.set_flip_v(true);
        let (_, dst, _) = s.get_rects().unwrap();
        assert_eq!(dst.size, Vector2::new(-10.0, -10.0));
    }

    #[test]
    fn region_rect_overrides_the_texture_size() {
        let mut s = sprite(100, 100);
        s.set_region_enabled(true);
        s.set_region_rect(Rect2::new(Vector2::new(10.0, 20.0), Vector2::new(30.0, 40.0)));
        s.set_hframes(3);
        s.set_vframes(2);
        assert_eq!(s.get_rect().size, Vector2::new(10.0, 20.0));

        let (src, _, _) = s.get_rects().unwrap();
        assert_eq!(src.position, Vector2::new(10.0, 20.0));
        assert_eq!(src.size, Vector2::new(10.0, 20.0));

        // Draggable only with a single frame and a region.
        assert!(!s.is_editor_region_rect_draggable());
        s.set_hframes(1);
        s.set_vframes(1);
        assert!(s.is_editor_region_rect_draggable());
        s.set_region_enabled(false);
        assert!(!s.is_editor_region_rect_draggable());
    }

    #[test]
    fn filter_clip_only_applies_to_regions() {
        let mut s = sprite(10, 10);
        s.set_region_filter_clip_enabled(true);
        let (_, _, clip) = s.get_rects().unwrap();
        assert!(!clip, "region disabled: filter clip is off");

        s.set_region_enabled(true);
        let (_, _, clip) = s.get_rects().unwrap();
        assert!(clip);
    }
}
