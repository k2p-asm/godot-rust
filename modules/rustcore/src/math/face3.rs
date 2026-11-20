//! Port of `core/math/face3.h` + `face3.cpp` (triangle primitive).
//!
//! Ported: `is_degenerate`, `get_plane`, `get_area`, `get_aabb`, `split_by_plane`
//! and `get_closest_point_to`.
//! Pending: ray/segment intersection (needs `Geometry3D`), `intersects_aabb`
//! (SAT with edges) and `get_support`/`project_range`.

use crate::math::aabb::Aabb;
use crate::math::funcs;
use crate::math::plane::Plane;
use crate::math::vector3::Vector3;

pub enum Side {
    Over,
    Under,
    Spanning,
    Coplanar,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Face3 {
    pub vertex: [Vector3; 3],
}

impl Face3 {
    #[inline]
    pub const fn new(v1: Vector3, v2: Vector3, v3: Vector3) -> Self {
        Self { vertex: [v1, v2, v3] }
    }

    #[inline]
    pub fn is_degenerate(&self) -> bool {
        let normal = (self.vertex[0] - self.vertex[1]).cross(self.vertex[0] - self.vertex[2]);
        normal.length_squared() < funcs::CMP_EPSILON2 as f32
    }

    /// Plane through the three vertices (clockwise winding by default).
    #[inline]
    pub fn get_plane(&self) -> Plane {
        Plane::from_points(self.vertex[0], self.vertex[1], self.vertex[2])
    }

    #[inline]
    pub fn get_area(&self) -> f32 {
        (self.vertex[0] - self.vertex[1])
            .cross(self.vertex[0] - self.vertex[2])
            .length()
            * 0.5
    }

    #[inline]
    pub fn get_aabb(&self) -> Aabb {
        let mut aabb = Aabb::new(self.vertex[0], Vector3::ZERO);
        aabb.expand_to(self.vertex[1]);
        aabb.expand_to(self.vertex[2]);
        aabb
    }

    /// Splits the face with a plane, returning up to 4 resulting faces and
    /// whether each one is on the "over" side.
    pub fn split_by_plane(&self, plane: &Plane) -> Vec<(Face3, bool)> {
        if self.is_degenerate() {
            return Vec::new();
        }

        let mut above: Vec<Vector3> = Vec::with_capacity(4);
        let mut below: Vec<Vector3> = Vec::with_capacity(4);

        for i in 0..3 {
            if plane.has_point(self.vertex[i], funcs::CMP_EPSILON as f32) {
                // Point lies on the plane: goes to both sides.
                above.push(self.vertex[i]);
                below.push(self.vertex[i]);
            } else {
                if plane.is_point_over(self.vertex[i]) {
                    above.push(self.vertex[i]);
                } else {
                    below.push(self.vertex[i]);
                }

                // Intersection with the next edge, if any.
                if let Some(inters) =
                    plane.intersects_segment(self.vertex[i], self.vertex[(i + 1) % 3])
                {
                    above.push(inters);
                    below.push(inters);
                }
            }
        }

        if above.len() >= 4 && below.len() >= 4 {
            return Vec::new(); // Algorithm invariant violated.
        }

        let mut out = Vec::new();

        if above.len() >= 3 {
            out.push((Face3::new(above[0], above[1], above[2]), true));
            if above.len() == 4 {
                out.push((Face3::new(above[2], above[3], above[0]), true));
            }
        }

        if below.len() >= 3 {
            out.push((Face3::new(below[0], below[1], below[2]), false));
            if below.len() == 4 {
                out.push((Face3::new(below[2], below[3], below[0]), false));
            }
        }

        out
    }

    /// Ray/triangle intersection (`Face3::intersects_ray`).
    #[inline]
    pub fn intersects_ray(&self, from: Vector3, dir: Vector3) -> Option<Vector3> {
        crate::math::geometry_3d::ray_intersects_triangle(from, dir, self.vertex[0], self.vertex[1], self.vertex[2])
    }

    /// Segment/triangle intersection (`Face3::intersects_segment`).
    #[inline]
    pub fn intersects_segment(&self, from: Vector3, to: Vector3) -> Option<Vector3> {
        crate::math::geometry_3d::segment_intersects_triangle(from, to, self.vertex[0], self.vertex[1], self.vertex[2])
    }

