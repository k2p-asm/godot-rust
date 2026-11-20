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
/// `Geometry2D::get_closest_point_to_segment_uncapped`: projection onto the
/// infinite line through `a` and `b` (no clamping to the segment). A degenerate
/// segment returns `a`.
pub fn get_closest_point_to_segment_uncapped(
    point: Vector2,
    a: Vector2,
    b: Vector2,
) -> Vector2 {
    let c = point - a;
    let n = b - a;
    let d = n.length_squared();
    if d < 1e-12 {
        return a;
    }
    let t = n.dot(c) / d;
    a + n * t
}

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
/// `Geometry2D::is_polygon_clockwise`: Godot's winding sum, positive when the
/// polygon is clockwise in screen coordinates (+y down).
pub fn is_polygon_clockwise(polygon: &[Vector2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut sum = 0.0f32;
    for i in 0..polygon.len() {
        let v1 = polygon[i];
        let v2 = polygon[(i + 1) % polygon.len()];
        sum += (v2.x - v1.x) * (v2.y + v1.y);
    }
    sum > 0.0
}

/// `Geometry2D::triangulate_polygon`: ear-clipping through the ported
/// `Triangulate`; an empty result means the polygon could not be triangulated.
pub fn triangulate_polygon(polygon: &[Vector2]) -> Vec<i32> {
    crate::math::triangulate::triangulate(polygon).unwrap_or_default()
}

/// `Math::next_power_of_2` for the values used by the atlas packer.
fn next_power_of_2(value: u32) -> u32 {
    if value <= 1 {
        1
    } else {
        1u32 << (32 - (value - 1).leading_zeros())
    }
}

/// `Geometry2D::make_atlas`: a scanline rect packer that picks the most square
/// power-of-two atlas among the candidate widths. Positions are indexed like the
/// input rects; the size is the packed extent (not rounded).
///
/// `None` on empty input or a non-positive width/height (the engine's
/// `ERR_FAIL_COND`). Ties in the width sort keep the input order (the engine's
/// sort is not stable).
pub fn make_atlas(rects: &[Vector2i]) -> Option<(Vec<Vector2i>, Vector2i)> {
    if rects.is_empty() || rects.iter().any(|rect| rect.x <= 0 || rect.y <= 0) {
        return None;
    }

    #[derive(Clone)]
    struct WorkRect {
        size: Vector2i,
        pos: Vector2i,
        index: usize,
    }

    let mut work: Vec<WorkRect> = rects
        .iter()
        .enumerate()
        .map(|(index, rect)| WorkRect {
            size: *rect,
            pos: Vector2i::ZERO,
            index,
        })
        .collect();
    work.sort_by(|a, b| b.size.x.cmp(&a.size.x));

    let widest = work[0].size.x;
    let mut results: Vec<(Vec<WorkRect>, i32, i32)> = Vec::new();

    for i in 0..=12u32 {
        let width = 1i32 << i;
        if width < widest {
            continue;
        }
        let mut hmax = vec![0i32; width as usize];
        let mut placed = work.clone();
        let mut ofs = 0i32;
        let mut limit_h = 0i32;
        let mut max_h = 0i32;
        let mut max_w = 0i32;

        for rect in placed.iter_mut() {
            if ofs + rect.size.x > width {
                ofs = 0;
            }
            let mut from_y = 0i32;
            for k in 0..rect.size.x as usize {
                from_y = from_y.max(hmax[ofs as usize + k]);
            }
            rect.pos = Vector2i::new(ofs, from_y);
            let end_h = from_y + rect.size.y;
            let end_w = ofs + rect.size.x;
            if ofs == 0 {
                limit_h = end_h;
            }
            for k in 0..rect.size.x as usize {
                hmax[ofs as usize + k] = end_h;
            }
            max_h = max_h.max(end_h);
            max_w = max_w.max(end_w);
            if ofs == 0 || end_h > limit_h {
                ofs += rect.size.x;
            }
        }

        results.push((placed, max_h, max_w));
    }

    // Best aspect ratio with power-of-two extents.
    let mut best = 0usize;
    let mut best_aspect = f32::MAX;
    for (index, (_, max_h, max_w)) in results.iter().enumerate() {
        let h = next_power_of_2(*max_h as u32) as f32;
        let w = next_power_of_2(*max_w as u32) as f32;
        let aspect = if h > w { h / w } else { w / h };
        if aspect < best_aspect {
            best = index;
            best_aspect = aspect;
        }
    }

    let (placed, max_h, max_w) = &results[best];
    let mut positions = vec![Vector2i::ZERO; rects.len()];
    for rect in placed {
        positions[rect.index] = rect.pos;
    }
    Some((positions, Vector2i::new(*max_w, *max_h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clockwise_detection_matches_godot() {
        // Screen coordinates: this square winds clockwise.
        let cw = [
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 4.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(4.0, 0.0),
        ];
        assert!(is_polygon_clockwise(&cw));
        let ccw: Vec<Vector2> = cw.iter().rev().copied().collect();
        assert!(!is_polygon_clockwise(&ccw));
        // Fewer than three points is never clockwise.
        assert!(!is_polygon_clockwise(&[Vector2::ZERO]));
    }

    #[test]
    fn polygon_triangulation() {
        let quad = [
            Vector2::new(0.0, 0.0),
            Vector2::new(0.0, 4.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(4.0, 0.0),
        ];
        let indices = triangulate_polygon(&quad);
        assert_eq!(indices.len(), 6, "two triangles of three indices");
        assert!(indices.iter().all(|&i| (0..4).contains(&i)));

        // Degenerate input cannot be triangulated.
        assert!(triangulate_polygon(&quad[..2]).is_empty());
    }

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

    #[test]
    fn atlas_rejects_invalid_input() {
        assert!(make_atlas(&[]).is_none());
        assert!(make_atlas(&[Vector2i::new(0, 4)]).is_none());
        assert!(make_atlas(&[Vector2i::new(4, -1)]).is_none());
    }

    #[test]
    fn atlas_single_and_edges() {
        let (positions, size) = make_atlas(&[Vector2i::new(10, 20)]).expect("packed");
        assert_eq!(positions, vec![Vector2i::ZERO]);
        assert_eq!(size, Vector2i::new(10, 20));

        // Two rects that do not fit in a row stack vertically.
        let (positions, size) =
            make_atlas(&[Vector2i::new(8, 4), Vector2i::new(8, 3)]).expect("packed");
        assert_eq!(size, Vector2i::new(8, 7));
        for (index, position) in positions.iter().enumerate() {
            let rect = if index == 0 {
                Vector2i::new(8, 4)
            } else {
                Vector2i::new(8, 3)
            };
            assert!(position.x + rect.x <= size.x);
            assert!(position.y + rect.y <= size.y);
        }
        // No overlap between the two placed rects.
        let a = (positions[0], Vector2i::new(8, 4));
        let b = (positions[1], Vector2i::new(8, 3));
        let overlap = a.0.x < b.0.x + b.1.x
            && b.0.x < a.0.x + a.1.x
            && a.0.y < b.0.y + b.1.y
            && b.0.y < a.0.y + a.1.y;
        assert!(!overlap);
    }

    #[test]
    fn atlas_packs_many_rects_within_bounds() {
        let rects = vec![
            Vector2i::new(64, 16),
            Vector2i::new(16, 64),
            Vector2i::new(32, 32),
            Vector2i::new(8, 8),
            Vector2i::new(24, 12),
            Vector2i::new(12, 24),
        ];
        let (positions, size) = make_atlas(&rects).expect("packed");
        assert_eq!(positions.len(), rects.len());
        for (index, position) in positions.iter().enumerate() {
            assert!(position.x >= 0 && position.y >= 0);
            assert!(position.x + rects[index].x <= size.x);
            assert!(position.y + rects[index].y <= size.y);
        }
        // All pairs are disjoint.
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let ax = positions[i].x;
                let ay = positions[i].y;
                let aw = rects[i].x;
                let ah = rects[i].y;
                let bx = positions[j].x;
                let by = positions[j].y;
                let bw = rects[j].x;
                let bh = rects[j].y;
                let overlap =
                    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah;
                assert!(!overlap, "rects {i} and {j} overlap");
            }
        }
        // The chosen atlas is square-ish (the power-of-two aspect is limited).
        let po2 = |v: i32| -> f32 {
            let mut p = 1i32;
            while p < v {
                p <<= 1;
            }
            p as f32
        };
        let aspect = (po2(size.x) / po2(size.y)).max(po2(size.y) / po2(size.x));
        assert!(aspect <= 4.0, "atlas aspect {aspect}");
    }
}
