//! `core/math/triangle_mesh.*`: the BVH-accelerated triangle soup.
//!
//! Ported from `triangle_mesh.cpp`:
//!
//! - `create` snaps vertices to `0.0001`, dedupes them, stores per-triangle
//!   normals/surface indices and builds the median-split BVH over the triangle
//!   AABBs (the engine's `nth_element` partition is replaced by a full sort by
//!   centroid on the longest axis — same split rule, different tie order);
//! - `intersect_segment` / `intersect_ray` walk the BVH, test each leaf triangle
//!   with `Face3`, keep the closest hit and flip the normal to face the ray;
//! - `inside_convex_shape` rejects any triangle vertex above a clipping plane
//!   (the engine's AABB/convex SAT early-outs are omitted, which cannot change
//!   the result for vertices inside the planes);
//! - `get_faces`, `get_triangles`, `get_vertices`, `get_indices`, `is_valid`.
//!
//! The traversal uses recursion instead of the engine's encoded stack.

use std::collections::HashMap;

use crate::math::aabb::Aabb;
use crate::math::face3::Face3;
use crate::math::plane::Plane;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug)]
pub struct Triangle {
    pub normal: Vector3,
    pub indices: [usize; 3],
    pub surface_index: i32,
}

#[derive(Clone, Debug)]
pub struct TriangleBvh {
    pub aabb: Aabb,
    pub left: i32,
    pub right: i32,
    /// `>= 0` for leaves.
    pub face_index: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TriangleMesh {
    triangles: Vec<Triangle>,
    vertices: Vec<Vector3>,
    bvh: Vec<TriangleBvh>,
    max_depth: usize,
    valid: bool,
}

/// The intersection result (the engine's out-parameters).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangleHit {
    pub point: Vector3,
    pub normal: Vector3,
    pub surface_index: i32,
    pub face_index: i32,
}