    /// Closest point on the triangle to `p_point` (Ericson's barycentric clamp,
    /// identical to the engine's `get_closest_point_to`).
    pub fn get_closest_point_to(&self, p_point: Vector3) -> Vector3 {
        let edge0 = self.vertex[1] - self.vertex[0];
        let edge1 = self.vertex[2] - self.vertex[0];
        let v0 = self.vertex[0] - p_point;

        let a = edge0.dot(edge0);
        let b = edge0.dot(edge1);
        let c = edge1.dot(edge1);
        let d = edge0.dot(v0);
        let e = edge1.dot(v0);

        let det = a * c - b * b;
        let mut s = b * e - c * d;
        let mut t = b * d - a * e;

        if s + t < det {
            if s < 0.0 {
                if t < 0.0 {
                    if d < 0.0 {
                        s = funcs::clamp_f32(-d / a, 0.0, 1.0);
                        t = 0.0;
                    } else {
                        s = 0.0;
                        t = funcs::clamp_f32(-e / c, 0.0, 1.0);
                    }
                } else {
                    s = 0.0;
                    t = funcs::clamp_f32(-e / c, 0.0, 1.0);
                }
            } else if t < 0.0 {
                s = funcs::clamp_f32(-d / a, 0.0, 1.0);
                t = 0.0;
            } else {
                let inv_det = 1.0 / det;
                s *= inv_det;
                t *= inv_det;
            }
        } else if s < 0.0 {
            let tmp0 = b + d;
            let tmp1 = c + e;
            if tmp1 > tmp0 {
                let numer = tmp1 - tmp0;
                let denom = a - 2.0 * b + c;
                s = funcs::clamp_f32(numer / denom, 0.0, 1.0);
                t = 1.0 - s;
            } else {
                t = funcs::clamp_f32(-e / c, 0.0, 1.0);
                s = 0.0;
            }
        } else if t < 0.0 {
            if a + d > b + e {
                let numer = c + e - b - d;
                let denom = a - 2.0 * b + c;
                s = funcs::clamp_f32(numer / denom, 0.0, 1.0);
                t = 1.0 - s;
            } else {
                s = funcs::clamp_f32(-d / a, 0.0, 1.0);
                t = 0.0;
            }
        } else {
            let numer = c + e - b - d;
            let denom = a - 2.0 * b + c;
            s = funcs::clamp_f32(numer / denom, 0.0, 1.0);
            t = 1.0 - s;
        }

        self.vertex[0] + edge0 * s + edge1 * t
    }

    /// Support points of the triangle for `normal` in the space of `transform`
    /// (`Face3::get_support`): the whole face, an edge, or a single vertex,
    /// depending on how aligned the normal is.
    pub fn get_support(
        &self,
        normal: Vector3,
        transform: &crate::math::transform_3d::Transform3D,
        max: usize,
    ) -> Vec<Vector3> {
        const FACE_SUPPORT_THRESHOLD: f64 = 0.98;
        const EDGE_SUPPORT_THRESHOLD: f64 = 0.05;

        if max == 0 {
            return Vec::new();
        }

        let n = transform.basis.xform_inv(normal);

        // Face as support.
        if self.get_plane().normal.dot(n) as f64 > FACE_SUPPORT_THRESHOLD {
            let count = 3.min(max);
            return (0..count).map(|i| transform.xform(self.vertex[i])).collect();
        }

        // Find the support vertex.
        let mut vert_support_idx = 0usize;
        let mut support_max = 0.0f32;
        for i in 0..3 {
            let d = n.dot(self.vertex[i]);
            if i == 0 || d > support_max {
                support_max = d;
                vert_support_idx = i;
            }
        }

        // Edges as support.
        for i in 0..3 {
            // NOTE: the engine compares `i + 1` without wrapping modulo 3, so
            // for i = 2 the comparison is 3 != idx and the edge is skipped.
            // Wrapping here would change which edges count as support.
            if i != vert_support_idx && i + 1 != vert_support_idx {
                continue;
            }

            let mut dot = (self.vertex[i] - self.vertex[(i + 1) % 3]).normalized().dot(n);
            dot = dot.abs();

            if (dot as f64) < EDGE_SUPPORT_THRESHOLD {
                let count = 2.min(max);
                return (0..count)
                    .map(|j| transform.xform(self.vertex[(j + i) % 3]))
                    .collect();
            }
        }

        vec![transform.xform(self.vertex[vert_support_idx])]
    }
    /// Projection range of the triangle onto `normal` (identity transform).
    pub fn project_range(&self, normal: Vector3) -> (f32, f32) {
        let mut min = 0.0f32;
        let mut max = 0.0f32;
        for i in 0..3 {
            let d = normal.dot(self.vertex[i]);
            if i == 0 || d > max {
                max = d;
            }
            if i == 0 || d < min {
                min = d;
            }
        }
        (min, max)
    }

