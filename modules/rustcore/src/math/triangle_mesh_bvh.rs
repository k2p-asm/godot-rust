//! BVH for `TriangleMesh`: median-split build plus ray/segment queries.
//!
//! The engine builds its BVH with `_create_bvh` (sorting by centroid on the
//! widest axis, storing `left`/`right`/`face_index`); this port does the same
//! median split over a flat node array. Queries are verified against brute
//! force over all triangles, which is what makes the structure trustworthy.
//!
//! Non-differential: `triangle_mesh.cpp` is a `GDCLASS` and cannot be linked in
//! the parity harness.

use crate::math::aabb::Aabb;
use crate::math::face3::Face3;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct BvhNode {
    pub aabb: Aabb,
    /// Children indices, or -1 for a leaf.
    pub left: i32,
    pub right: i32,
    /// Face index for leaves, -1 for inner nodes.
    pub face_index: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Bvh {
    pub nodes: Vec<BvhNode>,
    pub faces: Vec<Face3>,
}

// One face per leaf keeps traversal trivially correct; the engine packs up to a\n// few faces per leaf for cache locality.

impl Bvh {
    /// Builds over `faces`. Faces keep their original order; leaves store their
    /// index into `faces`.
    pub fn build(faces: Vec<Face3>) -> Self {
        let mut order: Vec<usize> = (0..faces.len()).collect();
        let mut bvh = Bvh { nodes: Vec::new(), faces };
        let total = order.len();
        if total > 0 {
            bvh.build_range(&mut order, 0, total);
        }
        bvh
    }

    fn face_aabb(f: &Face3) -> Aabb {
        f.get_aabb()
    }

    fn build_range(&mut self, order: &mut [usize], from: usize, len: usize) -> i32 {
        let mut aabb = Self::face_aabb(&self.faces[order[from]]);
        for k in from + 1..from + len {
            aabb = aabb.merge(&Self::face_aabb(&self.faces[order[k]]));
        }

        if len == 1 {
            self.nodes.push(BvhNode { aabb, left: -1, right: -1, face_index: order[from] as i32 });
            return (self.nodes.len() - 1) as i32;
        }
        // Split on the widest axis at the median of centroids.
        let size = aabb.size;
        let axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };
        // Precompute centroids so the sort closure does not borrow `self`
        // while `order` is borrowed mutably.
        let centers: Vec<f32> = self
            .faces
            .iter()
            .map(|f| f.get_aabb().get_center()[axis])
            .collect();
        order[from..from + len].sort_by(|a, b| {
            centers[*a].partial_cmp(&centers[*b]).unwrap_or(std::cmp::Ordering::Equal)
        });

        let half = len / 2;
        let node_idx = self.nodes.len();
        self.nodes.push(BvhNode { aabb, left: -1, right: -1, face_index: -1 });

        let left = self.build_range(order, from, half);
        let right = self.build_range(order, from + half, len - half);

        self.nodes[node_idx].left = left;
        self.nodes[node_idx].right = right;
        node_idx as i32
    }

    /// Segment query. Returns the closest hit as `(t, point, normal, face)`.
    pub fn intersect_segment(
        &self,
        from: Vector3,
        to: Vector3,
    ) -> Option<(f32, Vector3, Vector3, usize)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<(f32, Vector3, Vector3, usize)> = None;
        self.collect_segment(0, from, to, &mut best);
        best
    }

    fn collect_segment(
        &self,
        node: i32,
        from: Vector3,
        to: Vector3,
        best: &mut Option<(f32, Vector3, Vector3, usize)>,
    ) {
        if node < 0 {
            return;
        }
        let n = self.nodes[node as usize];

        // Cheap reject with the padded box.
        if n.aabb.intersects_segment(from, to).is_none() {
            return;
        }

        if n.face_index >= 0 {
            let face = &self.faces[n.face_index as usize];
            if let Some(hit) = crate::math::geometry_3d::segment_intersects_triangle(
                from,
                to,
                face.vertex[0],
                face.vertex[1],
                face.vertex[2],
            ) {
                let t = if (to - from).length_squared() > 0.0 {
                    (hit - from).length() / (to - from).length()
                } else {
                    0.0
                };
                if best.map_or(true, |(bt, _, _, _)| t < bt) {
                    *best = Some((t, hit, face.get_plane().normal, n.face_index as usize));
                }
            }
            return;
        }

        self.collect_segment(n.left, from, to, best);
        self.collect_segment(n.right, from, to, best);
    }

    /// Brute-force reference, used by the tests to prove the BVH agrees.
    pub fn intersect_segment_brute(
        &self,
        from: Vector3,
        to: Vector3,
    ) -> Option<(f32, Vector3, Vector3, usize)> {
        let mut best: Option<(f32, Vector3, Vector3, usize)> = None;
        for (i, face) in self.faces.iter().enumerate() {
            if let Some(hit) = crate::math::geometry_3d::segment_intersects_triangle(
                from,
                to,
                face.vertex[0],
                face.vertex[1],
                face.vertex[2],
            ) {
                let t = (hit - from).length() / (to - from).length();
                if best.map_or(true, |(bt, _, _, _)| t < bt) {
                    best = Some((t, hit, face.get_plane().normal, i));
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod bvh_tests {
    use super::*;

    /// Grid of triangles in the plane z = 0, spanning [0, n] x [0, n].
    fn plane_mesh(n: i32) -> Bvh {
        let mut faces = Vec::new();
        for y in 0..n {
            for x in 0..n {
                let p = |dx: f32, dy: f32| Vector3::new(x as f32 + dx, y as f32 + dy, 0.0);
                faces.push(Face3::new(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)));
                faces.push(Face3::new(p(0.0, 0.0), p(1.0, 1.0), p(0.0, 1.0)));
            }
        }
        Bvh::build(faces)
    }

    #[test]
    fn single_quad_is_hit() {
        let bvh = plane_mesh(1);
        assert_eq!(bvh.faces.len(), 2);
        let hit = bvh
            .intersect_segment(Vector3::new(0.25, 0.25, -1.0), Vector3::new(0.25, 0.25, 1.0))
            .expect("ray through the quad must hit");
        assert!((hit.1 - Vector3::new(0.25, 0.25, 0.0)).length() < 1e-5);
        assert!(hit.2.is_normalized());
    }

    #[test]
    fn miss_returns_none() {
        let bvh = plane_mesh(1);
        assert!(bvh
            .intersect_segment(Vector3::new(5.0, 5.0, -1.0), Vector3::new(5.0, 5.0, 1.0))
            .is_none());
    }

    #[test]
    fn bvh_agrees_with_brute_force() {
        let bvh = plane_mesh(4); // 32 triangles
        // Deterministic pseudo-random segments through and around the plane.
        let mut state = 12345u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state & 0xFFFF) as f32 / 65535.0 * 6.0 - 1.0
        };

        for _ in 0..200 {
            let from = Vector3::new(next(), next(), -1.0 - next().abs());
            let to = Vector3::new(next(), next(), 1.0 + next().abs());
            let a = bvh.intersect_segment(from, to);
            let b = bvh.intersect_segment_brute(from, to);
            match (a, b) {
                (None, None) => {}
                (Some(x), Some(y)) => {
                    assert!((x.1 - y.1).length() < 1e-4, "hit point mismatch {x:?} vs {y:?}");
                }
                (x, y) => panic!("bvh/brute disagree: {x:?} vs {y:?}"),
            }
        }
    }

    #[test]
    fn deepest_triangle_is_reachable() {
        let bvh = plane_mesh(5);
        // A segment straight down onto the last cell must still find it.
        let hit = bvh
            .intersect_segment(Vector3::new(4.5, 4.5, 2.0), Vector3::new(4.5, 4.5, -2.0))
            .expect("hit in the far cell");
        assert!((hit.1.x - 4.5).abs() < 1e-5 && (hit.1.y - 4.5).abs() < 1e-5);
    }
}
