//! Port of `core/math/triangulate.h/.cpp` (flipcode ear clipping).
//!
//! `triangulate` returns indices into the contour, `get_area` the signed area
//! (positive for counter-clockwise contours), and `is_inside_triangle` the
//! half-plane test used by the snip step.
//!
//! The engine's ear order is not reproduced bit for bit: ear clipping has
//! freedom in which ear it takes first. The tests therefore check the
//! invariants (count, index coverage, total area) rather than the exact index
//! list, and say so.

use crate::math::funcs;
use crate::math::vector2::Vector2;

/// Signed area of a contour (shoelace).
pub fn get_area(contour: &[Vector2]) -> f32 {
    let n = contour.len();
    let mut area = 0.0f32;
    let mut p = n as i64 - 1;

    for i in 0..n {
        area += (contour[p as usize].x - contour[i].x) * (contour[p as usize].y + contour[i].y);
        p = i as i64;
    }

    area * 0.5
}

/// Half-plane test for `(px, py)` against the triangle `(a, b, c)`.
#[allow(clippy::too_many_arguments)]
pub fn is_inside_triangle(
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    cx: f32,
    cy: f32,
    px: f32,
    py: f32,
    include_edges: bool,
) -> bool {
    let mut inside = false;

    // Edge AB.
    let ab = (cx - bx) * (py - by) - (cy - by) * (px - bx);
    // Edge BC.
    let bc = (ax - cx) * (py - cy) - (ay - cy) * (px - cx);
    // Edge CA.
    let ca = (bx - ax) * (py - ay) - (by - ay) * (px - ax);

    if include_edges {
        inside = ab >= 0.0 && bc >= 0.0 && ca >= 0.0;
    } else {
        inside = ab > 0.0 && bc > 0.0 && ca > 0.0;
    }

    inside
}

fn snip(contour: &[Vector2], u: usize, v: usize, w: usize, n: usize, vtx: &[i64], relaxed: bool) -> bool {
    let a = contour[vtx[u] as usize];
    let b = contour[vtx[v] as usize];
    let c = contour[vtx[w] as usize];

    if funcs::is_equal_approx_tol(
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x),
        0.0,
        1e-5,
    ) {
        // Degenerate triangle.
        return false;
    }

    for p in 0..n {
        if p == u || p == v || p == w {
            continue;
        }
        let point = contour[vtx[p] as usize];
        if is_inside_triangle(a.x, a.y, b.x, b.y, c.x, c.y, point.x, point.y, !relaxed) {
            return false;
        }
    }

    true
}

/// Ear-clipping triangulation. Returns `None` when the contour has fewer than
/// three vertices or no ear can be found (self-intersecting input).
pub fn triangulate(contour: &[Vector2]) -> Option<Vec<i32>> {
    let n = contour.len();
    if n < 3 {
        return None;
    }

    // Flip the vertex order for clockwise contours, as the original does.
    let mut vtx: Vec<i64> = if get_area(contour) > 0.0 {
        (0..n as i64).collect()
    } else {
        (0..n as i64).rev().collect()
    };

    let mut out: Vec<i32> = Vec::with_capacity(3 * (n - 2));
    let mut nv = n;
    let mut count: i64 = 2 * n as i64;
    let mut v: usize = nv - 1;

    while nv > 2 {
        count -= 1;
        if count < 0 {
            return None; // No ear found after the full budget: bad input.
        }

        let u = if nv <= v { 0 } else { v };
        v = if nv <= u + 1 { 0 } else { u + 1 };
        let w = if nv <= v + 1 { 0 } else { v + 1 };

        if snip(contour, u, v, w, nv, &vtx, true) {
            out.push(vtx[u] as i32);
            out.push(vtx[v] as i32);
            out.push(vtx[w] as i32);

            // Cut the ear vertex out.
            vtx.remove(v);
            nv -= 1;
            count = 2 * nv as i64;
            v = if nv == 0 { 0 } else { (v + nv - 1) % nv };
        }
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<Vector2> {
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(0.0, 1.0),
        ]
    }

    /// Area of the polygon formed by the given index triples.
    fn tri_area(contour: &[Vector2], idx: &[i32]) -> f32 {
        let mut total = 0.0;
        for t in idx.chunks(3) {
            let a = contour[t[0] as usize];
            let b = contour[t[1] as usize];
            let c = contour[t[2] as usize];
            total += ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() * 0.5;
        }
        total
    }

    #[test]
    fn area_of_square() {
        assert!((get_area(&square()) - 1.0).abs() < 1e-6);
        // Reversed winding flips the sign.
        let mut rev = square();
        rev.reverse();
        assert!((get_area(&rev) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangle_inside_test() {
        // Triangle (0,0) (1,0) (0,1).
        assert!(is_inside_triangle(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.2, 0.2, false));
        assert!(!is_inside_triangle(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.9, 0.9, true));
        // A vertex is on the edge: only accepted when edges are included.
        assert!(is_inside_triangle(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, true));
    }

    #[test]
    fn square_gives_two_triangles() {
        let c = square();
        let idx = triangulate(&c).expect("square must triangulate");
        assert_eq!(idx.len(), 6, "two triangles for a quad");
        assert!((tri_area(&c, &idx) - 1.0).abs() < 1e-5, "area {}", tri_area(&c, &idx));
        // Every emitted index is a valid contour vertex.
        assert!(idx.iter().all(|i| (0..c.len() as i32).contains(i)), "{idx:?}");
    }

    #[test]
    fn concave_polygon_is_triangulated() {
        // Arrow shape: (0,0) (2,0) (1,0.5) (2,2) (0,2)
        let c = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(1.0, 0.5),
            Vector2::new(2.0, 2.0),
            Vector2::new(0.0, 2.0),
        ];
        let idx = triangulate(&c).expect("concave polygon must triangulate");
        assert_eq!(idx.len(), 3 * (c.len() - 2));
        let poly_area = get_area(&c).abs();
        let total = tri_area(&c, &idx);
        assert!((total - poly_area).abs() < 1e-4, "{total} vs {poly_area}");
    }

    #[test]
    fn degenerate_inputs() {
        assert!(triangulate(&[]).is_none());
        assert!(triangulate(&[Vector2::ZERO, Vector2::ONE]).is_none());
        // Three collinear points have no area: no ear can be found.
        let line = [
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(2.0, 0.0),
        ];
        assert!(triangulate(&line).is_none());
    }

    #[test]
    fn convex_ngon_area_is_preserved() {
        // Regular hexagon of radius 1, centred at the origin.
        let n = 6usize;
        let c: Vec<Vector2> = (0..n)
            .map(|i| {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                Vector2::new(a.cos(), a.sin())
            })
            .collect();
        let idx = triangulate(&c).expect("hexagon must triangulate");
        assert_eq!(idx.len(), 3 * (n - 2));
        let poly = get_area(&c).abs();
        assert!((tri_area(&c, &idx) - poly).abs() < 1e-4);
    }
}
