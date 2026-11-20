//! Port of `core/math/a_star.h`/`.cpp`: `AStar3D`, the `AStar2D` wrapper and a
//! restricted `AStarGrid2D`.
//!
//! Costs are plain Euclidean distance (what `_compute_cost`/`_estimate_cost`
//! return in the engine). `weight_scale` scales the cost of edges *entering* a
//! point: `g + _compute_cost(p, e) * e->weight_scale`.
//!
//! Not modelled: `GDVIRTUAL` overrides, `jumping_enabled`, per-cell weights and
//! the heuristic-selection enum of `AStarGrid2D`. Non-differential: the engine
//! sources pull `class_db.h`.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug)]
struct Point {
    pos: Vector3,
    weight_scale: f32,
    connections: HashMap<i64, f32>,
    enabled: bool,
}

#[derive(PartialEq)]
struct OpenEntry {
    f: f32,
    id: i64,
}

impl Eq for OpenEntry {}

impl Ord for OpenEntry {
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal).then_with(|| o.id.cmp(&self.id))
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

pub struct AStar {
    points: HashMap<i64, Point>,
}

impl Default for AStar {
    fn default() -> Self {
        Self::new()
    }
}

impl AStar {
    pub fn new() -> Self {
        Self { points: HashMap::new() }
    }

    /// `add_point`; rejects negative ids and negative weight scales.
    pub fn add_point(&mut self, id: i64, pos: Vector3, weight_scale: f32) -> bool {
        if id < 0 || weight_scale < 0.0 {
            return false;
        }
        if let Some(p) = self.points.get_mut(&id) {
            p.pos = pos;
            p.weight_scale = weight_scale;
        } else {
            self.points
                .insert(id, Point { pos, weight_scale, connections: HashMap::new(), enabled: true });
        }
        true
    }

    pub fn has_point(&self, id: i64) -> bool {
        self.points.contains_key(&id)
    }

    pub fn get_point_position(&self, id: i64) -> Option<Vector3> {
        self.points.get(&id).map(|p| p.pos)
    }

    pub fn set_point_position(&mut self, id: i64, pos: Vector3) -> bool {
        match self.points.get_mut(&id) {
            Some(p) => {
                p.pos = pos;
                true
            }
            None => false,
        }
    }

    pub fn get_point_weight_scale(&self, id: i64) -> Option<f32> {
        self.points.get(&id).map(|p| p.weight_scale)
    }

    pub fn set_point_weight_scale(&mut self, id: i64, scale: f32) -> bool {
        if scale < 0.0 {
            return false;
        }
        match self.points.get_mut(&id) {
            Some(p) => {
                p.weight_scale = scale;
                true
            }
            None => false,
        }
    }

    pub fn set_point_disabled(&mut self, id: i64, disabled: bool) {
        if let Some(p) = self.points.get_mut(&id) {
            p.enabled = !disabled;
        }
    }

