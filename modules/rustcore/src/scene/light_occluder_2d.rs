//! `scene/2d/light_occluder_2d.*`: occluder polygon state.
//!
//! `OccluderPolygon2D` holds the vertices, the `closed` flag (default true) and
//! the cull mode (default disabled). `LightOccluder2D` holds a polygon
//! reference, its light mask (default 1) and the SDF-collision flag (off).
//! The RIDs, changed notifications and SDF generation are not ported.

use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CullMode {
    Disabled,
    Clockwise,
    CounterClockwise,
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::Disabled
    }
}

#[derive(Clone, Debug, Default)]
pub struct OccluderPolygon2DData {
    polygon: Vec<Vector2>,
    closed: bool,
    cull_mode: CullMode,
}

impl OccluderPolygon2DData {
    pub fn new() -> Self {
        Self {
            polygon: Vec::new(),
            closed: true,
            cull_mode: CullMode::Disabled,
        }
    }

    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn set_cull_mode(&mut self, mode: CullMode) {
        self.cull_mode = mode;
    }

    pub fn get_cull_mode(&self) -> CullMode {
        self.cull_mode
    }
}

#[derive(Clone, Debug)]
pub struct LightOccluder2DData {
    occluder_polygon: Option<OccluderPolygon2DData>,
    occluder_light_mask: i32,
    sdf_collision: bool,
}

impl Default for LightOccluder2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl LightOccluder2DData {
    pub fn new() -> Self {
        Self {
            occluder_polygon: None,
            occluder_light_mask: 1,
            sdf_collision: false,
        }
    }

    pub fn set_occluder_polygon(&mut self, polygon: Option<OccluderPolygon2DData>) {
        self.occluder_polygon = polygon;
    }

    pub fn get_occluder_polygon(&self) -> Option<&OccluderPolygon2DData> {
        self.occluder_polygon.as_ref()
    }

    pub fn set_occluder_light_mask(&mut self, mask: i32) {
        self.occluder_light_mask = mask;
    }

    pub fn get_occluder_light_mask(&self) -> i32 {
        self.occluder_light_mask
    }

    pub fn set_as_sdf_collision(&mut self, enable: bool) {
        self.sdf_collision = enable;
    }

    pub fn is_set_as_sdf_collision(&self) -> bool {
        self.sdf_collision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occluder_polygon_defaults() {
        let mut o = OccluderPolygon2DData::new();
        assert!(o.get_polygon().is_empty());
        assert!(o.is_closed());
        assert_eq!(o.get_cull_mode(), CullMode::Disabled);

        o.set_polygon(vec![Vector2::ZERO, Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)]);
        assert_eq!(o.get_polygon().len(), 3);
        o.set_closed(false);
        o.set_cull_mode(CullMode::CounterClockwise);
        assert!(!o.is_closed());
        assert_eq!(o.get_cull_mode(), CullMode::CounterClockwise);

        assert_eq!(CullMode::Disabled as i32, 0);
        assert_eq!(CullMode::Clockwise as i32, 1);
        assert_eq!(CullMode::CounterClockwise as i32, 2);
    }

    #[test]
    fn light_occluder_defaults_and_flags() {
        let mut l = LightOccluder2DData::new();
        assert!(l.get_occluder_polygon().is_none());
        assert_eq!(l.get_occluder_light_mask(), 1);
        assert!(!l.is_set_as_sdf_collision());

        l.set_occluder_polygon(Some(OccluderPolygon2DData::new()));
        assert!(l.get_occluder_polygon().is_some());
        l.set_occluder_light_mask(0xF);
        l.set_as_sdf_collision(true);
        assert_eq!(l.get_occluder_light_mask(), 0xF);
        assert!(l.is_set_as_sdf_collision());

        l.set_occluder_polygon(None);
        assert!(l.get_occluder_polygon().is_none());
    }
}
