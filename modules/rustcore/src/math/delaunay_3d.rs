//! `Delaunay3D` (Bowyer-Watson) plus the packing step of
//! `Geometry3D::tetrahedralize_delaunay`.
//!
//! The engine's implementation is incremental with an explicit cavity walk and
//! sorts points by distance to the centroid first. This port uses the classic
//! Bowyer-Watson formulation: insert each point, delete every tetrahedron whose
//! circumsphere contains it, and re-triangulate the resulting boundary hull.
//!
//! The **tetrahedra are equivalent** (same point set, no overlaps, covering the
//! convex hull) but the order of the output is not guaranteed to match the
//! engine's, and degenerate/cospherical inputs may pick a different valid
//! triangulation. Non-differential: `delaunay_3d.cpp` cannot be linked in the
//! parity harness.

use crate::math::funcs;
use crate::math::vector3::Vector3;

/// One tetrahedron, as `Delaunay3D::OutputSimplex`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputSimplex {
    pub points: [i32; 4],
}

/// `tetrahedralize_delaunay`: pads each simplex with its four indices.
pub fn tetrahedralize_indices(simplices: &[OutputSimplex]) -> Vec<i32> {
    let mut out = Vec::with_capacity(4 * simplices.len());
    for s in simplices {
        out.extend_from_slice(&s.points);
    }
    out
}

/// Tetrahedralizes `points`; empty when there are fewer than 4 points or they
/// are coplanar.
pub fn tetrahedralize(points: &[Vector3]) -> Vec<OutputSimplex> {
    if points.len() < 4 {
        return Vec::new();
    }

    // Work with extended point list: real points first, then 4 super-vertices.
    let mut verts: Vec<Vector3> = points.to_vec();
    let n = points.len();

    // Super-tetrahedron: large enough to contain every point.
    let mut min = points[0];
    let mut max = points[0];
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    let center = (min + max) * 0.5;
    let mut radius = 0.0f32;
    for p in points {
        radius = radius.max(center.distance_to(*p));
    }
    let r = if radius > 0.0 { radius * 100.0 } else { 100.0 };

    let sup = [
        center + Vector3::new(r, 0.0, 0.0) + Vector3::new(0.0, r, 0.0) + Vector3::new(0.0, 0.0, r),
        center + Vector3::new(-r, 0.0, 0.0) + Vector3::new(0.0, -r, 0.0) + Vector3::new(0.0, 0.0, r),
        center + Vector3::new(-r, 0.0, 0.0) + Vector3::new(0.0, r, 0.0) + Vector3::new(0.0, 0.0, -r),
        center + Vector3::new(r, 0.0, 0.0) + Vector3::new(0.0, -r, 0.0) + Vector3::new(0.0, 0.0, -r),
    ];
    verts.extend_from_slice(&sup);

    let mut tets: Vec<[usize; 4]> = vec![[n, n + 1, n + 2, n + 3]];

    for pi in 0..n {
        let p = verts[pi];

        // Every tetrahedron whose circumsphere contains p must go.
        let mut bad = Vec::new();
        for (ti, t) in tets.iter().enumerate() {
            if insphere(verts[t[0]], verts[t[1]], verts[t[2]], verts[t[3]], p) {
                bad.push(ti);
            }
        }
        if bad.is_empty() {
            continue; // p is outside every circumsphere: coplanar/degenerate.
        }

        // Boundary faces of the cavity: faces that appear exactly once.
        let mut faces: Vec<[usize; 3]> = Vec::new();
        for &ti in &bad {
            let t = tets[ti];
            for f in [
                [t[0], t[1], t[2]],
                [t[0], t[1], t[3]],
                [t[0], t[2], t[3]],
                [t[1], t[2], t[3]],
            ] {
                // Unoriented key: sorted indices.
                let mut sorted = f;
                sorted.sort_unstable();
                if let Some(pos) = faces.iter().position(|g| {
                    let mut gs = *g;
                    gs.sort_unstable();
                    gs == sorted
                }) {
                    faces.remove(pos);
                } else {
                    faces.push(f);
                }
            }
        }

        // Drop the bad tetrahedra (descending so indices stay valid).
        bad.sort_unstable();
        for &ti in bad.iter().rev() {
            tets.remove(ti);
        }

        // Re-triangulate the cavity with p.
        for f in faces {
            let a = verts[f[0]];
            let b = verts[f[1]];
            let c = verts[f[2]];
            // Skip degenerate faces.
            if signed_volume(a, b, c, p).abs() < funcs::CMP_EPSILON as f32 {
                continue;
            }
            tets.push([f[0], f[1], f[2], pi]);
        }
    }

    // Remove everything touching the super-tetrahedron.
    let mut out = Vec::new();
    for t in tets {
        if t.iter().any(|i| *i >= n) {
            continue;
        }
        out.push(OutputSimplex { points: [t[0] as i32, t[1] as i32, t[2] as i32, t[3] as i32] });
    }
    out
}