    pub fn remove_point(&mut self, id: i64) -> bool {
        if self.points.remove(&id).is_none() {
            return false;
        }
        for p in self.points.values_mut() {
            p.connections.remove(&id);
        }
        true
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn connect_points(&mut self, a: i64, b: i64, bidirectional: bool) -> bool {
        let (Some(pa), Some(pb)) = (self.points.get(&a), self.points.get(&b)) else {
            return false;
        };
        let w = pa.pos.distance_to(pb.pos);
        self.points.get_mut(&a).unwrap().connections.insert(b, w);
        if bidirectional {
            self.points.get_mut(&b).unwrap().connections.insert(a, w);
        }
        true
    }

    pub fn disconnect_points(&mut self, a: i64, b: i64, bidirectional: bool) -> bool {
        let mut removed = false;
        if let Some(p) = self.points.get_mut(&a) {
            removed |= p.connections.remove(&b).is_some();
        }
        if bidirectional {
            if let Some(p) = self.points.get_mut(&b) {
                removed |= p.connections.remove(&a).is_some();
            }
        }
        removed
    }

    pub fn are_points_connected(&self, a: i64, b: i64) -> bool {
        self.points.get(&a).is_some_and(|p| p.connections.contains_key(&b))
    }

    pub fn get_id_path(&self, from: i64, to: i64, partial_path: bool) -> Vec<i64> {
        if !self.points.contains_key(&from) || !self.points.contains_key(&to) {
            return Vec::new();
        }

        let mut open = BinaryHeap::new();
        let mut g: HashMap<i64, f32> = HashMap::new();
        let mut came: HashMap<i64, i64> = HashMap::new();
        let mut closed: HashSet<i64> = HashSet::new();

        g.insert(from, 0.0);
        open.push(OpenEntry { f: self.estimate(from, to), id: from });

        let mut best_id = from;
        let mut best_h = self.estimate(from, to);

        while let Some(OpenEntry { id: current, .. }) = open.pop() {
            if current == to {
                return Self::reconstruct(&came, from, to);
            }
            if !closed.insert(current) {
                continue;
            }
            let h = self.estimate(current, to);
            if partial_path && h < best_h {
                best_h = h;
                best_id = current;
            }

            let Some(point) = self.points.get(&current) else { continue };
            if !point.enabled {
                continue;
            }
            for (&next, &w) in point.connections.iter() {
                let Some(np) = self.points.get(&next) else { continue };
                if !np.enabled || closed.contains(&next) {
                    continue;
                }
                let tentative =
                    g.get(&current).copied().unwrap_or(f32::INFINITY) + w * np.weight_scale;
                if tentative < g.get(&next).copied().unwrap_or(f32::INFINITY) {
                    came.insert(next, current);
                    g.insert(next, tentative);
                    open.push(OpenEntry { f: tentative + self.estimate(next, to), id: next });
                }
            }
        }

        if partial_path && best_id != from {
            return Self::reconstruct(&came, from, best_id);
        }
        Vec::new()
    }

    fn estimate(&self, a: i64, b: i64) -> f32 {
        match (self.points.get(&a), self.points.get(&b)) {
            (Some(pa), Some(pb)) => pa.pos.distance_to(pb.pos),
            _ => 0.0,
        }
    }

    fn reconstruct(came: &HashMap<i64, i64>, from: i64, to: i64) -> Vec<i64> {
        let mut path = vec![to];
        let mut cur = to;
        while cur != from {
            match came.get(&cur) {
                Some(&prev) => {
                    path.push(prev);
                    cur = prev;
                }
                None => return Vec::new(),
            }
        }
        path.reverse();
        path
    }
}

/// `AStar2D`: the 3D search lifted to the plane (`Vector2(x, y)` -> z = 0).
pub struct AStar2D {
    inner: AStar,
}

impl Default for AStar2D {
    fn default() -> Self {
        Self::new()
    }
}

impl AStar2D {
    pub fn new() -> Self {
        Self { inner: AStar::new() }
    }

    pub fn add_point(&mut self, id: i64, pos: Vector2, weight_scale: f32) -> bool {
        self.inner.add_point(id, Vector3::new(pos.x, pos.y, 0.0), weight_scale)
    }

    pub fn get_point_position(&self, id: i64) -> Option<Vector2> {
        self.inner.get_point_position(id).map(|p| Vector2::new(p.x, p.y))
    }

    pub fn connect_points(&mut self, a: i64, b: i64, bidirectional: bool) -> bool {
        self.inner.connect_points(a, b, bidirectional)
    }

    pub fn disconnect_points(&mut self, a: i64, b: i64, bidirectional: bool) -> bool {
        self.inner.disconnect_points(a, b, bidirectional)
    }

    pub fn get_id_path(&self, from: i64, to: i64, partial: bool) -> Vec<i64> {
        self.inner.get_id_path(from, to, partial)
    }

    pub fn get_point_count(&self) -> usize {
        self.inner.get_point_count()
    }
}

/// Diagonal modes of `AStarGrid2D`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagonalMode {
    Never,
    Always,
    AtLeastOneWalkable,
    OnlyIfNoObstacles,
}

/// Grid search with solid cells and the four diagonal modes.
pub struct AStarGrid2D {
    size: (i32, i32),
    solid: Vec<bool>,
    /// Per-cell weight, applied to the cost of *entering* that cell, the same
    /// rule already verified for `AStar3D` (`+ w * e->weight_scale`).
    weights: Vec<f32>,
    diagonal: DiagonalMode,
    /// Only affects the positions returned by `get_point_path`, never the cost:
    /// the engine's heuristics work on grid indices.
    cell_size: Vector2,
    /// `jumping_enabled`: jump-point search, see `jump`.
    jumping: bool,
}

