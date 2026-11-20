//! 3D convex hull: the `ConvexHullComputer` interface without Bullet's code.
//!
//! `ConvexHullComputer::convex_hull` (core/math/convex_hull.cpp) is Bullet's
//! incremental hull with edge merging; it returns a closed, outward-oriented
//! triangulation. This port uses the same algorithm family (incremental
//! visible-face removal + horizon re-triangulation) but a different face order
//! and no edge merging, so the **mesh is equivalent, the triangle list is not
//! identical**.
//!
//! Non-differential: the C++ implementation depends on `PagedAllocator` and
//! `Vector`, which the parity harness does not link. The tests therefore check
//! the invariants that define a convex hull:
//!
//! - every input point lies on or inside the hull,
//! - every face is outward-oriented,
//! - the surface is a closed manifold (each edge shared by exactly two faces),
//! - Euler's formula `V - E + F = 2`,
//! - the enclosed volume matches the analytic volume of the shape.

use crate::math::funcs;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug, PartialEq)]
pub struct HullMesh {
    /// Hull vertices, as indices into the input point array.
    pub vertices: Vec<Vector3>,
    /// Outward-oriented triangles, indexing `vertices`.
    pub faces: Vec<[i32; 3]>,
    /// One unit normal per face.
    pub normals: Vec<Vector3>,
}

impl HullMesh {
    pub fn get_vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_triangle_count(&self) -> usize {
        self.faces.len()
    }

    /// Enclosed volume via the divergence theorem (origin-based tetrahedra).
    pub fn volume(&self) -> f32 {
        let mut v = 0.0;
        for f in &self.faces {
            let a = self.vertices[f[0] as usize];
            let b = self.vertices[f[1] as usize];
            let c = self.vertices[f[2] as usize];
            v += a.dot(b.cross(c)) / 6.0;
        }
        v.abs()
    }

    /// True when every undirected edge is used exactly twice.
    pub fn is_closed(&self) -> bool {
        use std::collections::HashMap;
        let mut edges: HashMap<(i32, i32), i32> = HashMap::new();
        for f in &self.faces {
            for (a, b) in [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])] {
                let key = if a < b { (a, b) } else { (b, a) };
                *edges.entry(key).or_insert(0) += 1;
            }
        }
        !edges.is_empty() && edges.values().all(|&n| n == 2)
    }
}

/// Builds the convex hull of `points`. Returns `None` when the points are
/// coplanar or collinear (fewer than four affinely independent points).
pub fn convex_hull(points: &[Vector3]) -> Option<HullMesh> {
    if points.len() < 4 {
        return None;
    }

    let (a, b, c, d) = initial_tetrahedron(points)?;
    let pa = points[a];
    let pb = points[b];
    let pc = points[c];
    let pd = points[d];

    // Orient the tetrahedron so the face normals point away from the interior.
    let vol = (pb - pa).cross(pc - pa).dot(pd - pa);
    let (b, c) = if vol > 0.0 { (b, c) } else { (c, b) };

    let mut faces: Vec<[i32; 3]> = vec![
        [a as i32, c as i32, b as i32],
        [a as i32, b as i32, d as i32],
        [b as i32, c as i32, d as i32],
        [c as i32, a as i32, d as i32],
    ];

    // An interior reference point, used only for the outward test in debug.
    let inside = (points[a] + points[b] + points[c] + points[d]) / 4.0;
    debug_assert!(faces
        .iter()
        .all(|f| face_normal(points, f).dot(inside - points[f[0] as usize]) < 0.0));

    for (pi, p) in points.iter().enumerate() {
        if pi == a || pi == b || pi == c || pi == d {
            continue;
        }

        // Faces this point can see.
        let visible: Vec<usize> = faces
            .iter()
            .enumerate()
            .filter(|(_, f)| face_sees(points, f, *p))
            .map(|(i, _)| i)
            .collect();
        if visible.is_empty() {
            continue; // Inside or on the hull: nothing to do.
        }

        // Horizon: directed edges of the visible region without a twin.
        use std::collections::HashSet;
        let mut edges: Vec<(i32, i32)> = Vec::new();
        for &fi in &visible {
            let f = faces[fi];
            for e in [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])] {
                if let Some(pos) = edges.iter().position(|&r| r == (e.1, e.0)) {
                    edges.remove(pos);
                } else {
                    edges.push(e);
                }
            }
        }
        debug_assert!(!edges.is_empty());

        // Drop the visible faces (descending, so indices stay valid).
        let mut visible_sorted = visible.clone();
        visible_sorted.sort_unstable();
        for &fi in visible_sorted.iter().rev() {
            faces.remove(fi);
        }

        // Close the hole with the horizon edges.
        for (e0, e1) in edges {
            faces.push([e0, e1, pi as i32]);
        }
    }

    // Compact: keep only the vertices the faces reference.
    let mut remap = vec![-1i32; points.len()];
    let mut vertices = Vec::new();
    for f in &faces {
        for &idx in f {
            let i = idx as usize;
            if remap[i] < 0 {
                remap[i] = vertices.len() as i32;
                vertices.push(points[i]);
            }
        }
    }

    let faces: Vec<[i32; 3]> = faces
        .into_iter()
        .map(|f| [remap[f[0] as usize], remap[f[1] as usize], remap[f[2] as usize]])
        .collect();

    let normals: Vec<Vector3> = faces
        .iter()
        .map(|f| {
            let a = vertices[f[0] as usize];
            let b = vertices[f[1] as usize];
            let c = vertices[f[2] as usize];
            (b - a).cross(c - a).normalized()
        })
        .collect();

    let mesh = HullMesh { vertices, faces, normals };
    debug_assert!(mesh.faces.iter().all(|f| {
        let a = mesh.vertices[f[0] as usize];
        let b = mesh.vertices[f[1] as usize];
        let c = mesh.vertices[f[2] as usize];
        (b - a).cross(c - a).length_squared() > 1e-20
    }));
    Some(mesh)
}