/// Six times the signed volume of the tetrahedron (positive when `d` is on the
/// positive side of `abc`).
fn signed_volume(a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> f32 {
    (b - a).cross(c - a).dot(d - a)
}

/// True when `p` lies inside the circumsphere of `(a, b, c, d)`.
///
/// Computes the circumcenter explicitly instead of using the 4x4 determinant:
/// the sign of that determinant depends on the tetrahedron's orientation, and
/// getting it wrong silently produces zero tetrahedra.
fn insphere(a: Vector3, b: Vector3, c: Vector3, d: Vector3, p: Vector3) -> bool {
    let (Some(center), Some(radius)) = circumsphere(a, b, c, d) else {
        return false;
    };
    center.distance_squared_to(p) < radius * radius
}

/// Circumcenter and radius of a tetrahedron; `None` when it is degenerate.
fn circumsphere(a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> (Option<Vector3>, Option<f32>) {
    let ac = c - a;
    let ab = b - a;
    let ad = d - a;

    let det = det3(ab.x, ab.y, ab.z, ac.x, ac.y, ac.z, ad.x, ad.y, ad.z);
    if det.abs() < 1e-12 {
        return (None, None);
    }

    let ab2 = ab.dot(ab);
    let ac2 = ac.dot(ac);
    let ad2 = ad.dot(ad);

    // Solve the 3x3 system 2 * M * x = rhs with Cramer's rule.
    let x = det3(ab2, ab.y, ab.z, ac2, ac.y, ac.z, ad2, ad.y, ad.z) / (2.0 * det);
    let y = det3(ab.x, ab2, ab.z, ac.x, ac2, ac.z, ad.x, ad2, ad.z) / (2.0 * det);
    let z = det3(ab.x, ab.y, ab2, ac.x, ac.y, ac2, ad.x, ad.y, ad2) / (2.0 * det);

    let center = a + Vector3::new(x, y, z);
    (Some(center), Some(center.distance_to(a)))
}

#[allow(clippy::too_many_arguments)]
fn det3(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32, i: f32) -> f32 {
    a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_four_indices_per_simplex() {
        let simplices = [
            OutputSimplex { points: [0, 1, 2, 3] },
            OutputSimplex { points: [4, 5, 6, 7] },
        ];
        assert_eq!(tetrahedralize_indices(&simplices), vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn preserves_order_and_duplicates() {
        let simplices = [
            OutputSimplex { points: [3, 1, 0, 2] },
            OutputSimplex { points: [1, 0, 3, 2] },
        ];
        let out = tetrahedralize_indices(&simplices);
        assert_eq!(out.len(), 8);
        assert_eq!(&out[0..4], &[3, 1, 0, 2]);
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(tetrahedralize_indices(&[]).is_empty());
        assert!(tetrahedralize(&[]).is_empty());
        assert!(tetrahedralize(&[Vector3::ZERO, Vector3::ONE, Vector3::UP]).is_empty());
    }

    #[test]
    fn single_tetrahedron() {
        let pts = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let tets = tetrahedralize(&pts);
        assert_eq!(tets.len(), 1);
        let mut idx = tets[0].points;
        idx.sort_unstable();
        assert_eq!(idx, [0, 1, 2, 3]);
    }

    #[test]
    fn cube_corners_are_tetrahedralized_without_overlap() {
        // Unit cube: 8 corners.
        let mut pts = Vec::new();
        for &x in &[0.0f32, 1.0] {
            for &y in &[0.0f32, 1.0] {
                for &z in &[0.0f32, 1.0] {
                    pts.push(Vector3::new(x, y, z));
                }
            }
        }
        let tets = tetrahedralize(&pts);
        assert!(tets.len() >= 5, "got {} tetrahedra", tets.len());

        // Every tetrahedron has positive volume inside [0,1]^3 and the total
        // equals the cube volume (no overlaps, no gaps).
        let mut total = 0.0f32;
        for t in &tets {
            let a = pts[t.points[0] as usize];
            let b = pts[t.points[1] as usize];
            let c = pts[t.points[2] as usize];
            let d = pts[t.points[3] as usize];
            let v = signed_volume(a, b, c, d).abs() / 6.0;
            assert!(v > 0.0, "degenerate tetrahedron {t:?}");
            total += v;

            // Every vertex must be a cube corner.
            for i in t.points {
                assert!((0..8).contains(&i), "super-vertex leaked: {i}");
            }
        }
        assert!((total - 1.0).abs() < 1e-4, "total volume {total}");

        // Every input point is used by at least one tetrahedron.
        let mut used = [false; 8];
        for t in &tets {
            for i in t.points {
                used[i as usize] = true;
            }
        }
        assert!(used.iter().all(|u| *u), "some corner unused");
        assert_eq!(tetrahedralize_indices(&tets).len(), 4 * tets.len());
    }

    #[test]
    fn interior_point_is_added() {
        // Tetrahedron corners plus its centroid: 4 tetrahedra.
        let pts = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(0.0, 2.0, 0.0),
            Vector3::new(0.0, 0.0, 2.0),
            Vector3::new(0.5, 0.5, 0.5),
        ];
        let tets = tetrahedralize(&pts);
        assert_eq!(tets.len(), 4, "got {:?}", tets);
        let total: f32 = tets
            .iter()
            .map(|t| {
                signed_volume(
                    pts[t.points[0] as usize],
                    pts[t.points[1] as usize],
                    pts[t.points[2] as usize],
                    pts[t.points[3] as usize],
                )
                .abs()
                    / 6.0
            })
            .sum();
        // Volume of the outer tetrahedron = 8/6 = 1.3333.
        assert!((total - 8.0 / 6.0).abs() < 1e-3, "total {total}");
    }

    #[test]
    fn coplanar_points_give_nothing() {
        let pts = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
        ];
        assert!(tetrahedralize(&pts).is_empty());
    }
}