impl Default for AStarGrid2D {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq)]
struct GridEntry {
    f: f32,
    p: (i32, i32),
}

impl Eq for GridEntry {}

impl Ord for GridEntry {
    fn cmp(&self, o: &Self) -> Ordering {
        o.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for GridEntry {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl AStarGrid2D {
    pub fn new() -> Self {
        Self {
            size: (0, 0),
            solid: Vec::new(),
            weights: Vec::new(),
            diagonal: DiagonalMode::Always,
            cell_size: Vector2::new(1.0, 1.0),
            jumping: false,
        }
    }

    pub fn set_region(&mut self, size_x: i32, size_y: i32) {
        self.size = (size_x.max(0), size_y.max(0));
        self.solid = vec![false; (self.size.0 * self.size.1) as usize];
        self.weights = vec![1.0; (self.size.0 * self.size.1) as usize];
    }

    /// Per-cell weight (`set_point_weight_scale`). Negative values are ignored.
    pub fn set_point_weight_scale(&mut self, x: i32, y: i32, scale: f32) {
        if self.is_in_bounds(x, y) && scale >= 0.0 {
            let i = self.idx(x, y);
            self.weights[i] = scale;
        }
    }

    pub fn get_point_weight_scale(&self, x: i32, y: i32) -> f32 {
        if self.is_in_bounds(x, y) {
            self.weights[self.idx(x, y)]
        } else {
            1.0
        }
    }

    pub fn set_diagonal_mode(&mut self, mode: DiagonalMode) {
        self.diagonal = mode;
    }

    pub fn set_cell_size(&mut self, size: Vector2) {
        self.cell_size = size;
    }

    pub fn set_jumping_enabled(&mut self, enabled: bool) {
        self.jumping = enabled;
    }

    pub fn is_jumping_enabled(&self) -> bool {
        self.jumping
    }

    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.size.0 && y < self.size.1
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.size.0 + x) as usize
    }

    pub fn set_point_solid(&mut self, x: i32, y: i32, solid: bool) {
        if self.is_in_bounds(x, y) {
            let i = self.idx(x, y);
            self.solid[i] = solid;
        }
    }

    pub fn is_point_solid(&self, x: i32, y: i32) -> bool {
        self.is_in_bounds(x, y) && self.solid[self.idx(x, y)]
    }

    /// `_is_walkable`: in bounds and not solid. Out-of-bounds counts as blocked,
    /// which is what stops every jump loop.
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.is_in_bounds(x, y) && !self.solid[self.idx(x, y)]
    }

    fn neighbors(&self) -> Vec<(i32, i32)> {
        let mut v = vec![(0, -1), (1, 0), (0, 1), (-1, 0)];
        if self.diagonal != DiagonalMode::Never {
            v.extend_from_slice(&[(-1, -1), (1, -1), (1, 1), (-1, 1)]);
        }
        v
    }

    /// `_forced_successor`: walks in `(dx, dy)` while walkable and reports the
    /// first cell where a walkable side cell appears. This is what makes a jump
    /// stop at a corner the search has to consider.
    fn forced_successor(
        &self,
        px: i32,
        py: i32,
        dx: i32,
        dy: i32,
        inclusive: bool,
        end: (i32, i32),
    ) -> Option<(i32, i32)> {
        let mut prev_l = false;
        let mut prev_r = false;
        let mut l = false;
        let mut r = false;

        let mut ox = px;
        let mut oy = py;
        if inclusive {
            ox += dx;
            oy += dy;
        }

        // Perpendicular offsets, exactly as the engine writes them.
        let mut lx = px - dy;
        let mut ly = py - dx;
        let mut rx = px + dy;
        let mut ry = py + dx;

        while self.is_walkable(ox, oy) {
            if (ox, oy) == end {
                return Some(end);
            }

            prev_l = l || self.is_walkable(lx, ly);
            prev_r = r || self.is_walkable(rx, ry);

            lx += dx;
            ly += dy;
            rx += dx;
            ry += dy;

            l = self.is_walkable(lx, ly);
            r = self.is_walkable(rx, ry);

            if (l && !prev_l) || (r && !prev_r) {
                return Some((ox, oy));
            }

            ox += dx;
            oy += dy;
        }

        None
    }

