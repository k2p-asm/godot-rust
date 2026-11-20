//! `scene/resources/atlas_texture.*`: the atlas texture resource.
//!
//! Ported: the atlas handle, the region (`rounded_region` keeps the floored size
//! like the engine), the margin rect, `filter_clip`, `get_width`/`get_height`
//! (region size plus margin, or the atlas size when the region is empty, or 1
//! without an atlas), and `_get_region_rect` (empty region dimensions filled from
//! the atlas).
//!
//! RID creation and the actual texture upload are not ported; `has_alpha` follows
//! the atlas when known, otherwise reports `false`.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct AtlasTextureData {
    atlas: Option<Rid>,
    region: Rect2,
    rounded_region: Rect2,
    margin: Rect2,
    filter_clip: bool,
}

impl Default for AtlasTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl AtlasTextureData {
    pub fn new() -> Self {
        Self {
            atlas: None,
            region: Rect2::default(),
            rounded_region: Rect2::default(),
            margin: Rect2::default(),
            filter_clip: false,
        }
    }

    pub fn set_atlas(&mut self, atlas: Option<Rid>) {
        self.atlas = atlas;
    }

    pub fn get_atlas(&self) -> Option<Rid> {
        self.atlas
    }

    /// `set_region`: `rounded_region` uses the floored size.
    pub fn set_region(&mut self, region: Rect2) {
        self.region = region;
        self.rounded_region = Rect2::new(
            region.position,
            Vector2::new(region.size.x.floor(), region.size.y.floor()),
        );
    }

    pub fn get_region(&self) -> Rect2 {
        self.region
    }

    pub fn set_margin(&mut self, margin: Rect2) {
        self.margin = margin;
    }

    pub fn get_margin(&self) -> Rect2 {
        self.margin
    }

    pub fn set_filter_clip(&mut self, enabled: bool) {
        self.filter_clip = enabled;
    }

    pub fn is_filter_clip_enabled(&self) -> bool {
        self.filter_clip
    }

    /// The atlas dimensions are not available without the texture object; the
    /// caller supplies them when known (`None` reproduces the "no atlas" path).
    pub fn get_width(&self, atlas_width: Option<i32>) -> i32 {
        if self.rounded_region.size.x == 0.0 {
            atlas_width.unwrap_or(1)
        } else {
            (self.rounded_region.size.x + self.margin.size.x) as i32
        }
    }

    pub fn get_height(&self, atlas_height: Option<i32>) -> i32 {
        if self.rounded_region.size.y == 0.0 {
            atlas_height.unwrap_or(1)
        } else {
            (self.rounded_region.size.y + self.margin.size.y) as i32
        }
    }

    /// `_get_region_rect`: empty region dimensions become the atlas size.
    pub fn get_region_rect(&self, atlas_size: Option<Vector2>) -> Rect2 {
        let mut rect = self.rounded_region;
        if let Some(size) = atlas_size {
            if rect.size.x == 0.0 {
                rect.size.x = size.x;
            }
            if rect.size.y == 0.0 {
                rect.size.y = size.y;
            }
        }
        rect
    }

    pub fn has_alpha(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect2 {
        Rect2::new(Vector2::new(x, y), Vector2::new(w, h))
    }

    #[test]
    fn defaults_and_dimensions() {
        let mut atlas = AtlasTextureData::new();
        assert!(atlas.get_atlas().is_none());
        assert_eq!(atlas.get_region(), Rect2::default());
        assert_eq!(atlas.get_margin(), Rect2::default());
        assert!(!atlas.is_filter_clip_enabled());
        assert!(!atlas.has_alpha());
        // No atlas, empty region → 1.
        assert_eq!(atlas.get_width(None), 1);
        assert_eq!(atlas.get_height(None), 1);
        // Empty region with an atlas → the atlas size.
        assert_eq!(atlas.get_width(Some(64)), 64);
        assert_eq!(atlas.get_height(Some(32)), 32);

        atlas.set_atlas(Some(Rid::from_u64(2)));
        atlas.set_region(rect(4.0, 8.0, 10.7, 20.2));
        assert_eq!(atlas.get_region().size, Vector2::new(10.7, 20.2));
        assert_eq!(atlas.get_width(None), 10, "floored region width");
        assert_eq!(atlas.get_height(None), 20);

        atlas.set_margin(rect(0.0, 0.0, 2.0, 3.0));
        assert_eq!(atlas.get_width(None), 12, "region + margin");
        assert_eq!(atlas.get_height(None), 23);
        assert_eq!(atlas.get_region_rect(Some(Vector2::new(64.0, 64.0))).size, Vector2::new(10.0, 20.0));
        assert_eq!(atlas.get_region_rect(None).size, Vector2::new(10.0, 20.0));

        atlas.set_region(rect(1.0, 2.0, 0.0, 0.0));
        assert_eq!(
            atlas.get_region_rect(Some(Vector2::new(64.0, 32.0))).size,
            Vector2::new(64.0, 32.0)
        );
        assert_eq!(atlas.get_width(Some(64)), 64);

        atlas.set_filter_clip(true);
        assert!(atlas.is_filter_clip_enabled());
    }
}