/// Four affinely independent points: extreme, farthest from the line, farthest
/// from the plane. Returns `None` when no such tetrahedron exists.
fn initial_tetrahedron(points: &[Vector3]) -> Option<(usize, usize, usize, usize)> {
    let n = points.len();

    // p0: lexicographically smallest, deterministic.
    let mut i0 = 0;
    for i in 1..n {
        if lex_less(points[i], points[i0]) {
            i0 = i;
        }
    }

    // p1: farthest from p0.
    let mut i1 = usize::MAX;
    let mut best = 0.0f32;
    for (i, p) in points.iter().enumerate() {
        if i == i0 {
            continue;
        }
        let d = p.distance_squared_to(points[i0]);
        if d > best {
            best = d;
            i1 = i;
        }
    }
    if i1 == usize::MAX || best < 1e-12 {
        return None; // All points coincide.
    }

    // p2: farthest from the line p0-p1.
    let dir = (points[i1] - points[i0]).normalized();
    let mut i2 = usize::MAX;
    let mut best = 0.0f32;
    for (i, p) in points.iter().enumerate() {
        if i == i0 || i == i1 {
            continue;
        }
        let v = *p - points[i0];
        let area = v.cross(dir).length_squared();
        if area > best {
            best = area;
            i2 = i;
        }
    }
    if i2 == usize::MAX || best < 1e-14 {
        return None; // Collinear.
    }

    // p3: farthest from the plane p0-p1-p2.
    let normal = (points[i1] - points[i0])
        .cross(points[i2] - points[i0])
        .normalized();
    let mut i3 = usize::MAX;
    let mut best = 0.0f32;
    for (i, p) in points.iter().enumerate() {
        if i == i0 || i == i1 || i == i2 {
            continue;
        }
        let d = normal.dot(*p - points[i0]).abs();
        if d > best {
            best = d;
            i3 = i;
        }
    }
    if i3 == usize::MAX || best < 1e-7 {
        return None; // Coplanar.
    }

    Some((i0, i1, i2, i3))
}

fn lex_less(a: Vector3, b: Vector3) -> bool {
    (a.x, a.y, a.z) < (b.x, b.y, b.z)
}

fn face_normal(points: &[Vector3], f: &[i32; 3]) -> Vector3 {
    let a = points[f[0] as usize];
    let b = points[f[1] as usize];
    let c = points[f[2] as usize];
    (b - a).cross(c - a)
}

/// Signed distance of `p` from the face plane, positive on the front side.
fn face_distance(points: &[Vector3], f: &[i32; 3], p: Vector3) -> f32 {
    let n = face_normal(points, f);
    let len = n.length();
    if len == 0.0 {
        return 0.0;
    }
    n.dot(p - points[f[0] as usize]) / len
}

