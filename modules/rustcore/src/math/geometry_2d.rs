//! Port of the hit-testing subset of `core/math/geometry_2d.h`.
//!
//! Ported: `segment_intersects_segment`, `is_point_in_circle`,
//! `is_point_in_polygon`, `is_segment_intersecting_polygon`.
//! Pending: triangulation (ear clipping), `segment_intersects_circle`,
//! `convex_hull`, `bresenham_line` and the 3D half of the header.

use crate::math::funcs;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;

/// `Geometry2D::segment_intersects_segment`; returns the intersection point.
pub fn segment_intersects_segment(
    from_a: Vector2,
    to_a: Vector2,
    from_b: Vector2,
    to_b: Vector2,
) -> Option<Vector2> {
    let b = to_a - from_a;
    let mut c = from_b - from_a;
    let mut d = to_b - from_a;

    let ab_len = b.dot(b);
    if ab_len <= 0.0 {
        return None;
    }
    let bn = b / ab_len;
    c = Vector2::new(c.x * bn.x + c.y * bn.y, c.y * bn.x - c.x * bn.y);
    d = Vector2::new(d.x * bn.x + d.y * bn.y, d.y * bn.x - d.x * bn.y);

    let eps = funcs::CMP_EPSILON as f32;
    // Same sign on both C and D: the segments do not cross.
    if (c.y < -eps && d.y < -eps) || (c.y > eps && d.y > eps) {
        return None;
    }

    // Parallel or colinear.
    if funcs::is_equal_approx(c.y, d.y) {
        return None;
    }

    let ab_pos = d.x + (c.x - d.x) * d.y / (d.y - c.y);

    // The crossing point lies outside segment A-B.
    if ab_pos < 0.0 || ab_pos > 1.0 {
        return None;
    }

    Some(from_a + b * ab_pos)
}

#[inline]
pub fn is_point_in_circle(point: Vector2, circle_pos: Vector2, radius: f32) -> bool {
    point.distance_squared_to(circle_pos) <= radius * radius
}

/// Even-odd rule against the closed polygon.
pub fn is_point_in_polygon(point: Vector2, polygon: &[Vector2]) -> bool {
    let c = polygon.len();
    if c < 3 {
        return false;
    }

    let mut further_away = Vector2::new(-1e20, -1e20);
    let mut further_away_opposite = Vector2::new(1e20, 1e20);

    for p in polygon {
        further_away = further_away.max(*p);
        further_away_opposite = further_away_opposite.min(*p);
    }

    // A point guaranteed to be outside the polygon.
    further_away +=
        (further_away - further_away_opposite) * Vector2::new(1.221313, 1.512312);

    let mut intersections = 0;
    for i in 0..c {
        let v1 = polygon[i];
        let v2 = polygon[(i + 1) % c];

        if let Some(res) = segment_intersects_segment(v1, v2, point, further_away) {
            intersections += 1;
            if res.is_equal_approx(point) {
                // The point lies on an edge.
                return true;
            }
        }
    }

    intersections & 1 == 1
}

pub fn is_segment_intersecting_polygon(from: Vector2, to: Vector2, polygon: &[Vector2]) -> bool {
    let c = polygon.len();
    for i in 0..c {
        let v1 = polygon[i];
        let v2 = polygon[(i + 1) % c];
        if segment_intersects_segment(from, to, v1, v2).is_some() {
            return true;
        }
    }
    false
}

/// `vec2_cross(O, A, B)`: the z of `(A-O) x (B-O)`.
#[inline]
pub fn vec2_cross(o: Vector2, a: Vector2, b: Vector2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Andrews monotone chain (`Geometry2D::convex_hull`): counter-clockwise hull.
/// Like the engine, the first point appears again as the last element.
pub fn convex_hull(points: &[Vector2]) -> Vec<Vector2> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    // Lexicographic order, like `Vector::sort` on Vector2.
    let mut p: Vec<Vector2> = points.to_vec();
    p.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal).then(
        a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal),
    ));

    let mut h: Vec<Vector2> = Vec::with_capacity(2 * n);

    // Lower hull.
    for point in p.iter() {
        while h.len() >= 2 && vec2_cross(h[h.len() - 2], h[h.len() - 1], *point) <= 0.0 {
            h.pop();
        }
        h.push(*point);
    }

    // Upper hull.
    let lower_len = h.len() + 1;
    for point in p.iter().rev().skip(1) {
        while h.len() >= lower_len && vec2_cross(h[h.len() - 2], h[h.len() - 1], *point) <= 0.0 {
            h.pop();
        }
        h.push(*point);
    }

    // The engine leaves the first point duplicated at the end.
    h
}

/// `Geometry2D::bresenham_line`, integer grid rasterisation.
pub fn bresenham_line(from: Vector2i, to: Vector2i) -> Vec<Vector2i> {
    let delta = (to - from).abs() * 2;
    let step = (to - from).sign();
    let mut current = from;
    let mut points = Vec::new();

    if delta.x > delta.y {
        let mut err = delta.x / 2;
        while current.x != to.x {
            points.push(current);
            err -= delta.y;
            if err < 0 {
                current.y += step.y;
                err += delta.x;
            }
            current.x += step.x;
        }
    } else {
        let mut err = delta.y / 2;
        while current.y != to.y {
            points.push(current);
            err -= delta.x;
            if err < 0 {
                current.x += step.x;
                err += delta.y;
            }
            current.y += step.y;
        }
    }

    points.push(current);
    points
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_segments() {
        // X shape crossing at the origin.
        let hit = segment_intersects_segment(
            Vector2::new(-1.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, -1.0),
            Vector2::new(0.0, 1.0),
        )
        .unwrap();
        assert!(hit.is_zero_approx(), "{hit:?}");

        // Parallel segments never cross.
        assert!(segment_intersects_segment(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
        )
        .is_none());

        // Touching only at an endpoint beyond the segment: rejected.
        assert!(segment_intersects_segment(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(2.0, -1.0),
            Vector2::new(2.0, 1.0),
        )
        .is_none());
    }

    #[test]
    fn point_in_polygon() {
        let square = [
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
        ];
        assert!(is_point_in_polygon(Vector2::new(2.0, 2.0), &square));
        assert!(!is_point_in_polygon(Vector2::new(5.0, 2.0), &square));
        assert!(!is_point_in_polygon(Vector2::new(-0.5, 2.0), &square));
        // Degenerate polygons are never "inside".
        assert!(!is_point_in_polygon(Vector2::new(0.0, 0.0), &square[..2]));
    }

    #[test]
    fn circle_and_segment_polygon() {
        assert!(is_point_in_circle(Vector2::new(1.0, 0.0), Vector2::ZERO, 1.0));
        assert!(!is_point_in_circle(Vector2::new(1.5, 0.0), Vector2::ZERO, 1.0));

        let square = [
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
        ];
        assert!(is_segment_intersecting_polygon(
            Vector2::new(-1.0, 2.0),
            Vector2::new(5.0, 2.0),
            &square
        ));
        assert!(!is_segment_intersecting_polygon(
            Vector2::new(-2.0, 2.0),
            Vector2::new(-1.0, 2.0),
            &square
        ));
    }
}