    /// `_jump`: from `from` towards the neighbour `to`, returns the jump point
    /// in that direction, or `None` when the ray dies.
    ///
    /// The rule is the engine's: walk in the direction while the path stays
    /// walkable, stop at the target, at a corner (a walkable cell beside a
    /// blocked one), or where a forced successor appears on either side.
    fn jump(&self, from: (i32, i32), to: (i32, i32), end: (i32, i32)) -> Option<(i32, i32)> {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let (mut tx, mut ty) = to;

        match self.diagonal {
            DiagonalMode::Always | DiagonalMode::AtLeastOneWalkable => {
                let at_least_one = self.diagonal == DiagonalMode::AtLeastOneWalkable;

                if dx == 0 || dy == 0 {
                    return self.forced_successor(to.0, to.1, dx, dy, false, end);
                }

                while self.is_walkable(tx, ty)
                    && (!at_least_one
                        || self.is_walkable(tx, ty - dy)
                        || self.is_walkable(tx - dx, ty))
                {
                    if (tx, ty) == end {
                        return Some(end);
                    }

                    if (self.is_walkable(tx - dx, ty + dy) && !self.is_walkable(tx - dx, ty))
                        || (self.is_walkable(tx + dx, ty - dy) && !self.is_walkable(tx, ty - dy))
                    {
                        return Some((tx, ty));
                    }

                    if self
                        .forced_successor(tx + dx, ty, dx, 0, false, end)
                        .is_some()
                        || self
                            .forced_successor(tx, ty + dy, 0, dy, false, end)
                            .is_some()
                    {
                        return Some((tx, ty));
                    }

                    tx += dx;
                    ty += dy;
                }
            }
            DiagonalMode::OnlyIfNoObstacles => {
                if dx == 0 || dy == 0 {
                    return self.forced_successor(from.0, from.1, dx, dy, true, end);
                }

                while self.is_walkable(tx, ty)
                    && self.is_walkable(tx, ty - dy)
                    && self.is_walkable(tx - dx, ty)
                {
                    if (tx, ty) == end {
                        return Some(end);
                    }

                    if (self.is_walkable(tx + dx, ty + dy) && !self.is_walkable(tx, ty + dy))
                        || !self.is_walkable(tx + dx, ty)
                    {
                        return Some((tx, ty));
                    }

                    if self
                        .forced_successor(tx, ty, dx, 0, false, end)
                        .is_some()
                        || self
                            .forced_successor(tx, ty, 0, dy, false, end)
                            .is_some()
                    {
                        return Some((tx, ty));
                    }

                    tx += dx;
                    ty += dy;
                }
            }
            DiagonalMode::Never => {
                if dy == 0 {
                    return self.forced_successor(from.0, from.1, dx, 0, true, end);
                }

                while self.is_walkable(tx, ty) {
                    if (tx, ty) == end {
                        return Some(end);
                    }

                    if (self.is_walkable(tx - 1, ty) && !self.is_walkable(tx - 1, ty - dy))
                        || (self.is_walkable(tx + 1, ty) && !self.is_walkable(tx + 1, ty - dy))
                    {
                        return Some((tx, ty));
                    }

                    if self
                        .forced_successor(tx, ty, 1, 0, true, end)
                        .is_some()
                        || self
                            .forced_successor(tx, ty, -1, 0, true, end)
                            .is_some()
                    {
                        return Some((tx, ty));
                    }

                    ty += dy;
                }
            }
        }

        None
    }