fn face_sees(points: &[Vector3], f: &[i32; 3], p: Vector3) -> bool {
    face_distance(points, f, p) > 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Vec<Vector3> {
        let mut v = Vec::new();
        for &x in &[-0.5f32, 0.5] {
            for &y in &[-0.5f32, 0.5] {
                for &z in &[-0.5f32, 0.5] {
                    v.push(Vector3::new(x, y, z));
                }
            }
        }
        v
    }

    fn assert_contains_all(points: &[Vector3], mesh: &HullMesh) {
        for p in points {
            for (f, n) in mesh.faces.iter().zip(&mesh.normals) {
                let a = mesh.vertices[f[0] as usize];
                let d = n.dot(*p - a);
                assert!(d <= 1e-4, "point {p:?} outside face {f:?} by {d}");
            }
        }
    }

    fn assert_outward(mesh: &HullMesh) {
        let center = mesh.vertices.iter().fold(Vector3::ZERO, |a, b| a + *b)
            / mesh.vertices.len() as f32;
        for (f, n) in mesh.faces.iter().zip(&mesh.normals) {
            let a = mesh.vertices[f[0] as usize];
            assert!(
                n.dot(a - center) > 0.0,
                "face {f:?} normal {n:?} points inward"
            );
        }
    }

    #[test]
    fn tetrahedron_has_four_faces() {
        let pts = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ];
        let hull = convex_hull(&pts).expect("tetrahedron");
        assert_eq!(hull.get_vertex_count(), 4);
        assert_eq!(hull.get_triangle_count(), 4);
        assert!(hull.is_closed());
        assert!((hull.volume() - 1.0 / 6.0).abs() < 1e-5, "{}", hull.volume());
        assert_outward(&hull);
        assert_contains_all(&pts, &hull);
    }

    #[test]
    fn cube_gives_a_closed_cube() {
        let pts = cube();
        let hull = convex_hull(&pts).expect("cube");
        assert_eq!(hull.get_vertex_count(), 8);
        // A triangulated cube has 12 triangles (6 quads * 2).
        assert_eq!(hull.get_triangle_count(), 12);
        assert!(hull.is_closed());
        assert!((hull.volume() - 1.0).abs() < 1e-5, "volume {}", hull.volume());
        assert_outward(&hull);
        assert_contains_all(&pts, &hull);

        // Euler: V - E + F = 2.
        let mut edges = std::collections::HashSet::new();
        for f in &hull.faces {
            for (a, b) in [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])] {
                edges.insert(if a < b { (a, b) } else { (b, a) });
            }
        }
        let e = edges.len() as i32;
        let v = hull.get_vertex_count() as i32;
        let f = hull.get_triangle_count() as i32;
        assert_eq!(v - e + f, 2, "V={v} E={e} F={f}");
    }

    #[test]
    fn interior_points_are_dropped() {
        let mut pts = cube();
        pts.push(Vector3::ZERO);
        pts.push(Vector3::new(0.1, -0.2, 0.3));
        let hull = convex_hull(&pts).expect("cube with interior points");
        assert_eq!(hull.get_vertex_count(), 8);
        assert_eq!(hull.get_triangle_count(), 12);
        assert_contains_all(&pts, &hull);
    }

    #[test]
    fn coplanar_and_collinear_give_none() {
        assert!(convex_hull(&[
            Vector3::ZERO,
            Vector3::RIGHT,
            Vector3::UP,
            Vector3::new(1.0, 1.0, 0.0),
        ])
        .is_none());

        assert!(convex_hull(&[
            Vector3::ZERO,
            Vector3::RIGHT,
            Vector3::new(2.0, 0.0, 0.0),
            Vector3::new(3.0, 0.0, 0.0),
        ])
        .is_none());
    }

    #[test]
    fn sphere_points_build_a_closed_hull() {
        // 12 points of an icosahedron: the hull is the icosahedron itself.
        let phi = (1.0f32 + 5.0f32.sqrt()) / 2.0;
        let mut pts = Vec::new();
        for &(x, y, z) in &[
            (0.0f32, 1.0f32, phi),
            (0.0, -1.0, phi),
            (0.0, 1.0, -phi),
            (0.0, -1.0, -phi),
            (1.0, phi, 0.0),
            (-1.0, phi, 0.0),
            (1.0, -phi, 0.0),
            (-1.0, -phi, 0.0),
            (phi, 0.0, 1.0),
            (-phi, 0.0, 1.0),
            (phi, 0.0, -1.0),
            (-phi, 0.0, -1.0),
        ] {
            pts.push(Vector3::new(x, y, z).normalized());
        }

        let hull = convex_hull(&pts).expect("icosahedron");
        assert_eq!(hull.get_vertex_count(), 12);
        assert_eq!(hull.get_triangle_count(), 20);
        assert!(hull.is_closed());
        assert_outward(&hull);
        assert_contains_all(&pts, &hull);

        // Regular icosahedron with unit circumradius: edge = 4 / sqrt(10 + 2*sqrt(5)),
        // volume = (5/12)(3 + sqrt(5)) * edge^3.
        let edge = 4.0 / (10.0 + 2.0 * 5.0f32.sqrt()).sqrt();
        let expected = 5.0 / 12.0 * (3.0 + 5.0f32.sqrt()) * edge * edge * edge;
        assert!(
            (hull.volume() - expected).abs() < 1e-4,
            "{} vs {}",
            hull.volume(),
            expected
        );
    }

    #[test]
    fn fewer_than_four_points_is_none() {
        assert!(convex_hull(&[]).is_none());
        assert!(convex_hull(&[Vector3::ZERO, Vector3::RIGHT, Vector3::UP]).is_none());
    }
}