    /// Projection range of the triangle onto `normal`, with the triangle
    /// transformed (`Face3::project_range` with a `Transform3D`).
    pub fn project_range_xf(
        &self,
        normal: Vector3,
        transform: &crate::math::transform_3d::Transform3D,
    ) -> (f32, f32) {
        let mut min = 0.0f32;
        let mut max = 0.0f32;

        for i in 0..3 {
            let v = transform.xform(self.vertex[i]);
            let d = normal.dot(v);

            if i == 0 || d > max {
                max = d;
            }
            if i == 0 || d < min {
                min = d;
            }
        }

        (min, max)
    }
    /// Triangle vs AABB by separating axes: face normal, the three box axes and
    /// all 12 box edges crossed with the 3 triangle edges.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        // Test plane.
        if !aabb.intersects_plane(&self.get_plane()) {
            return false;
        }

        // Box axes.
        for axis in 0..3 {
            let aabb_min = aabb.position[axis];
            let aabb_max = aabb.position[axis] + aabb.size[axis];
            let mut tri_min = self.vertex[0][axis];
            let mut tri_max = self.vertex[0][axis];
            for i in 1..3 {
                if self.vertex[i][axis] > tri_max {
                    tri_max = self.vertex[i][axis];
                }
                if self.vertex[i][axis] < tri_min {
                    tri_min = self.vertex[i][axis];
                }
            }
            if tri_max < aabb_min || aabb_max < tri_min {
                return false;
            }
        }

        // All edge pairs.
        let edge_norms = [
            self.vertex[0] - self.vertex[1],
            self.vertex[1] - self.vertex[2],
            self.vertex[2] - self.vertex[0],
        ];

        for i in 0..12 {
            let (from, to) = aabb.get_edge(i);
            let e1 = from - to;
            for e2 in edge_norms {
                let mut axis = e1.cross(e2);
                if axis.length_squared() < 0.0001 {
                    continue; // Coplanar.
                }
                axis.normalize();

                let (min_a, max_a) = aabb.project_range_in_plane(&Plane::new(axis.x, axis.y, axis.z, 0.0));
                let (min_b, max_b) = self.project_range(axis);

                if max_a < min_b || max_b < min_a {
                    return false;
                }
            }
        }

        true
    }
    #[allow(dead_code)]
    fn closest_on_edges(&self, p: Vector3) -> Vector3 {
        let mut best = self.vertex[0];
        let mut best_dist = f32::INFINITY;
        for i in 0..3 {
            let a = self.vertex[i];
            let b = self.vertex[(i + 1) % 3];
            let ab = b - a;
            let t = if ab.length_squared() > 0.0 {
                ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let cand = a + ab * t;
            let d = cand.distance_squared_to(p);
            if d < best_dist {
                best_dist = d;
                best = cand;
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_area_and_aabb() {
        let f = Face3::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 2.0, 0.0),
        );
        assert!(!f.is_degenerate());
        assert!((f.get_area() - 2.0).abs() < 1e-5);
        let aabb = f.get_aabb();
        assert_eq!(aabb.position, Vector3::ZERO);
        assert_eq!(aabb.size, Vector3::new(2.0, 2.0, 0.0));
        assert!(f.get_plane().normal.is_normalized());
    }

    #[test]
    fn degenerate_face() {
        let f = Face3::new(Vector3::ZERO, Vector3::new(1.0, 0.0, 0.0), Vector3::new(2.0, 0.0, 0.0));
        assert!(f.is_degenerate());
        assert!(f.split_by_plane(&Plane::new(0.0, 1.0, 0.0, 0.0)).is_empty());
    }

    #[test]
    fn split_produces_two_pieces() {
        let f = Face3::new(
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(2.0, -1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let plane = Plane::new(0.0, 1.0, 0.0, 0.0);
        // The plane crosses two edges and one vertex sits above: the lower
        // half becomes two triangles, the upper half one, so 3 pieces total.
        let parts = f.split_by_plane(&plane);
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().any(|(_, over)| *over));
        assert!(parts.iter().any(|(_, over)| !*over));
        // The two pieces must cover the original area.
        let total: f32 = parts.iter().map(|(f, _)| f.get_area()).sum();
        assert!((total - f.get_area()).abs() < 1e-4, "{total} vs {}", f.get_area());
    }
}
