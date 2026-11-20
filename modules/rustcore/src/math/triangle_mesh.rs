//! Core of `core/math/triangle_mesh.h/.cpp`, without the BVH.
//!
//! Modelled: vertices + triangles (normal, indices, surface index), building
//! from `Face3`s, `get_faces`, `get_aabb`, `is_valid` and the total area.
//!
//! NOT modelled: the BVH (`_create_bvh`, `intersect_segment`, `intersect_ray`,
//! `inside_convex_shape`) and the `GDCLASS` registration. Those need the object
//! system, so this module is **non-differential**. The BVH is the reason the
//! class exists at all, so treat this as the data layer only.

use crate::math::aabb::Aabb;
use crate::math::face3::Face3;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub normal: Vector3,
    pub indices: [i32; 3],
    pub surface_index: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TriangleMesh {
    pub vertices: Vec<Vector3>,
    pub triangles: Vec<Triangle>,
}

impl TriangleMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds from faces. Unlike the engine this does not weld duplicate
    /// vertices, so `vertices.len() == 3 * faces.len()`.
    pub fn create_from_faces(faces: &[Face3]) -> Self {
        let mut vertices = Vec::with_capacity(faces.len() * 3);
        let mut triangles = Vec::with_capacity(faces.len());

        for (t, f) in faces.iter().enumerate() {
            let base = (t * 3) as i32;
            vertices.extend_from_slice(&f.vertex);
            triangles.push(Triangle {
                normal: f.get_plane().normal,
                indices: [base, base + 1, base + 2],
                surface_index: 0,
            });
        }

        Self { vertices, triangles }
    }

    /// True when there is at least one triangle (`is_valid`).
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.triangles.is_empty()
    }

    #[inline]
    pub fn get_triangle_count(&self) -> usize {
        self.triangles.len()
    }

    #[inline]
    pub fn get_vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// `get_faces`: one `Face3` per triangle, in order.
    pub fn get_faces(&self) -> Vec<Face3> {
        self.triangles
            .iter()
            .map(|t| {
                Face3::new(
                    self.vertices[t.indices[0] as usize],
                    self.vertices[t.indices[1] as usize],
                    self.vertices[t.indices[2] as usize],
                )
            })
            .collect()
    }

    /// Bounding box of every vertex; an empty mesh yields the zero box.
    pub fn get_aabb(&self) -> Aabb {
        let Some(first) = self.vertices.first() else {
            return Aabb::default();
        };
        let mut aabb = Aabb::new(*first, Vector3::ZERO);
        for v in self.vertices.iter().skip(1) {
            aabb.expand_to(*v);
        }
        aabb
    }

    /// Sum of the triangle areas.
    pub fn get_area(&self) -> f32 {
        self.get_faces().iter().map(|f| f.get_area()).sum()
    }

    /// Recomputes every triangle normal from its vertices.
    pub fn recompute_normals(&mut self) {
        for t in self.triangles.iter_mut() {
            let f = Face3::new(
                self.vertices[t.indices[0] as usize],
                self.vertices[t.indices[1] as usize],
                self.vertices[t.indices[2] as usize],
            );
            t.normal = f.get_plane().normal;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad() -> TriangleMesh {
        // Two triangles forming a unit square in the XY plane.
        let a = Face3::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
        );
        let b = Face3::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        TriangleMesh::create_from_faces(&[a, b])
    }

    #[test]
    fn build_and_count() {
        let m = quad();
        assert!(m.is_valid());
        assert_eq!(m.get_triangle_count(), 2);
        assert_eq!(m.get_vertex_count(), 6); // No welding.
        assert_eq!(m.triangles[0].indices, [0, 1, 2]);
        assert_eq!(m.triangles[1].indices, [3, 4, 5]);
        assert_eq!(m.triangles[0].surface_index, 0);
    }

    #[test]
    fn area_and_normals() {
        let m = quad();
        assert!((m.get_area() - 1.0).abs() < 1e-5);
        // Both triangles face +/-Z depending on winding; check they are unit.
        for t in &m.triangles {
            assert!(t.normal.is_normalized(), "{:?}", t.normal);
        }
    }

    #[test]
    fn aabb_covers_vertices() {
        let m = quad();
        let aabb = m.get_aabb();
        assert_eq!(aabb.position, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.size, Vector3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn faces_roundtrip() {
        let m = quad();
        let faces = m.get_faces();
        assert_eq!(faces.len(), 2);
        assert_eq!(faces[0].vertex[1], Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn empty_mesh() {
        let m = TriangleMesh::new();
        assert!(!m.is_valid());
        assert_eq!(m.get_area(), 0.0);
        assert_eq!(m.get_aabb(), Aabb::default());
        assert!(m.get_faces().is_empty());
    }

    #[test]
    fn normals_can_be_recomputed() {
        let mut m = quad();
        m.triangles[0].normal = Vector3::splat(9.0);
        m.recompute_normals();
        assert!(m.triangles[0].normal.is_normalized());
    }
}