    /// `_estimate_cost` with the default heuristic (`HEURISTIC_EUCLIDEAN`),
    /// computed on grid indices: `cell_size` does not enter the cost.
    fn estimate(&self, a: (i32, i32), b: (i32, i32)) -> f32 {
        let dx = (a.0 - b.0) as f32;
        let dy = (a.1 - b.1) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// `_compute_cost`: same default heuristic, used for edge cost.
    fn step_cost(&self, a: (i32, i32), b: (i32, i32)) -> f32 {
        self.estimate(a, b)
    }

    pub fn get_id_path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        if !self.is_in_bounds(from.0, from.1)
            || !self.is_in_bounds(to.0, to.1)
            || self.is_point_solid(from.0, from.1)
            || self.is_point_solid(to.0, to.1)
        {
            return Vec::new();
        }

        let h = |p: (i32, i32)| self.estimate(p, to);

        let mut open = BinaryHeap::new();
        let mut g: HashMap<(i32, i32), f32> = HashMap::new();
        let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        let mut closed: HashSet<(i32, i32)> = HashSet::new();

        g.insert(from, 0.0);
        open.push(GridEntry { f: h(from), p: from });

        while let Some(GridEntry { p: cur, .. }) = open.pop() {
            if cur == to {
                let mut path = vec![to];
                let mut c = to;
                while c != from {
                    match came.get(&c) {
                        Some(&prev) => {
                            path.push(prev);
                            c = prev;
                        }
                        None => return Vec::new(),
                    }
                }
                path.reverse();
                return path;
            }
            if !closed.insert(cur) {
                continue;
            }

            for (dx, dy) in self.neighbors() {
                let nx = cur.0 + dx;
                let ny = cur.1 + dy;
                if !self.is_in_bounds(nx, ny)
                    || self.is_point_solid(nx, ny)
                    || closed.contains(&(nx, ny))
                {
                    continue;
                }

                if dx != 0 && dy != 0 {
                    let s1 = self.is_point_solid(cur.0 + dx, cur.1);
                    let s2 = self.is_point_solid(cur.0, cur.1 + dy);
                    match self.diagonal {
                        DiagonalMode::AtLeastOneWalkable if s1 && s2 => continue,
                        DiagonalMode::OnlyIfNoObstacles if s1 || s2 => continue,
                        _ => {}
                    }
                }

                // Jump point search: replace the neighbour by the cell the jump
                // lands on. `_jump` can land on the destination itself.
                let (target, weight) = if self.jumping {
                    match self.jump(cur, (nx, ny), to) {
                        Some(e) => {
                            if closed.contains(&e) {
                                continue;
                            }
                            // The engine does not apply weight_scale while
                            // jumping (`// TODO: Make it works with weight_scale`).
                            (e, 1.0)
                        }
                        None => continue,
                    }
                } else {
                    ((nx, ny), self.get_point_weight_scale(nx, ny))
                };

                let tentative = g.get(&cur).copied().unwrap_or(f32::INFINITY)
                    + self.step_cost(cur, target) * weight;
                if tentative < g.get(&target).copied().unwrap_or(f32::INFINITY) {
                    came.insert(target, cur);
                    g.insert(target, tentative);
                    open.push(GridEntry { f: tentative + h(target), p: target });
                }
            }
        }

        Vec::new()
    }

