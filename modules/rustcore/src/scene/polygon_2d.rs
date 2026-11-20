//! `scene/2d/polygon_2d.*`: polygon data and outer-contour triangulation.
//!
//! Stored: the outline, the optional hole list (`polygons`, where index 0 is the
//! outer contour and the rest are holes), UVs, vertex colors, texture transform
//! and the invert flag.
//!
//! Triangulation: `triangulate_outer` runs the ported ear-clipping
//! (`math::triangulate`) over the outline. The engine triangulates outlines
//! **with holes** through `Geometry2D::triangulate_polygon` (Clipper), which is
//! not ported; holes are therefore rejected here rather than silently ignored.

use crate::math::triangulate;
use crate::math::vector2::Vector2;

#[derive(Clone, Debug, Default)]
pub struct Polygon2DData {
    polygon: Vec<Vector2>,
    polygons: Vec<Vec<i32>>,
    uv: Vec<Vector2>,
    vertex_colors: Vec<crate::math::color::Color>,
    texture_offset: Vector2,
    texture_scale: Vector2,
    texture_rotation: f32,
    invert: bool,
    invert_border: f32,
    poly_as_lines: bool,
    internal_vertex_count: i32,
    antialiased: bool,
}

impl Polygon2DData {
    pub fn new() -> Self {
        Self {
            texture_scale: Vector2::ONE,
            ..Default::default()
        }
    }

    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }

    /// `get_polygon_count` / `get_polygon(index)` for the hole list; an empty
    /// list means "just the outline".
    pub fn get_polygon_count(&self) -> usize {
        self.polygons.len()
    }

    pub fn get_polygon_at(&self, index: usize) -> Option<&[i32]> {
        self.polygons.get(index).map(|v| v.as_slice())
    }

    /// Each entry is a list of vertex indices; entry 0 is the outer contour.
    pub fn set_polygons(&mut self, polygons: Vec<Vec<i32>>) {
        self.polygons = polygons;
    }

    pub fn set_uv(&mut self, uv: Vec<Vector2>) {
        self.uv = uv;
    }

    pub fn get_uv(&self) -> &[Vector2] {
        &self.uv
    }

    pub fn set_vertex_colors(&mut self, colors: Vec<crate::math::color::Color>) {
        self.vertex_colors = colors;
    }

    pub fn get_vertex_colors(&self) -> &[crate::math::color::Color] {
        &self.vertex_colors
    }

    pub fn set_texture_offset(&mut self, offset: Vector2) {
        self.texture_offset = offset;
    }

    pub fn get_texture_offset(&self) -> Vector2 {
        self.texture_offset
    }

    pub fn set_texture_scale(&mut self, scale: Vector2) {
        self.texture_scale = scale;
    }

    pub fn get_texture_scale(&self) -> Vector2 {
        self.texture_scale
    }

    pub fn set_texture_rotation(&mut self, rotation: f32) {
        self.texture_rotation = rotation;
    }

    pub fn get_texture_rotation(&self) -> f32 {
        self.texture_rotation
    }

    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    pub fn get_invert(&self) -> bool {
        self.invert
    }

    pub fn set_invert_border(&mut self, border: f32) {
        self.invert_border = border.max(0.0);
    }

    pub fn get_invert_border(&self) -> f32 {
        self.invert_border
    }

    pub fn set_poly_as_lines(&mut self, enabled: bool) {
        self.poly_as_lines = enabled;
    }

    pub fn is_poly_as_lines(&self) -> bool {
        self.poly_as_lines
    }

    pub fn set_internal_vertex_count(&mut self, count: i32) {
        self.internal_vertex_count = count;
    }

    pub fn get_internal_vertex_count(&self) -> i32 {
        self.internal_vertex_count
    }

    pub fn set_antialiased(&mut self, antialiased: bool) {
        self.antialiased = antialiased;
    }

    pub fn get_antialiased(&self) -> bool {
        self.antialiased
    }

    /// Triangulates the outline. `None` when there are fewer than three points,
    /// the contour is degenerate, or holes are present (the hole-aware Clipper
    /// path is not ported).
    pub fn triangulate_outer(&self) -> Option<Vec<i32>> {
        if self.polygon.len() < 3 || self.polygons.len() > 1 {
            return None;
        }
        triangulate::triangulate(&self.polygon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Vector2> {
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, 10.0),
        ]
    }

    #[test]
    fn defaults_and_polygon_storage() {
        let mut p = Polygon2DData::new();
        assert!(p.get_polygon().is_empty());
        assert_eq!(p.get_texture_scale(), Vector2::ONE);
        assert!(!p.get_invert());
        assert!(!p.is_poly_as_lines());
        assert_eq!(p.get_polygon_count(), 0);

        p.set_polygon(square());
        assert_eq!(p.get_polygon().len(), 4);
    }

    #[test]
    fn outer_triangulation() {
        let mut p = Polygon2DData::new();
        p.set_polygon(square());
        let idx = p.triangulate_outer().expect("square triangulates");
        assert_eq!(idx.len(), 6);
        assert!(idx.iter().all(|i| (0..4).contains(i)));
    }

    #[test]
    fn holes_are_rejected_not_ignored() {
        let mut p = Polygon2DData::new();
        p.set_polygon(square());
        p.set_polygons(vec![vec![0, 1, 2, 3], vec![4, 5, 6]]);
        assert_eq!(p.get_polygon_count(), 2);
        // The Clipper-based hole path is not ported, so this must fail loudly.
        assert!(p.triangulate_outer().is_none());
    }

    #[test]
    fn degenerate_outline_has_no_triangles() {
        let mut p = Polygon2DData::new();
        p.set_polygon(vec![Vector2::ZERO, Vector2::RIGHT]);
        assert!(p.triangulate_outer().is_none());
    }

    #[test]
    fn uvs_colors_and_texture_transform() {
        let mut p = Polygon2DData::new();
        p.set_uv(vec![Vector2::ZERO, Vector2::ONE]);
        p.set_vertex_colors(vec![crate::math::color::Color::new(1.0, 0.0, 0.0, 1.0)]);
        assert_eq!(p.get_uv().len(), 2);
        assert_eq!(p.get_vertex_colors().len(), 1);

        p.set_texture_offset(Vector2::new(3.0, -1.0));
        p.set_texture_scale(Vector2::new(2.0, 2.0));
        p.set_texture_rotation(0.5);
        assert_eq!(p.get_texture_offset(), Vector2::new(3.0, -1.0));
        assert_eq!(p.get_texture_scale(), Vector2::new(2.0, 2.0));
        assert!((p.get_texture_rotation() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn invert_border_is_clamped() {
        let mut p = Polygon2DData::new();
        p.set_invert_border(-4.0);
        assert_eq!(p.get_invert_border(), 0.0);
        p.set_invert_border(2.5);
        assert_eq!(p.get_invert_border(), 2.5);
    }
}
