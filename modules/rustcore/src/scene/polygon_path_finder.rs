//! `scene/resources/2d/polygon_path_finder.*`: visibility-graph pathfinding.
//!
//! Ported operation for operation from `polygon_path_finder.cpp`:
//!
//! - `setup` stores the points, the explicit connections (validated even and
//!   in range) and the segments, then fills every other mutually visible pair by
//!   testing the midpoint for inside-ness and rejecting pairs whose segment
//!   crosses an unrelated edge;
//! - `find_path` clamps both endpoints to the closest edge when they are outside
//!   (remembering those edges so they do not count as blockers), returns the
//!   direct two-point path when nothing blocks it, otherwise runs the engine's
//!   A* over a graph extended with the two clamped endpoints and walks the
//!   `prev` chain back;
//! - `is_point_inside` counts crossings of a ray towards `outside_point` and
//!   requires an odd count;
//! - `get_closest_point`, `get_intersections`, `get_bounds` and the point
//!   penalties.
//!
//! The engine seeds `outside_point` with `Math::randf()`; this port uses a small
//! deterministic generator so setup is reproducible (documented deviation).
//! Resource serialization (`_set_data`/`_get_data`) is not ported.

use crate::math::geometry_2d;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    points: [i32; 2],
}

impl Edge {
    fn new(a: i32, b: i32) -> Self {
        if a > b {
            Self { points: [b, a] }
        } else {
            Self { points: [a, b] }
        }
    }

    fn dummy() -> Self {
        Self { points: [-1, -1] }
    }
}

#[derive(Clone, Debug, Default)]
struct Point {
    pos: Vector2,
    connections: Vec<usize>,
    distance: f32,
    penalty: f32,
    prev: i32,
}

impl Point {
    fn connect(&mut self, other: usize) {
        if !self.connections.contains(&other) {
            self.connections.push(other);
        }
    }
}

/// A tiny deterministic PRNG standing in for the engine's global RNG.
fn next_randf(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*state >> 8) & 0x00FF_FFFF) as f32 / 16_777_216.0
}

#[derive(Clone, Debug, Default)]
pub struct PolygonPathFinderData {
    outside_point: Vector2,
    bounds: Rect2,
    points: Vec<Point>,
    edges: Vec<Edge>,
    rng_state: u32,
}