    /// `get_point_path`: the id path converted to positions with `cell_size`.
    pub fn get_point_path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<Vector2> {
        self.get_id_path(from, to)
            .into_iter()
            .map(|(x, y)| Vector2::new(x as f32 * self.cell_size.x, y as f32 * self.cell_size.y))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> AStar {
        let mut astar = AStar::new();
        for y in 0..3 {
            for x in 0..3 {
                astar.add_point((y * 3 + x) as i64, Vector3::new(x as f32, y as f32, 0.0), 1.0);
            }
        }
        for y in 0..3i64 {
            for x in 0..3i64 {
                let id = y * 3 + x;
                if x + 1 < 3 {
                    astar.connect_points(id, id + 1, true);
                }
                if y + 1 < 3 {
                    astar.connect_points(id, id + 3, true);
                }
            }
        }
        astar
    }

    #[test]
    fn registry_basics() {
        let mut a = AStar::new();
        assert!(a.add_point(1, Vector3::ZERO, 1.0));
        assert!(!a.add_point(-1, Vector3::ZERO, 1.0));
        assert!(!a.add_point(2, Vector3::ZERO, -1.0));
        assert_eq!(a.get_point_count(), 1);
        assert!(a.set_point_position(1, Vector3::new(5.0, 0.0, 0.0)));
        assert_eq!(a.get_point_position(1).unwrap(), Vector3::new(5.0, 0.0, 0.0));
        assert!(!a.set_point_position(99, Vector3::ZERO));
        assert!(a.remove_point(1));
        assert!(!a.remove_point(1));
    }

    #[test]
    fn straight_and_diagonal_paths() {
        let a = grid();
        assert_eq!(a.get_id_path(0, 2, false), vec![0, 1, 2]);
        let path = a.get_id_path(0, 8, false);
        assert_eq!(path.len(), 5);
        assert_eq!(path[0], 0);
        assert_eq!(*path.last().unwrap(), 8);
    }

    #[test]
    fn unreachable_returns_empty() {
        let mut a = AStar::new();
        a.add_point(0, Vector3::ZERO, 1.0);
        a.add_point(1, Vector3::new(1.0, 0.0, 0.0), 1.0);
        assert!(a.get_id_path(0, 1, false).is_empty());
        assert!(a.get_id_path(0, 42, false).is_empty());
    }

    #[test]
    fn disabled_points_are_avoided() {
        let mut a = AStar::new();
        a.add_point(0, Vector3::ZERO, 1.0);
        a.add_point(1, Vector3::new(1.0, 0.0, 0.0), 1.0);
        a.add_point(2, Vector3::new(2.0, 0.0, 0.0), 1.0);
        a.connect_points(0, 1, true);
        a.connect_points(1, 2, true);
        a.set_point_disabled(1, true);
        assert!(a.get_id_path(0, 2, false).is_empty());
        a.set_point_disabled(1, false);
        assert_eq!(a.get_id_path(0, 2, false), vec![0, 1, 2]);
    }

    #[test]
    fn one_way_connections_respect_direction() {
        let mut a = AStar::new();
        a.add_point(0, Vector3::ZERO, 1.0);
        a.add_point(1, Vector3::new(1.0, 0.0, 0.0), 1.0);
        a.connect_points(0, 1, false);
        assert!(a.are_points_connected(0, 1));
        assert!(!a.are_points_connected(1, 0));
        assert_eq!(a.get_id_path(0, 1, false), vec![0, 1]);
        assert!(a.get_id_path(1, 0, false).is_empty());
    }

    #[test]
    fn partial_path_returns_closest_reachable() {
        let mut a = AStar::new();
        a.add_point(0, Vector3::ZERO, 1.0);
        a.add_point(1, Vector3::new(1.0, 0.0, 0.0), 1.0);
        a.add_point(2, Vector3::new(9.0, 0.0, 0.0), 1.0);
        a.connect_points(0, 1, true);
        assert!(a.get_id_path(0, 2, false).is_empty());
        assert_eq!(a.get_id_path(0, 2, true), vec![0, 1]);
    }

    #[test]
    fn weight_scale_reroutes_the_path() {
        let mut a = AStar::new();
        a.add_point(0, Vector3::ZERO, 1.0);
        a.add_point(1, Vector3::new(1.0, 0.0, 0.0), 1.0);
        a.add_point(2, Vector3::new(2.0, 0.0, 0.0), 1.0);
        a.add_point(3, Vector3::new(1.0, 1.0, 0.0), 1.0);
        a.connect_points(0, 1, true);
        a.connect_points(1, 2, true);
        a.connect_points(0, 3, true);
        a.connect_points(3, 2, true);

        assert_eq!(a.get_id_path(0, 2, false), vec![0, 1, 2]);
        assert!(a.set_point_weight_scale(1, 10.0));
        assert_eq!(a.get_id_path(0, 2, false), vec![0, 3, 2]);
        assert!(!a.set_point_weight_scale(1, -2.0));
        assert_eq!(a.get_point_weight_scale(1), Some(10.0));
    }

    #[test]
    fn astar2d_wraps_the_3d_search() {
        let mut a = AStar2D::new();
        a.add_point(0, Vector2::new(0.0, 0.0), 1.0);
        a.add_point(1, Vector2::new(1.0, 0.0), 1.0);
        a.add_point(2, Vector2::new(2.0, 0.0), 1.0);
        a.connect_points(0, 1, true);
        a.connect_points(1, 2, true);
        assert_eq!(a.get_point_count(), 3);
        assert_eq!(a.get_point_position(1), Some(Vector2::new(1.0, 0.0)));
        assert_eq!(a.get_id_path(0, 2, false), vec![0, 1, 2]);
        assert!(a.disconnect_points(1, 2, true));
        assert!(a.get_id_path(0, 2, false).is_empty());
    }

    #[test]
    fn grid_goes_around_solid_cells() {
        let mut g = AStarGrid2D::new();
        g.set_region(3, 3);
        g.set_point_solid(1, 0, true);
        g.set_point_solid(1, 1, true);

        let path = g.get_id_path((0, 0), (2, 0));
        assert_eq!(path.first(), Some(&(0, 0)));
        assert_eq!(path.last(), Some(&(2, 0)));
        assert!(path.contains(&(1, 2)));
        for (x, y) in &path {
            assert!(!g.is_point_solid(*x, *y));
        }
    }

    #[test]
    fn grid_diagonal_modes() {
        let mut never = AStarGrid2D::new();
        never.set_region(2, 2);
        never.set_diagonal_mode(DiagonalMode::Never);
        // Two equivalent 2-step paths exist; only the length and endpoints
        // are fixed, not which side it goes round.
        let p = never.get_id_path((0, 0), (1, 1));
        assert_eq!(p.len(), 3);
        assert_eq!(p.first(), Some(&(0, 0)));
        assert_eq!(p.last(), Some(&(1, 1)));

        let mut always = AStarGrid2D::new();
        always.set_region(2, 2);
        always.set_diagonal_mode(DiagonalMode::Always);
        assert_eq!(always.get_id_path((0, 0), (1, 1)), vec![(0, 0), (1, 1)]);

        let mut strict = AStarGrid2D::new();
        strict.set_region(3, 3);
        strict.set_diagonal_mode(DiagonalMode::OnlyIfNoObstacles);
        strict.set_point_solid(1, 0, true);
        let p2 = strict.get_id_path((0, 0), (1, 1));
        assert_eq!(p2.len(), 3, "diagonal blocked by a solid side");
        assert!(p2.contains(&(0, 1)));

        let mut lenient = AStarGrid2D::new();
        lenient.set_region(3, 3);
        lenient.set_diagonal_mode(DiagonalMode::AtLeastOneWalkable);
        lenient.set_point_solid(1, 0, true);
        assert_eq!(lenient.get_id_path((0, 0), (1, 1)), vec![(0, 0), (1, 1)]);
    }

    #[test]
    fn grid_weights_reroute_the_path() {
        // 3 wide corridor; the middle column is expensive, so the path should
        // detour through row 0 instead of paying the weight.
        let mut g = AStarGrid2D::new();
        g.set_region(3, 3);
        g.set_diagonal_mode(DiagonalMode::Never);
        g.set_point_weight_scale(1, 1, 50.0);
        assert_eq!(g.get_point_weight_scale(1, 1), 50.0);

        let path = g.get_id_path((0, 1), (2, 1));
        assert_eq!(path.first(), Some(&(0, 1)));
        assert_eq!(path.last(), Some(&(2, 1)));
        // It must not step on the expensive cell.
        assert!(!path.contains(&(1, 1)), "{path:?}");
        assert_eq!(path.len(), 5); // up, right, right, down.

        // Cheap again: straight through.
        g.set_point_weight_scale(1, 1, 1.0);
        assert_eq!(g.get_id_path((0, 1), (2, 1)), vec![(0, 1), (1, 1), (2, 1)]);
    }

    #[test]
    fn grid_jumping_skips_along_open_rows() {
        let mut g = AStarGrid2D::new();
        g.set_region(10, 1);
        g.set_diagonal_mode(DiagonalMode::Never);

        let plain = g.get_id_path((0, 0), (9, 0));
        assert_eq!(plain.len(), 10, "no jumping: one cell per step");

        g.set_jumping_enabled(true);
        assert!(g.is_jumping_enabled());
        let jumped = g.get_id_path((0, 0), (9, 0));
        assert_eq!(jumped, vec![(0, 0), (9, 0)], "open row is one jump");
    }

    #[test]
    fn grid_jumping_preserves_path_cost() {
        // Wall at x = 2 with a single gap at (2, 4).
        let mut g = AStarGrid2D::new();
        g.set_region(5, 5);
        g.set_diagonal_mode(DiagonalMode::Never);
        for y in 0..4 {
            g.set_point_solid(2, y, true);
        }

        let cost = |path: &[(i32, i32)]| -> f32 {
            path.windows(2)
                .map(|w| {
                    let dx = (w[1].0 - w[0].0) as f32;
                    let dy = (w[1].1 - w[0].1) as f32;
                    (dx * dx + dy * dy).sqrt()
                })
                .sum()
        };

        let plain = g.get_id_path((0, 0), (4, 0));
        assert!(!plain.is_empty());
        assert!(plain.contains(&(2, 4)), "plain path must use the gap: {plain:?}");
        let plain_cost = cost(&plain);

        g.set_jumping_enabled(true);
        let jumped = g.get_id_path((0, 0), (4, 0));
        assert!(!jumped.is_empty(), "jump search must find the route");
        assert_eq!(jumped.first(), Some(&(0, 0)));
        assert_eq!(jumped.last(), Some(&(4, 0)));
        // Jumps are straight lines, so checking the segment cells proves the
        // route actually goes round the wall instead of through it.
        for w in jumped.windows(2) {
            let (mut x, mut y) = w[0];
            let sx = (w[1].0 - w[0].0).signum();
            let sy = (w[1].1 - w[0].1).signum();
            assert!(sx == 0 || sy == 0, "jump must be axis-aligned in NEVER mode");
            while (x, y) != w[1] {
                x += sx;
                y += sy;
                assert!(!g.is_point_solid(x, y), "jump crossed a solid cell at ({x},{y})");
            }
        }
        assert!(
            jumped.iter().any(|p| p.1 == 4),
            "jumped path must use the gap row: {jumped:?}"
        );

        // Both are optimal in the same metric, so the total cost matches even
        // though the waypoint lists differ.
        assert!(
            (cost(&jumped) - plain_cost).abs() < 1e-4,
            "cost {} vs {}",
            cost(&jumped),
            plain_cost
        );
        assert!(jumped.len() < plain.len(), "jumping should collapse waypoints");
    }

    #[test]
    fn grid_jumping_diagonal_mode() {
        let mut g = AStarGrid2D::new();
        g.set_region(6, 6);
        g.set_diagonal_mode(DiagonalMode::Always);
        g.set_jumping_enabled(true);

        assert_eq!(
            g.get_id_path((0, 0), (5, 5)),
            vec![(0, 0), (5, 5)],
            "open diagonal is one jump"
        );

        // Same grid without jumping: one cell per step.
        g.set_jumping_enabled(false);
        assert_eq!(g.get_id_path((0, 0), (5, 5)).len(), 6);

        // A diagonal wall the jump must not cross.
        let mut blocked = AStarGrid2D::new();
        blocked.set_region(6, 6);
        blocked.set_diagonal_mode(DiagonalMode::OnlyIfNoObstacles);
        blocked.set_jumping_enabled(true);
        for y in 0..5 {
            blocked.set_point_solid(3, y, true);
        }
        let path = blocked.get_id_path((0, 0), (5, 0));
        assert!(!path.is_empty(), "must route around the wall");
        for (x, y) in &path {
            assert!(!blocked.is_point_solid(*x, *y));
        }
        assert!(path.iter().any(|p| p.1 >= 5), "must go through the opening: {path:?}");
    }

    #[test]
    fn grid_point_path_uses_cell_size() {
        let mut g = AStarGrid2D::new();
        g.set_region(3, 1);
        g.set_cell_size(Vector2::new(16.0, 32.0));
        g.set_diagonal_mode(DiagonalMode::Never);

        let points = g.get_point_path((0, 0), (2, 0));
        assert_eq!(
            points,
            vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(16.0, 0.0),
                Vector2::new(32.0, 0.0)
            ]
        );
    }

    #[test]
    fn grid_rejects_bad_endpoints() {
        let mut g = AStarGrid2D::new();
        g.set_region(2, 2);
        assert!(g.get_id_path((0, 0), (5, 5)).is_empty());
        g.set_point_solid(1, 1, true);
        assert!(g.get_id_path((0, 0), (1, 1)).is_empty());
        assert!(g.is_point_solid(1, 1));
        assert!(!g.is_in_bounds(2, 0));
    }
}