impl TriangleMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// `create`; false where the engine has `ERR_FAIL_COND` (and leaves the mesh
    /// invalid).
    pub fn create(&mut self, faces: &[Vector3], surface_indices: &[i32]) -> bool {
        self.valid = false;
        if !surface_indices.is_empty() && surface_indices.len() != faces.len() {
            return false;
        }
        let count = faces.len();
        if count == 0 || count % 3 != 0 {
            return false;
        }
        let triangle_count = count / 3;

        let mut snap_index: HashMap<(i64, i64, i64), usize> = HashMap::new();
        let mut vertices: Vec<Vector3> = Vec::new();
        let mut triangles: Vec<Triangle> = Vec::with_capacity(triangle_count);
        let mut bvh: Vec<TriangleBvh> = Vec::with_capacity(triangle_count * 3);

        for i in 0..triangle_count {
            let raw = [faces[i * 3], faces[i * 3 + 1], faces[i * 3 + 2]];
            let mut snapped = [Vector3::ZERO; 3];
            let mut indices = [0usize; 3];
            let mut aabb: Option<Aabb> = None;
            for j in 0..3 {
                let value = raw[j].snappedf(0.0001);
                snapped[j] = value;
                let key = (
                    (value.x * 10000.0).round() as i64,
                    (value.y * 10000.0).round() as i64,
                    (value.z * 10000.0).round() as i64,
                );
                let index = match snap_index.get(&key) {
                    Some(index) => *index,
                    None => {
                        let index = vertices.len();
                        vertices.push(value);
                        snap_index.insert(key, index);
                        index
                    }
                };
                indices[j] = index;
                aabb = Some(match aabb {
                    None => Aabb::new(value, Vector3::ZERO),
                    Some(mut aabb) => {
                        aabb.expand_to(value);
                        aabb
                    }
                });
            }

            let plane = Face3::new(raw[0], raw[1], raw[2]).get_plane();
            triangles.push(Triangle {
                normal: plane.normal,
                indices,
                surface_index: surface_indices.get(i).copied().unwrap_or(0),
            });
            bvh.push(TriangleBvh {
                aabb: aabb.expect("three vertices"),
                left: -1,
                right: -1,
                face_index: i as i32,
            });
        }

        let mut order: Vec<i32> = (0..triangle_count as i32).collect();
        let mut max_depth = 0;
        self.build_bvh(&mut bvh, &mut order, 1, &mut max_depth);

        self.triangles = triangles;
        self.vertices = vertices;
        self.bvh = bvh;
        self.max_depth = max_depth;
        self.valid = true;
        true
    }

    /// `_create_bvh` over node indices; internal nodes are appended.
    fn build_bvh(
        &self,
        bvh: &mut Vec<TriangleBvh>,
        order: &mut [i32],
        depth: usize,
        max_depth: &mut usize,
    ) -> i32 {
        if depth > *max_depth {
            *max_depth = depth;
        }
        if order.len() == 1 {
            return order[0];
        }
        if order.is_empty() {
            return -1;
        }

        let mut aabb = bvh[order[0] as usize].aabb;
        for index in &order[1..] {
            aabb.merge_with(&bvh[*index as usize].aabb);
        }
        let axis = aabb.get_longest_axis_index();
        order.sort_by(|a, b| {
            let ca = bvh[*a as usize].aabb.get_center();
            let cb = bvh[*b as usize].aabb.get_center();
            let (va, vb) = match axis {
                0 => (ca.x, cb.x),
                1 => (ca.y, cb.y),
                _ => (ca.z, cb.z),
            };
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = order.len() / 2;
        let (left_order, right_order) = order.split_at_mut(mid);
        let left = self.build_bvh(bvh, left_order, depth + 1, max_depth);
        let right = self.build_bvh(bvh, right_order, depth + 1, max_depth);

        let index = bvh.len();
        bvh.push(TriangleBvh {
            aabb,
            left,
            right,
            face_index: -1,
        });
        index as i32
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn get_vertices(&self) -> &[Vector3] {
        &self.vertices
    }

    pub fn get_faces(&self) -> Vec<Face3> {
        self.triangles
            .iter()
            .map(|triangle| {
                Face3::new(
                    self.vertices[triangle.indices[0]],
                    self.vertices[triangle.indices[1]],
                    self.vertices[triangle.indices[2]],
                )
            })
            .collect()
    }

    /// `get_indices`: three vertex indices per triangle.
    pub fn get_indices(&self) -> Vec<i32> {
        let mut indices = Vec::with_capacity(self.triangles.len() * 3);
        for triangle in &self.triangles {
            indices.push(triangle.indices[0] as i32);
            indices.push(triangle.indices[1] as i32);
            indices.push(triangle.indices[2] as i32);
        }
        indices
    }

    fn visit_segment(
        &self,
        node: i32,
        begin: Vector3,
        end: Vector3,
        direction: Vector3,
        best: &mut Option<TriangleHit>,
        best_distance: &mut f32,
    ) {
        if node < 0 {
            return;
        }
        let node = &self.bvh[node as usize];
        if node.aabb.intersects_segment(begin, end).is_none() {
            return;
        }
        if node.face_index >= 0 {
            let triangle = &self.triangles[node.face_index as usize];
            let face = Face3::new(
                self.vertices[triangle.indices[0]],
                self.vertices[triangle.indices[1]],
                self.vertices[triangle.indices[2]],
            );
            if let Some(point) = face.intersects_segment(begin, end) {
                let distance = direction.dot(point);
                if distance < *best_distance {
                    *best_distance = distance;
                    *best = Some(TriangleHit {
                        point,
                        normal: face.get_plane().normal,
                        surface_index: triangle.surface_index,
                        face_index: node.face_index,
                    });
                }
            }
            return;
        }
        self.visit_segment(node.left, begin, end, direction, best, best_distance);
        self.visit_segment(node.right, begin, end, direction, best, best_distance);
    }

    fn visit_ray(
        &self,
        node: i32,
        begin: Vector3,
        direction: Vector3,
        best: &mut Option<TriangleHit>,
        best_distance: &mut f32,
    ) {
        if node < 0 {
            return;
        }
        let node = &self.bvh[node as usize];
        if !node.aabb.intersects_ray(begin, direction) {
            return;
        }
        if node.face_index >= 0 {
            let triangle = &self.triangles[node.face_index as usize];
            let face = Face3::new(
                self.vertices[triangle.indices[0]],
                self.vertices[triangle.indices[1]],
                self.vertices[triangle.indices[2]],
            );
            if let Some(point) = face.intersects_ray(begin, direction) {
                let distance = direction.dot(point);
                if distance < *best_distance {
                    *best_distance = distance;
                    *best = Some(TriangleHit {
                        point,
                        normal: face.get_plane().normal,
                        surface_index: triangle.surface_index,
                        face_index: node.face_index,
                    });
                }
            }
            return;
        }
        self.visit_ray(node.left, begin, direction, best, best_distance);
        self.visit_ray(node.right, begin, direction, best, best_distance);
    }

    /// `intersect_segment`: the closest triangle hit, normal facing the segment.
    pub fn intersect_segment(&self, begin: Vector3, end: Vector3) -> Option<TriangleHit> {
        if !self.valid {
            return None;
        }
        let direction = (end - begin).normalized();
        let mut best = None;
        let mut best_distance = 1e10;
        let root = self.bvh.len() as i32 - 1;
        self.visit_segment(root, begin, end, direction, &mut best, &mut best_distance);
        best.map(|mut hit| {
            if direction.dot(hit.normal) > 0.0 {
                hit.normal = -hit.normal;
            }
            hit
        })
    }

    /// `intersect_ray`: the closest triangle hit, normal facing the ray.
    pub fn intersect_ray(&self, begin: Vector3, direction: Vector3) -> Option<TriangleHit> {
        if !self.valid {
            return None;
        }
        let mut best = None;
        let mut best_distance = 1e20;
        let root = self.bvh.len() as i32 - 1;
        self.visit_ray(root, begin, direction, &mut best, &mut best_distance);
        best.map(|mut hit| {
            if direction.dot(hit.normal) > 0.0 {
                hit.normal = -hit.normal;
            }
            hit
        })
    }

    /// `inside_convex_shape`: every triangle vertex must be below every plane.
    pub fn inside_convex_shape(&self, planes: &[Plane], scale: Vector3) -> bool {
        if !self.valid {
            return false;
        }
        for vertex in &self.vertices {
            let point = Vector3::new(vertex.x * scale.x, vertex.y * scale.y, vertex.z * scale.z);
            for plane in planes {
                if plane.is_point_over(point) {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_max_depth(&self) -> usize {
        self.max_depth
    }

    pub fn get_bvh_node_count(&self) -> usize {
        self.bvh.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad() -> TriangleMesh {
        // Two triangles in the XY plane, counter-clockwise when seen from +Z.
        let faces = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        let mut mesh = TriangleMesh::new();
        // The engine compares the surface-index array against the *vertex* count;
        // only the first entries (one per triangle) are read.
        assert!(mesh.create(&faces, &[0, 1, 0, 0, 0, 0]));
        mesh
    }

    #[test]
    fn invalid_inputs() {
        let mut mesh = TriangleMesh::new();
        assert!(!mesh.is_valid());
        assert!(!mesh.create(&[], &[]));
        assert!(!mesh.create(&[Vector3::ZERO, Vector3::ONE], &[]));
        assert!(!mesh.create(
            &[Vector3::ZERO, Vector3::ONE, Vector3::new(1.0, 1.0, 0.0)],
            &[0, 1]
        ));
        assert!(mesh.create(
            &[Vector3::ZERO, Vector3::ONE, Vector3::new(1.0, 1.0, 0.0)],
            &[]
        ));
        assert_eq!(mesh.get_triangles().len(), 1);
    }

    #[test]
    fn vertices_are_snapped_and_deduped() {
        let mut mesh = TriangleMesh::new();
        assert!(mesh.create(
            &[
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                // Duplicate of the first triangle's vertices, offset below the
                // snap threshold.
                Vector3::new(0.00001, 0.0, 0.0),
                Vector3::new(0.0, 1.00001, 0.0),
                Vector3::new(1.0, 0.00001, 0.0),
            ],
            &[],
        ));
        assert_eq!(mesh.get_vertices().len(), 3);
        assert_eq!(mesh.get_triangles().len(), 2);
        assert_eq!(mesh.get_indices().len(), 6);
        assert_eq!(mesh.get_faces().len(), 2);
    }

    #[test]
    fn segment_and_ray_hits() {
        let mesh = quad();
        assert!(mesh.is_valid());
        assert_eq!(mesh.get_triangles().len(), 2);
        assert!(mesh.get_max_depth() >= 1);
        assert_eq!(mesh.get_bvh_node_count(), 3, "two leaves and one root");

        // Surface 1 is hit: the second triangle covers (1,0)-(0,1)-(1,1).
        let hit = mesh
            .intersect_segment(
                Vector3::new(0.75, 0.75, 1.0),
                Vector3::new(0.75, 0.75, -1.0),
            )
            .expect("segment hits the quad");
        assert!((hit.point.z).abs() < 1e-5);
        assert_eq!(hit.surface_index, 1);
        assert!(hit.face_index >= 0);
        // The normal is flipped to face the segment start (upwards).
        assert!(hit.normal.z > 0.0);

        let ray = mesh
            .intersect_ray(
                Vector3::new(0.25, 0.25, 3.0),
                Vector3::new(0.0, 0.0, -1.0),
            )
            .expect("ray hits the first triangle");
        assert_eq!(ray.surface_index, 0);
        assert!(ray.normal.z > 0.0);
        assert!((ray.point.z).abs() < 1e-5);

        // A segment above the quad never reports a hit.
        assert!(mesh
            .intersect_segment(
                Vector3::new(0.5, 0.5, 1.0),
                Vector3::new(0.5, 0.5, 2.0)
            )
            .is_none());
        // Nor does a ray pointing away.
        assert!(mesh
            .intersect_ray(
                Vector3::new(0.5, 0.5, 1.0),
                Vector3::new(0.0, 0.0, 1.0)
            )
            .is_none());
    }

    #[test]
    fn inside_convex_shape_uses_every_vertex() {
        let mesh = quad();
        let box_planes = crate::math::geometry_3d::build_box_planes(Vector3::new(2.0, 2.0, 2.0));
        assert!(mesh.inside_convex_shape(&box_planes, Vector3::ONE));

        // A box that only covers half of the quad rejects it.
        let mut shifted = Vec::new();
        for plane in crate::math::geometry_3d::build_box_planes(Vector3::new(2.0, 2.0, 2.0)) {
            if plane.normal.x > 0.5 {
                shifted.push(Plane::new(1.0, 0.0, 0.0, 0.5));
            } else {
                shifted.push(plane);
            }
        }
        assert!(!mesh.inside_convex_shape(&shifted, Vector3::ONE));

        // Scaling moves the vertices out of a unit box.
        let unit = crate::math::geometry_3d::build_box_planes(Vector3::new(1.0, 1.0, 1.0));
        assert!(mesh.inside_convex_shape(&unit, Vector3::new(0.5, 0.5, 0.5)));

        // An invalid mesh reports false.
        let empty = TriangleMesh::new();
        assert!(!empty.inside_convex_shape(&box_planes, Vector3::ONE));
    }
}