impl PolygonPathFinderData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `_is_point_inside`: odd number of edge crossings towards `outside_point`.
    fn is_point_inside_inner(&self, point: Vector2) -> bool {
        let mut crosses = 0;
        for edge in &self.edges {
            let a = self.points[edge.points[0] as usize].pos;
            let b = self.points[edge.points[1] as usize].pos;
            if geometry_2d::segment_intersects_segment(a, b, point, self.outside_point).is_some()
            {
                crosses += 1;
            }
        }
        crosses & 1 == 1
    }

    /// `setup`; returns `Err` with the engine's failure conditions.
    pub fn setup(&mut self, input: &[Vector2], connections: &[i32]) -> Result<(), String> {
        if connections.len() % 2 != 0 {
            return Err("connections size must be even".to_string());
        }

        self.points.clear();
        self.edges.clear();

        let point_count = input.len();
        self.points = vec![Point::default(); point_count + 2];
        let mut bounds = Rect2::default();
        let mut outside = Vector2::ZERO;

        for (i, position) in input.iter().enumerate() {
            self.points[i].pos = *position;
            self.points[i].penalty = 0.0;
            outside = if i == 0 {
                *position
            } else {
                Vector2::new(
                    outside.x.max(position.x),
                    outside.y.max(position.y),
                )
            };
            if i == 0 {
                bounds.position = self.points[i].pos;
                bounds.size = Vector2::ZERO;
            } else {
                bounds.expand_to(self.points[i].pos);
            }
        }
        self.bounds = bounds;

        outside.x += 20.451 + next_randf(&mut self.rng_state) * 10.2039;
        outside.y += 21.193 + next_randf(&mut self.rng_state) * 12.5412;
        self.outside_point = outside;

        for pair in connections.chunks(2) {
            let a = pair[0];
            let b = pair[1];
            if a < 0 || a as usize >= point_count || b < 0 || b as usize >= point_count {
                return Err(format!("connection index out of range: {a}, {b}"));
            }
            self.points[a as usize].connect(b as usize);
            self.points[b as usize].connect(a as usize);
            self.edges.push(Edge::new(a, b));
        }

        // Fill the remaining connections from visibility.
        for i in 0..point_count {
            for j in i + 1..point_count {
                if self.edges.contains(&Edge::new(i as i32, j as i32)) {
                    continue;
                }
                let from = self.points[i].pos;
                let to = self.points[j].pos;
                if !self.is_point_inside_inner(from * 0.5 + to * 0.5) {
                    continue;
                }
                let mut valid = true;
                for edge in &self.edges {
                    if edge.points[0] == i as i32
                        || edge.points[1] == i as i32
                        || edge.points[0] == j as i32
                        || edge.points[1] == j as i32
                    {
                        continue;
                    }
                    let a = self.points[edge.points[0] as usize].pos;
                    let b = self.points[edge.points[1] as usize].pos;
                    if geometry_2d::segment_intersects_segment(a, b, from, to).is_some() {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    self.points[i].connect(j);
                    self.points[j].connect(i);
                }
            }
        }

        Ok(())
    }

    /// `find_path`: the visibility-graph A* result (empty when unreachable).
    pub fn find_path(&mut self, p_from: Vector2, p_to: Vector2) -> Vec<Vector2> {
        let mut work = self.points.clone();
        let mut path = Vec::new();
        let mut from = p_from;
        let mut to = p_to;
        let mut ignore_from_edge = Edge::dummy();
        let mut ignore_to_edge = Edge::dummy();

        if !self.is_point_inside_inner(from) {
            let mut closest_dist = 1e20f32;
            let mut closest_point = Vector2::ZERO;
            for edge in &self.edges {
                let a = work[edge.points[0] as usize].pos;
                let b = work[edge.points[1] as usize].pos;
                let closest = closest_point_on_segment(from, a, b);
                let distance = from.distance_squared_to(closest);
                if distance < closest_dist {
                    ignore_from_edge = *edge;
                    closest_dist = distance;
                    closest_point = closest;
                }
            }
            from = closest_point;
        }

        if !self.is_point_inside_inner(to) {
            let mut closest_dist = 1e20f32;
            let mut closest_point = Vector2::ZERO;
            for edge in &self.edges {
                let a = work[edge.points[0] as usize].pos;
                let b = work[edge.points[1] as usize].pos;
                let closest = closest_point_on_segment(to, a, b);
                let distance = to.distance_squared_to(closest);
                if distance < closest_dist {
                    ignore_to_edge = *edge;
                    closest_dist = distance;
                    closest_point = closest;
                }
            }
            to = closest_point;
        }

        // Direct line of sight.
        {
            let mut can_see_each_other = true;
            for edge in &self.edges {
                if *edge == ignore_from_edge || *edge == ignore_to_edge {
                    continue;
                }
                let a = work[edge.points[0] as usize].pos;
                let b = work[edge.points[1] as usize].pos;
                if geometry_2d::segment_intersects_segment(a, b, from, to).is_some() {
                    can_see_each_other = false;
                    break;
                }
            }
            if can_see_each_other {
                path.push(from);
                path.push(to);
                return path;
            }
        }

        let aidx = work.len() - 2;
        let bidx = work.len() - 1;
        work[aidx].pos = from;
        work[bidx].pos = to;
        for index in [aidx, bidx] {
            work[index].distance = 0.0;
            work[index].prev = -1;
            work[index].penalty = 0.0;
        }

        for i in 0..work.len() - 2 {
            let mut valid_a = true;
            let mut valid_b = true;
            work[i].prev = -1;
            work[i].distance = 0.0;

            if !self.is_point_inside_inner(from * 0.5 + work[i].pos * 0.5) {
                valid_a = false;
            }
            if !self.is_point_inside_inner(to * 0.5 + work[i].pos * 0.5) {
                valid_b = false;
            }

            for edge in &self.edges {
                if edge.points[0] == i as i32 || edge.points[1] == i as i32 {
                    continue;
                }
                let a = work[edge.points[0] as usize].pos;
                let b = work[edge.points[1] as usize].pos;

                if valid_a
                    && edge.points[0] != ignore_from_edge.points[1]
                    && edge.points[1] != ignore_from_edge.points[1]
                    && edge.points[0] != ignore_from_edge.points[0]
                    && edge.points[1] != ignore_from_edge.points[0]
                    && geometry_2d::segment_intersects_segment(a, b, from, work[i].pos).is_some()
                {
                    valid_a = false;
                }
                if valid_b
                    && edge.points[0] != ignore_to_edge.points[1]
                    && edge.points[1] != ignore_to_edge.points[1]
                    && edge.points[0] != ignore_to_edge.points[0]
                    && edge.points[1] != ignore_to_edge.points[0]
                    && geometry_2d::segment_intersects_segment(a, b, to, work[i].pos).is_some()
                {
                    valid_b = false;
                }
                if !valid_a && !valid_b {
                    break;
                }
            }

            if valid_a {
                work[i].connect(aidx);
                work[aidx].connect(i);
            }
            if valid_b {
                work[i].connect(bidx);
                work[bidx].connect(i);
            }
        }

        // Solve the graph (A*).
        let mut open_list: Vec<usize> = Vec::new();
        work[aidx].distance = 0.0;
        work[aidx].prev = aidx as i32;
        for index in work[aidx].connections.clone() {
            if !open_list.contains(&index) {
                open_list.push(index);
            }
            work[index].distance = from.distance_to(work[index].pos);
            work[index].prev = aidx as i32;
        }

        let mut found_route = false;
        while !open_list.is_empty() {
            let mut least_cost_point = 0usize;
            let mut least_cost = 1e30f32;
            for &index in &open_list {
                let point = &work[index];
                let cost = point.distance + point.pos.distance_to(to) + point.penalty;
                if cost < least_cost {
                    least_cost = cost;
                    least_cost_point = index;
                }
            }

            let np_pos = work[least_cost_point].pos;
            let np_distance = work[least_cost_point].distance;
            for index in work[least_cost_point].connections.clone() {
                let distance = np_pos.distance_to(work[index].pos) + np_distance;
                if work[index].prev != -1 {
                    if work[index].distance > distance {
                        work[index].prev = least_cost_point as i32;
                        work[index].distance = distance;
                    }
                } else {
                    work[index].prev = least_cost_point as i32;
                    work[index].distance = distance;
                    open_list.push(index);
                    if index == bidx {
                        found_route = true;
                        break;
                    }
                }
            }

            if found_route {
                break;
            }
            open_list.retain(|index| *index != least_cost_point);
        }

        if found_route {
            let mut at = bidx as i32;
            path.push(work[at as usize].pos);
            loop {
                at = work[at as usize].prev;
                if at < 0 {
                    break;
                }
                path.push(work[at as usize].pos);
                if at == aidx as i32 {
                    break;
                }
            }
            path.reverse();
        }

        path
    }

    pub fn is_point_inside(&self, point: Vector2) -> bool {
        self.is_point_inside_inner(point)
    }

    /// `get_closest_point`; `None` when there are no edges (the engine returns
    /// the zero vector after its `ERR_FAIL_COND_V`).
    pub fn get_closest_point(&self, point: Vector2) -> Option<Vector2> {
        let mut closest_dist = 1e20f32;
        let mut closest_point = Vector2::ZERO;
        for edge in &self.edges {
            let a = self.points[edge.points[0] as usize].pos;
            let b = self.points[edge.points[1] as usize].pos;
            let closest = closest_point_on_segment(point, a, b);
            let distance = point.distance_squared_to(closest);
            if distance < closest_dist {
                closest_dist = distance;
                closest_point = closest;
            }
        }
        if (closest_dist - 1e20).abs() < 1.0 {
            return None;
        }
        Some(closest_point)
    }

    /// `get_intersections`: every edge crossing the given segment.
    pub fn get_intersections(&self, from: Vector2, to: Vector2) -> Vec<Vector2> {
        let mut intersections = Vec::new();
        for edge in &self.edges {
            let a = self.points[edge.points[0] as usize].pos;
            let b = self.points[edge.points[1] as usize].pos;
            if let Some(result) = geometry_2d::segment_intersects_segment(a, b, from, to) {
                intersections.push(result);
            }
        }
        intersections
    }

    pub fn get_bounds(&self) -> Rect2 {
        self.bounds
    }

    /// `ERR_FAIL_INDEX(point, points.size() - 2)`.
    pub fn set_point_penalty(&mut self, point: i32, penalty: f32) -> bool {
        if point < 0 || point as usize >= self.points.len().saturating_sub(2) {
            return false;
        }
        self.points[point as usize].penalty = penalty;
        true
    }

    pub fn get_point_penalty(&self, point: i32) -> f32 {
        if point < 0 || point as usize >= self.points.len().saturating_sub(2) {
            return 0.0;
        }
        self.points[point as usize].penalty
    }

    /// The number of input points (the two temporary path endpoints excluded).
    pub fn get_point_count(&self) -> usize {
        self.points.len().saturating_sub(2)
    }

    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// `Geometry2D::get_closest_point_to_segment` (2D, capped).
fn closest_point_on_segment(point: Vector2, a: Vector2, b: Vector2) -> Vector2 {
    let c = point - a;
    let n = b - a;
    let length_squared = n.length_squared();
    if length_squared < 1e-12 {
        return a;
    }
    let d = n.dot(c);
    if d <= 0.0 {
        return a;
    }
    if d >= length_squared {
        return b;
    }
    a + n * (d / length_squared)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> PolygonPathFinderData {
        let mut finder = PolygonPathFinderData::new();
        finder
            .setup(
                &[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(4.0, 0.0),
                    Vector2::new(4.0, 4.0),
                    Vector2::new(0.0, 4.0),
                ],
                &[0, 1, 1, 2, 2, 3, 3, 0],
            )
            .unwrap();
        finder
    }

    #[test]
    fn setup_bounds_and_edges() {
        let finder = square();
        assert_eq!(finder.get_point_count(), 4);
        assert_eq!(finder.get_edge_count(), 4, "the four sides");
        assert_eq!(
            finder.get_bounds(),
            Rect2::new(Vector2::ZERO, Vector2::new(4.0, 4.0))
        );

        // Odd connection counts and bad indices are refused.
        let mut finder = PolygonPathFinderData::new();
        assert!(finder.setup(&[Vector2::ZERO], &[0]).is_err());
        assert!(finder
            .setup(&[Vector2::ZERO, Vector2::ONE], &[0, 5])
            .is_err());
        assert!(finder.setup(&[Vector2::ZERO], &[]).is_ok());
        assert_eq!(finder.get_point_count(), 1);
        assert_eq!(finder.get_edge_count(), 0);
        assert_eq!(finder.get_bounds().size, Vector2::ZERO);
    }

    #[test]
    fn inside_checks() {
        let finder = square();
        assert!(finder.is_point_inside(Vector2::new(2.0, 2.0)));
        assert!(!finder.is_point_inside(Vector2::new(10.0, 10.0)));
        assert!(!finder.is_point_inside(Vector2::new(-2.0, 2.0)));
    }

    #[test]
    fn closest_point_and_intersections() {
        let finder = square();
        assert_eq!(
            finder.get_closest_point(Vector2::new(-1.0, 2.0)),
            Some(Vector2::new(0.0, 2.0))
        );
        assert_eq!(
            finder.get_closest_point(Vector2::new(2.0, 6.0)),
            Some(Vector2::new(2.0, 4.0))
        );
        let intersections = finder.get_intersections(
            Vector2::new(-1.0, 2.0),
            Vector2::new(5.0, 2.0),
        );
        assert_eq!(intersections.len(), 2);

        // No edges: the closest point is undefined.
        let empty = PolygonPathFinderData::new();
        assert_eq!(empty.get_closest_point(Vector2::ZERO), None);
    }

    #[test]
    fn penalties() {
        let mut finder = square();
        assert_eq!(finder.get_point_penalty(0), 0.0);
        assert!(finder.set_point_penalty(0, 3.0));
        assert_eq!(finder.get_point_penalty(0), 3.0);
        assert!(!finder.set_point_penalty(-1, 1.0));
        assert!(!finder.set_point_penalty(4, 1.0));
        assert_eq!(finder.get_point_penalty(9), 0.0);
    }

    #[test]
    fn direct_path_inside() {
        let mut finder = square();
        let path = finder.find_path(Vector2::new(1.0, 1.0), Vector2::new(3.0, 3.0));
        assert_eq!(path, vec![Vector2::new(1.0, 1.0), Vector2::new(3.0, 3.0)]);
    }

    #[test]
    fn path_through_the_square() {
        let mut finder = square();
        // Both endpoints are outside, so they are clamped to the boundary and the
        // path walks the visible corners instead of crossing the walls.
        let path = finder.find_path(Vector2::new(-1.0, -1.0), Vector2::new(5.0, 5.0));
        assert!(path.len() >= 2);
        assert_eq!(path[0], Vector2::new(0.0, 0.0));
        assert_eq!(*path.last().unwrap(), Vector2::new(4.0, 4.0));
        // The route never jumps outside the polygon's bounds (clamped endpoints
        // included) and visits at least the two ends.
        assert!(path
            .iter()
            .all(|p| p.x >= -1e-5 && p.x <= 4.0 + 1e-5 && p.y >= -1e-5 && p.y <= 4.0 + 1e-5));
    }

    #[test]
    fn disconnected_polygons_have_no_path() {
        // Two separate triangles: no visibility between them, so the graph has
        // no route and `find_path` returns nothing for interior points.
        let mut finder = PolygonPathFinderData::new();
        finder
            .setup(
                &[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(10.0, 10.0),
                    Vector2::new(11.0, 10.0),
                    Vector2::new(10.0, 11.0),
                ],
                &[0, 1, 1, 2, 2, 0, 3, 4, 4, 5, 5, 3],
            )
            .unwrap();
        assert!(finder.is_point_inside(Vector2::new(0.2, 0.2)));
        assert!(finder.is_point_inside(Vector2::new(10.2, 10.2)));
        let path = finder.find_path(Vector2::new(0.2, 0.2), Vector2::new(10.2, 10.2));
        assert!(path.is_empty());
    }
}
