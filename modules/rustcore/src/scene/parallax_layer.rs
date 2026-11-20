//! `scene/2d/parallax_layer.*`: motion scale/offset and mirroring.
//!
//! `set_base_offset_and_scale` is the whole point of the class: it maps the
//! viewport offset through `motion_scale`, adds the layer offset and wraps the
//! result into the mirroring period. Here that is `compute_offset_scale`,
//! returning the values instead of applying them to a node.
//!
//! Not ported: the `RenderingServer` mirroring call and the tree notifications
//! that capture `orig_offset`/`orig_scale`.

use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct ParallaxLayerData {
    motion_scale: Vector2,
    motion_offset: Vector2,
    mirroring: Vector2,
    orig_offset: Vector2,
    orig_scale: Vector2,
}

impl Default for ParallaxLayerData {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallaxLayerData {
    pub fn new() -> Self {
        Self {
            motion_scale: Vector2::ONE,
            motion_offset: Vector2::ZERO,
            mirroring: Vector2::ZERO,
            orig_offset: Vector2::ZERO,
            orig_scale: Vector2::ONE,
        }
    }

    pub fn set_motion_scale(&mut self, scale: Vector2) {
        self.motion_scale = scale;
    }

    pub fn get_motion_scale(&self) -> Vector2 {
        self.motion_scale
    }

    pub fn set_motion_offset(&mut self, offset: Vector2) {
        self.motion_offset = offset;
    }

    pub fn get_motion_offset(&self) -> Vector2 {
        self.motion_offset
    }

    /// `set_mirroring`: each component is clamped to `>= 0` (`maxf(0)`).
    pub fn set_mirroring(&mut self, mirroring: Vector2) {
        self.mirroring = Vector2::new(mirroring.x.max(0.0), mirroring.y.max(0.0));
    }

    pub fn get_mirroring(&self) -> Vector2 {
        self.mirroring
    }

    /// Captured when the layer enters the tree (`NOTIFICATION_ENTER_TREE`).
    pub fn set_origin(&mut self, offset: Vector2, scale: Vector2) {
        self.orig_offset = offset;
        self.orig_scale = scale;
    }

    pub fn get_origin(&self) -> (Vector2, Vector2) {
        (self.orig_offset, self.orig_scale)
    }

    /// `set_base_offset_and_scale` without the node write-back: returns the
    /// position and scale the layer would get.
    pub fn compute_offset_scale(&self, base_offset: Vector2, scale: f32) -> (Vector2, Vector2) {
        let mut new_ofs = base_offset * self.motion_scale
            + self.motion_offset * scale
            + self.orig_offset * scale;

        if self.mirroring.x != 0.0 {
            let den = self.mirroring.x * scale;
            new_ofs.x -= den * (new_ofs.x / den).ceil();
        }
        if self.mirroring.y != 0.0 {
            let den = self.mirroring.y * scale;
            new_ofs.y -= den * (new_ofs.y / den).ceil();
        }

        (new_ofs, self.orig_scale * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = ParallaxLayerData::new();
        assert_eq!(p.get_motion_scale(), Vector2::ONE);
        assert_eq!(p.get_motion_offset(), Vector2::ZERO);
        assert_eq!(p.get_mirroring(), Vector2::ZERO);
    }

    #[test]
    fn mirroring_is_clamped_to_zero() {
        let mut p = ParallaxLayerData::new();
        p.set_mirroring(Vector2::new(-5.0, 10.0));
        assert_eq!(p.get_mirroring(), Vector2::new(0.0, 10.0));
    }

    #[test]
    fn offset_formula_without_mirroring() {
        let mut p = ParallaxLayerData::new();
        p.set_motion_scale(Vector2::new(0.5, 0.25));
        p.set_motion_offset(Vector2::new(10.0, -4.0));
        p.set_origin(Vector2::new(1.0, 2.0), Vector2::ONE);

        // 100*0.5 + 10*2 + 1*2 = 72 ; 50*0.25 + (-4)*2 + 2*2 = 8.5
        let (ofs, scale) = p.compute_offset_scale(Vector2::new(100.0, 50.0), 2.0);
        assert!((ofs.x - 72.0).abs() < 1e-4, "{ofs:?}");
        assert!((ofs.y - 8.5).abs() < 1e-4, "{ofs:?}");
        assert_eq!(scale, Vector2::new(2.0, 2.0));
    }

    #[test]
    fn mirroring_wraps_into_the_period() {
        let mut p = ParallaxLayerData::new();
        p.set_mirroring(Vector2::new(10.0, 10.0));
        // new_ofs = (7, -3); x wraps by ceil(0.7)=1 -> -3; y by ceil(-0.3)=0 -> -3.
        let (ofs, _) = p.compute_offset_scale(Vector2::new(7.0, -3.0), 1.0);
        assert!((ofs.x + 3.0).abs() < 1e-4, "{ofs:?}");
        assert!((ofs.y + 3.0).abs() < 1e-4, "{ofs:?}");

        // Scaling doubles the period: den = 20.
        let (ofs, _) = p.compute_offset_scale(Vector2::new(7.0, 0.0), 2.0);
        assert!((ofs.x - (7.0 - 20.0)).abs() < 1e-4, "{ofs:?}");
    }

    #[test]
    fn original_scale_multiplies_the_viewport_scale() {
        let mut p = ParallaxLayerData::new();
        p.set_origin(Vector2::ZERO, Vector2::new(3.0, 3.0));
        let (_, scale) = p.compute_offset_scale(Vector2::ZERO, 0.5);
        assert_eq!(scale, Vector2::new(1.5, 1.5));
    }
}
