//! `core/math/quick_hull.*`: the incremental convex hull builder.
//!
//! Ported from `quick_hull.cpp`: AABB rejection, duplicate removal by
//! `snappedf(0.0001)`, the two most distant points on the longest axis, the
//! third vertex furthest from that line, the fourth furthest from the triangle's
//! plane, the four outward-facing initial faces (flipped when the simplex centre
//! is over them), the `3 * UNIT_EPSILON * (sx + sy + sz)` over-tolerance,
//! point-to-face distribution, and the lit-face/horizon-edge expansion loop.
//!
//! The engine merges coplanar output faces into polygons; this port stops at the
//! triangulated hull (same faces, same planes) and lists the unique edges, which
//! is documented as the deviation. Face ordering follows the Vec's stable sort
//! rather than the intrusive list, which only affects output ordering.
//!
//! `debug_stop_after` is honoured through [`build_with_limit`].

use std::collections::HashMap;
use std::collections::HashSet;

use crate::math::aabb::Aabb;
use crate::math::plane::Plane;
use crate::math::vector3::Vector3;

const UNIT_EPSILON: f32 = 0.00001;

#[derive(Clone, Debug)]
pub struct HullFace {
    pub plane: Plane,
    pub indices: [usize; 3],
    /// Points still outside this face (drained during the build).
    pub points_over: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct ConvexHullData {
    pub faces: Vec<HullFace>,
    /// Unique undirected edges (sorted vertex pairs, ascending).
    pub edges: Vec<(usize, usize)>,
}

/// A sorted edge key, mirroring `QuickHull::Edge`'s canonical ordering.
fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

fn make_face(points: &[Vector3], center: Vector3, a: usize, b: usize, c: usize) -> HullFace {
    let mut indices = [a, b, c];
    let mut plane = Plane::from_points(points[indices[0]], points[indices[1]], points[indices[2]]);
    if plane.is_point_over(center) {
        indices.swap(0, 1);
        plane = -plane;
    }
    HullFace {
        plane,
        indices,
        points_over: Vec::new(),
    }
}

/// `QuickHull::build`; `None` mirrors `ERR_CANT_CREATE` (a degenerate AABB).
pub fn build(points: &[Vector3]) -> Option<ConvexHullData> {
    build_with_limit(points, u32::MAX)
}

/// The same build with the engine's `debug_stop_after` iteration cap.
pub fn build_with_limit(points: &[Vector3], debug_stop_after: u32) -> Option<ConvexHullData> {
    if points.is_empty() {
        return None;
    }

    let mut aabb = Aabb::new(points[0], Vector3::ZERO);
    for point in points.iter().skip(1) {
        aabb.expand_to(*point);
    }
    if aabb.size == Vector3::ZERO {
        return None;
    }

    // Discard duplicates after snapping.
    let mut valid_points = vec![true; points.len()];
    let mut valid_cache: HashSet<(i64, i64, i64)> = HashSet::new();
    for (i, point) in points.iter().enumerate() {
        let snapped = point.snappedf(0.0001);
        let key = (
            (snapped.x * 10000.0).round() as i64,
            (snapped.y * 10000.0).round() as i64,
            (snapped.z * 10000.0).round() as i64,
        );
        if valid_cache.contains(&key) {
            valid_points[i] = false;
        } else {
            valid_cache.insert(key);
        }
    }

    let longest_axis = aabb.get_longest_axis_index();

    let value_on_axis = |point: Vector3, axis: usize| -> f32 {
        match axis {
            0 => point.x,
            1 => point.y,
            _ => point.z,
        }
    };

    // First two vertices: the extremes on the longest axis.
    let mut simplex = [0usize; 4];
    {
        let mut max = 0.0f32;
        let mut min = 0.0f32;
        for (i, point) in points.iter().enumerate() {
            if !valid_points[i] {
                continue;
            }
            let d = value_on_axis(*point, longest_axis);
            if i == 0 || d < min {
                simplex[0] = i;
                min = d;
            }
            if i == 0 || d > max {
                simplex[1] = i;
                max = d;
            }
        }
    }

    // Third vertex: furthest from the line between the first two.
    {
        let mut maxd = 0.0f32;
        let rel12 = points[simplex[0]] - points[simplex[1]];
        for (i, point) in points.iter().enumerate() {
            if !valid_points[i] {
                continue;
            }
            let n = rel12
                .cross(points[simplex[0]] - *point)
                .cross(rel12)
                .normalized();
            let d = (n.dot(points[simplex[0]]) - n.dot(*point)).abs();
            if i == 0 || d > maxd {
                maxd = d;
                simplex[2] = i;
            }
        }
    }

    // Fourth vertex: furthest from the triangle's plane.
    {
        let mut maxd = 0.0f32;
        let plane = Plane::from_points(points[simplex[0]], points[simplex[1]], points[simplex[2]]);
        for (i, point) in points.iter().enumerate() {
            if !valid_points[i] {
                continue;
            }
            let d = plane.distance_to(*point).abs();
            if i == 0 || d > maxd {
                maxd = d;
                simplex[3] = i;
            }
        }
    }

    if std::env::var("QH_TRACE").is_ok() {
        eprintln!("simplex={:?} longest={}", simplex, longest_axis);
        for i in 0..4 {
            eprintln!("  s{} = {:?}", i, points[simplex[i]]);
        }
    }
    let mut center = Vector3::ZERO;
    for index in simplex {
        center += points[index];
    }
    center /= 4.0;

    const FACE_ORDER: [[usize; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
    let mut faces: Vec<HullFace> = Vec::new();
    for order in FACE_ORDER {
        faces.push(make_face(
            points,
            center,
            simplex[order[0]],
            simplex[order[1]],
            simplex[order[2]],
        ));
    }

    let over_tolerance =
        3.0 * UNIT_EPSILON * (aabb.size.x + aabb.size.y + aabb.size.z);

    // Distribute the remaining points over the initial faces.
    for i in 0..points.len() {
        if simplex.contains(&i) || !valid_points[i] {
            continue;
        }
        for face in faces.iter_mut() {
            if face.plane.distance_to(points[i]) > over_tolerance {
                face.points_over.push(i);
                break;
            }
        }
    }

    faces.sort_by_key(|face| face.points_over.len());

    let mut debug_stop = debug_stop_after;
    while debug_stop > 0 && faces.iter().any(|face| !face.points_over.is_empty()) {
        debug_stop -= 1;
        // The engine keeps the face with the most points at the back of its list;
        // this port selects it explicitly, which is order-independent.
        let mut last_index = 0;
        let mut most_points = 0;
        for (index, face) in faces.iter().enumerate() {
            if face.points_over.len() > most_points {
                most_points = face.points_over.len();
                last_index = index;
            }
        }

        // The furthest point over the selected face.
        let mut next: Option<usize> = None;
        let mut next_d = 0.0f32;
        for &candidate in &faces[last_index].points_over {
            let d = faces[last_index].plane.distance_to(points[candidate]);
            if d > next_d {
                next_d = d;
                next = Some(candidate);
            }
        }
        let Some(next_point) = next else {
            return None;
        };
        let v = points[next_point];

        if std::env::var("QH_TRACE").is_ok() {
            eprintln!("-- v={:?} next={} point={:?}", next_point, next_d, v);
            for (i, f) in faces.iter().enumerate() {
                eprintln!("   face {} plane n={:?} d={} dist={} over={:?}", i, f.plane.normal, f.plane.d, f.plane.distance_to(v), f.points_over);
            }
        }
        // Lit faces and their lit edges.
        let mut lit_faces: Vec<usize> = Vec::new();
        let mut lit_edges: HashMap<(usize, usize), (Option<usize>, Option<usize>)> =
            HashMap::new();
        for (index, face) in faces.iter().enumerate() {
            if face.plane.distance_to(v) <= 0.0 {
                continue;
            }
            lit_faces.push(index);
            for i in 0..3 {
                let a = face.indices[i];
                let b = face.indices[(i + 1) % 3];
                let key = edge_key(a, b);
                let entry = lit_edges.entry(key).or_insert((None, None));
                if key.0 == a {
                    entry.0 = Some(index);
                } else {
                    entry.1 = Some(index);
                }
            }
        }

        // Horizon edges become new faces.
        let mut new_faces: Vec<HullFace> = Vec::new();
        for (key, (left, right)) in lit_edges.iter() {
            if left.is_some() && right.is_some() {
                continue;
            }
            new_faces.push(make_face(points, center, next_point, key.0, key.1));
        }

        // Distribute the lit faces' remaining points into the new faces.
        for &lit_index in &lit_faces {
            let candidates = faces[lit_index].points_over.clone();
            for candidate in candidates {
                if candidate == next_point {
                    continue;
                }
                let p = points[candidate];
                for face in new_faces.iter_mut() {
                    if face.plane.distance_to(p) > over_tolerance {
                        face.points_over.push(candidate);
                        break;
                    }
                }
            }
        }

        if std::env::var("QH_TRACE").is_ok() {
            eprintln!("iter: faces={} lit={} new={} next={} over={}", faces.len(), lit_faces.len(), new_faces.len(), next_point, faces.iter().map(|f| f.points_over.len()).sum::<usize>());
        }
        // Drop the lit faces, append the new ones, then put the empty new faces
        // at the front so the next iteration picks the face with the most points.
        let mut lit = vec![false; faces.len()];
        for &index in &lit_faces {
            lit[index] = true;
        }
        let mut kept: Vec<HullFace> = Vec::new();
        for (index, face) in faces.drain(..).enumerate() {
            if !lit[index] {
                kept.push(face);
            }
        }
        let first_new = kept.len();
        kept.extend(new_faces);
        let mut empties: Vec<usize> = Vec::new();
        for index in first_new..kept.len() {
            if kept[index].points_over.is_empty() {
                empties.push(index);
            }
        }
        for index in empties.into_iter().rev() {
            let face = kept.remove(index);
            kept.insert(0, face);
        }
        faces = kept;
    }

    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for face in &faces {
        for i in 0..3 {
            let key = edge_key(face.indices[i], face.indices[(i + 1) % 3]);
            if seen.insert(key) {
                edges.push(key);
            }
        }
    }
    edges.sort_unstable();

    Some(ConvexHullData { faces, edges })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Vec<Vector3> {
        let mut points = Vec::new();
        for x in [-1.0f32, 1.0] {
            for y in [-1.0f32, 1.0] {
                for z in [-1.0f32, 1.0] {
                    points.push(Vector3::new(x, y, z));
                }
            }
        }
        points
    }

    #[test]
    fn aabb_probe() {
        let points = cube();
        let mut aabb = Aabb::new(points[0], Vector3::ZERO);
        for point in points.iter().skip(1) {
            aabb.expand_to(*point);
        }
        assert_eq!(aabb.size, Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(aabb.get_longest_axis_index(), 0, "ties resolve to X");
    }

    #[test]
    fn tetrahedron() {
        let points = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let hull = build(&points).expect("a tetrahedron");
        assert_eq!(hull.faces.len(), 4);
        assert_eq!(hull.edges.len(), 6);
        // Every face is outward-facing: all other points are below or on it.
        for face in &hull.faces {
            for (index, point) in points.iter().enumerate() {
                if face.indices.contains(&index) {
                    continue;
                }
                assert!(face.plane.distance_to(*point) <= 1e-4);
            }
        }
        // Every edge appears exactly once.
        let mut unique = hull.edges.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), hull.edges.len());
    }

    #[test]
    fn cube_uses_every_corner() {
        let points = cube();
        let hull = build(&points).expect("a cube");
        // 12 triangles for six quads; 12 cube edges plus 6 face diagonals.
        assert_eq!(hull.faces.len(), 12);
        assert_eq!(hull.edges.len(), 18);

        let mut used = [false; 8];
        for face in &hull.faces {
            for index in face.indices {
                used[index] = true;
            }
        }
        assert!(used.iter().all(|value| *value));

        // The hull contains the cube's corners, never above a face.
        for face in &hull.faces {
            for point in &points {
                assert!(face.plane.distance_to(*point) <= 1e-3);
            }
        }
    }

    #[test]
    fn duplicates_are_ignored() {
        let mut points = cube();
        points.push(Vector3::new(1.0, 1.0, 1.0));
        points.push(Vector3::new(-1.0, -1.0, -1.0));
        let hull = build(&points).expect("a cube with duplicates");
        assert_eq!(hull.faces.len(), 12);
    }

    #[test]
    fn interior_points_do_not_add_faces() {
        let mut points = cube();
        points.push(Vector3::ZERO);
        points.push(Vector3::new(0.5, 0.5, 0.5));
        let hull = build(&points).expect("a cube with interior points");
        assert_eq!(hull.faces.len(), 12);
        for face in &hull.faces {
            assert!(face.plane.distance_to(Vector3::ZERO) <= 1e-3);
        }
    }

    #[test]
    fn degenerate_inputs() {
        assert!(build(&[]).is_none());
        assert!(build(&[Vector3::new(2.0, 3.0, 4.0)]).is_none());
        assert!(build(&[
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
        ])
        .is_none());
    }

    #[test]
    fn stop_limit_returns_a_partial_hull() {
        // Stopping before the first expansion keeps the two initial simplex
        // faces per side (four faces) of the seed tetrahedron.
        let points = cube();
        let hull = build_with_limit(&points, 0).expect("no iterations");
        assert_eq!(hull.faces.len(), 4);
        let full = build(&points).unwrap();
        assert!(full.faces.len() > hull.faces.len());
    }
}
