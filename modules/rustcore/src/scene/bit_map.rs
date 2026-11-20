//! `scene/resources/bit_map.*`: a packed 1-bit-per-pixel grid.
//!
//! Ported: `create` (size validation and the `INT32_MAX` pixel limit),
//! `set_bit`/`get_bit` with index checks, `set_bit_rect` clipped to the bitmap,
//! the popcount, `resize` with nearest-neighbour sampling, `grow_mask` /
//! `shrink_mask` (circular kernel, clipped by the passed rect) and `blit`.
//!
//! Not ported: `create_from_image_alpha`, `convert_to_image` and the
//! marching-squares polygon extraction (they need `Image`/`Geometry2D` APIs
//! beyond this module).

use crate::math::rect2i::Rect2i;
use crate::math::vector2i::Vector2i;

/// `CMP_EPSILON2` from `core/typedefs.h`.
const CMP_EPSILON2: f32 = 1e-10;

#[derive(Clone, Debug, Default)]
pub struct BitMap {
    bits: Vec<u8>,
    width: i32,
    height: i32,
}

impl BitMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rejects sizes below 1 or above `INT32_MAX` total pixels.
    pub fn create(&mut self, width: i32, height: i32) -> bool {
        if width < 1 || height < 1 {
            return false;
        }
        if width as i64 * height as i64 > i32::MAX as i64 {
            return false;
        }
        let bytes = (width as usize * height as usize).div_ceil(8);
        self.bits = vec![0u8; bytes];
        self.width = width;
        self.height = height;
        true
    }

    pub fn get_size(&self) -> Vector2i {
        Vector2i::new(self.width, self.height)
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some((self.width as usize) * (y as usize) + x as usize)
    }

    pub fn set_bit(&mut self, x: i32, y: i32, value: bool) -> bool {
        let Some(ofs) = self.index(x, y) else {
            return false;
        };
        let byte = ofs / 8;
        let bit = ofs % 8;
        if value {
            self.bits[byte] |= 1 << bit;
        } else {
            self.bits[byte] &= !(1 << bit);
        }
        true
    }

    pub fn get_bit(&self, x: i32, y: i32) -> bool {
        match self.index(x, y) {
            Some(ofs) => (self.bits[ofs / 8] & (1 << (ofs % 8))) != 0,
            None => false,
        }
    }

    /// The rect is clipped to `(0, 0, width, height)` first.
    pub fn set_bit_rect(&mut self, rect: Rect2i, value: bool) {
        let current = Rect2i::new_xywh(0, 0, self.width, self.height).intersection(&rect);
        for y in current.position.y..current.position.y + current.size.y {
            for x in current.position.x..current.position.x + current.size.x {
                self.set_bit(x, y, value);
            }
        }
    }

    pub fn get_true_bit_count(&self) -> i32 {
        self.bits.iter().map(|b| b.count_ones() as i32).sum()
    }

    /// Nearest-neighbour resize; a zero dimension empties the bitmap, as the
    /// engine's failed `create` leaves an empty temporary behind.
    pub fn resize(&mut self, width: i32, height: i32) -> bool {
        if width < 0 || height < 0 {
            return false;
        }
        if width == self.width && height == self.height {
            return true;
        }
        if width == 0 || height == 0 {
            self.bits.clear();
            self.width = 0;
            self.height = 0;
            return true;
        }

        let mut new_bitmap = BitMap::new();
        if !new_bitmap.create(width, height) {
            return false;
        }
        let lw = if self.width == 0 { 0 } else { width };
        let lh = if self.height == 0 { 0 } else { height };
        let scale_x = self.width as f32 / width as f32;
        let scale_y = self.height as f32 / height as f32;
        for x in 0..lw {
            for y in 0..lh {
                let new_bit = self.get_bit((x as f32 * scale_x) as i32, (y as f32 * scale_y) as i32);
                new_bitmap.set_bit(x, y, new_bit);
            }
        }
        *self = new_bitmap;
        true
    }

    /// Positive pixels grow the set bits, negative (via `shrink_mask`) erode
    /// them. Only the area inside `rect` is considered, and outside it counts
    /// as unset when growing.
    pub fn grow_mask(&mut self, pixels: i32, rect: Rect2i) {
        if pixels == 0 {
            return;
        }
        let bit_value = pixels > 0;
        let pixels = pixels.abs();
        let pixels2 = (pixels * pixels) as f32;
        let r = Rect2i::new_xywh(0, 0, self.width, self.height).intersection(&rect);
        let copy = self.clone();

        for i in r.position.y..r.position.y + r.size.y {
            for j in r.position.x..r.position.x + r.size.x {
                if bit_value == self.get_bit(j, i) {
                    continue;
                }

                let mut found = false;
                'search: for y in (i - pixels)..=(i + pixels) {
                    for x in (j - pixels)..=(j + pixels) {
                        let mut outside = false;
                        if x < rect.position.x
                            || x >= rect.position.x + rect.size.x
                            || y < rect.position.y
                            || y >= rect.position.y + rect.size.y
                        {
                            // Outside the rect counts as unset.
                            if !bit_value {
                                outside = true;
                            } else {
                                continue;
                            }
                        }

                        let dx = (j - x) as f32;
                        let dy = (i - y) as f32;
                        let d = dx * dx + dy * dy - CMP_EPSILON2;
                        if d > pixels2 {
                            continue;
                        }

                        if outside || bit_value == copy.get_bit(x, y) {
                            found = true;
                            break 'search;
                        }
                    }
                }

                if found {
                    self.set_bit(j, i, bit_value);
                }
            }
        }
    }

    pub fn shrink_mask(&mut self, pixels: i32, rect: Rect2i) {
        self.grow_mask(-pixels, rect);
    }

    /// Copies set bits from `source`, clipped to this bitmap.
    pub fn blit(&mut self, position: Vector2i, source: &BitMap) {
        let width = source.get_width();
        let height = source.get_height();
        for i in 0..width {
            for j in 0..height {
                let px = position.x + i;
                let py = position.y + j;
                if px < 0 || px >= self.width || py < 0 || py >= self.height {
                    continue;
                }
                if source.get_bit(i, j) {
                    self.set_bit(px, py, true);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_validates_and_clears() {
        let mut b = BitMap::new();
        assert!(!b.create(0, 10));
        assert!(!b.create(10, 0));
        assert!(!b.create(-1, 10));
        assert_eq!(b.get_size(), Vector2i::new(0, 0));

        assert!(b.create(10, 10));
        assert_eq!(b.get_size(), Vector2i::new(10, 10));
        assert_eq!(b.get_true_bit_count(), 0);

        // Pixels greater than INT32_MAX are rejected.
        assert!(!b.create(100_000, 100_000));
        // Creating again clears the previous contents.
        b.set_bit(0, 0, true);
        assert!(b.create(4, 4));
        assert_eq!(b.get_true_bit_count(), 0);
    }

    #[test]
    fn bits_and_index_bounds() {
        let mut b = BitMap::new();
        b.create(10, 5);

        assert!(b.set_bit(3, 2, true));
        assert!(b.get_bit(3, 2));
        assert_eq!(b.get_true_bit_count(), 1);

        // Setting the same bit twice does not double count.
        assert!(b.set_bit(3, 2, true));
        assert_eq!(b.get_true_bit_count(), 1);
        assert!(b.set_bit(3, 2, false));
        assert!(!b.get_bit(3, 2));

        assert!(!b.set_bit(-1, 0, true));
        assert!(!b.set_bit(10, 0, true));
        assert!(!b.set_bit(0, 5, true));
        assert!(!b.get_bit(99, 99));
    }

    #[test]
    fn bit_rect_is_clipped() {
        let mut b = BitMap::new();
        b.create(8, 8);
        b.set_bit_rect(Rect2i::new_xywh(2, 2, 2, 2), true);
        assert_eq!(b.get_true_bit_count(), 4);
        assert!(b.get_bit(2, 2));
        assert!(b.get_bit(3, 3));
        assert!(!b.get_bit(4, 4));

        // Partially outside: only the overlapping cells are written.
        b.set_bit_rect(Rect2i::new_xywh(-2, -2, 4, 4), true);
        assert_eq!(b.get_true_bit_count(), 4 + 4);
        assert!(b.get_bit(0, 0));
        assert!(b.get_bit(1, 1));

        // Clearing with a clipped rect.
        b.set_bit_rect(Rect2i::new_xywh(0, 0, 100, 100), false);
        assert_eq!(b.get_true_bit_count(), 0);
    }

    #[test]
    fn resize_resamples_nearest_neighbour() {
        let mut b = BitMap::new();
        b.create(2, 2);
        b.set_bit(0, 0, true);
        b.set_bit(1, 1, true);

        // Doubling keeps the pattern in the matching quadrants.
        assert!(b.resize(4, 4));
        assert_eq!(b.get_size(), Vector2i::new(4, 4));
        assert!(b.get_bit(0, 0));
        assert!(b.get_bit(1, 1));
        assert!(b.get_bit(2, 2));
        assert!(b.get_bit(3, 3));
        assert!(b.get_bit(0, 1));
        assert!(!b.get_bit(0, 3));
        // Sampling is `x * (old/new)`, so each source pixel covers a 2x2 block.
        assert_eq!(b.get_true_bit_count(), 8);

        // Halving samples with the scale factor.
        assert!(b.resize(2, 2));
        assert_eq!(b.get_true_bit_count(), 2);

        assert!(!b.resize(-1, 2));
        // A zero dimension empties the map.
        assert!(b.resize(0, 4));
        assert_eq!(b.get_size(), Vector2i::new(0, 0));
        assert_eq!(b.get_true_bit_count(), 0);
    }

    #[test]
    fn grow_and_shrink_mask() {
        let mut b = BitMap::new();
        b.create(7, 7);
        b.set_bit(3, 3, true);
        let rect = Rect2i::new_xywh(0, 0, 7, 7);

        b.grow_mask(1, rect);
        // The kernel is a circle (`distance^2 <= pixels^2`), so a radius of 1
        // around one bit is a plus sign, not a 3x3 square.
        assert_eq!(b.get_true_bit_count(), 5);
        assert!(b.get_bit(3, 2));
        assert!(b.get_bit(4, 3));
        assert!(!b.get_bit(2, 2), "diagonal distance is sqrt(2) > 1");

        // Growing again expands to the radius-2 diamond (13 cells).
        b.grow_mask(1, rect);
        assert_eq!(b.get_true_bit_count(), 13);
        assert!(b.get_bit(2, 2));
        assert!(!b.get_bit(1, 1));

        // Shrinking undoes a growth step.
        b.shrink_mask(1, rect);
        assert_eq!(b.get_true_bit_count(), 5);
        b.shrink_mask(1, rect);
        assert_eq!(b.get_true_bit_count(), 1);

        // pixels == 0 is a no-op.
        b.grow_mask(0, rect);
        assert_eq!(b.get_true_bit_count(), 1);
    }

    #[test]
    fn blit_clips_and_only_sets() {
        let mut target = BitMap::new();
        target.create(4, 4);
        let mut source = BitMap::new();
        source.create(2, 2);
        source.set_bit(0, 0, true);
        source.set_bit(1, 0, true);

        target.blit(Vector2i::new(1, 1), &source);
        assert!(target.get_bit(1, 1));
        assert!(target.get_bit(2, 1));
        assert_eq!(target.get_true_bit_count(), 2);

        // Negative offsets clip.
        target.blit(Vector2i::new(-1, -1), &source);
        // Only the source (1,0) maps to target (0,-1)/(1,-1): nothing visible.
        assert_eq!(target.get_true_bit_count(), 2);

        let mut corner = BitMap::new();
        corner.create(2, 2);
        corner.set_bit(0, 0, true);
        target.blit(Vector2i::new(-0, 0), &corner);
        assert!(target.get_bit(0, 0));
        assert_eq!(target.get_true_bit_count(), 3);
    }
}
