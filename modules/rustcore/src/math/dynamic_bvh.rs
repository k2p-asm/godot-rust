//! Dynamic AABB tree (`core/math/dynamic_bvh.*`).
//!
//! The engine's `DynamicBVH` is Bullet's `btDbvt` with leaf fattening and
//! bottom-up/top-down optimisation passes. This port keeps the same public
//! behaviour — insert / update / remove / `aabb_query` / `ray_query`, leaves
//! holding user data — with a standard dynamic AABB tree: insertion descends by
//! surface-area growth, removals splice the sibling up, and refits walk to the
//! root.
//!
//! Internally the tree stores min/max, like the engine's `Volume`, not
//! position/size: rebuilding an end point as `position + size` rounds twice and
//! a parent would stop "enclosing" its child by one ulp.
//!
//! Consequence of the different construction: the tree shape and therefore the
//! *order* of query results are not the engine's. The **sets** are, which is
//! what the tests pin down against brute force. Non-differential:
//! `dynamic_bvh.cpp` needs `PagedAllocator` and `List`, which the parity
//! harness does not link.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;

const NONE: i32 = -1;

#[derive(Clone, Copy)]
struct Node {
    min: Vector3,
    max: Vector3,
    parent: i32,
    left: i32,
    right: i32,
    /// 0 for a leaf.
    height: i32,
    user: u64,
}

impl Node {
    fn leaf(aabb: &Aabb, user: u64) -> Self {
        Self {
            min: aabb.position,
            max: aabb.position + aabb.size,
            parent: NONE,
            left: NONE,
            right: NONE,
            height: 0,
            user,
        }
    }

    fn to_aabb(&self) -> Aabb {
        Aabb::new(self.min, self.max - self.min)
    }
}

fn merge_bounds(a_min: Vector3, a_max: Vector3, b_min: Vector3, b_max: Vector3) -> (Vector3, Vector3) {
    (a_min.min(b_min), a_max.max(b_max))
}

/// `DynamicBVH::Volume::get_size`: edge products plus the perimeter sum.
fn surface_area(min: Vector3, max: Vector3) -> f32 {
    let e = max - min;
    e.x * e.y * e.z + e.x + e.y + e.z
}

pub struct DynamicBvh {
    nodes: Vec<Node>,
    free: Vec<i32>,
    root: i32,
}

impl Default for DynamicBvh {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicBvh {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            root: NONE,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root == NONE
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.free.clear();
        self.root = NONE;
    }

    fn alloc(&mut self, node: Node) -> i32 {
        if let Some(i) = self.free.pop() {
            self.nodes[i as usize] = node;
            i
        } else {
            self.nodes.push(node);
            (self.nodes.len() - 1) as i32
        }
    }

    fn free_node(&mut self, i: i32) {
        self.free.push(i);
    }

    fn is_leaf(&self, i: i32) -> bool {
        self.nodes[i as usize].left == NONE
    }

    pub fn get_aabb(&self, id: i32) -> Option<Aabb> {
        self.valid(id).then(|| self.nodes[id as usize].to_aabb())
    }

    pub fn get_user_data(&self, id: i32) -> Option<u64> {
        self.valid(id).then(|| self.nodes[id as usize].user)
    }

    fn valid(&self, id: i32) -> bool {
        id >= 0 && (id as usize) < self.nodes.len() && self.is_leaf(id)
    }

    /// Surface-area growth of `node` when it has to contain `aabb`.
    fn growth(&self, node: i32, min: Vector3, max: Vector3) -> f32 {
        let n = &self.nodes[node as usize];
        let (m_min, m_max) = merge_bounds(n.min, n.max, min, max);
        surface_area(m_min, m_max) - surface_area(n.min, n.max)
    }

    /// Inserts a leaf and returns its id.
    pub fn insert(&mut self, aabb: Aabb, user: u64) -> i32 {
        let leaf = self.alloc(Node::leaf(&aabb, user));
        let (min, max) = (aabb.position, aabb.position + aabb.size);

        if self.root == NONE {
            self.root = leaf;
            return leaf;
        }

        // Descend by surface-area growth.
        let mut index = self.root;
        while !self.is_leaf(index) {
            let l = self.nodes[index as usize].left;
            let r = self.nodes[index as usize].right;

            let here = self.growth(index, min, max);
            let cost_l = self.growth(l, min, max);
            let cost_r = self.growth(r, min, max);

            if here < cost_l && here < cost_r {
                break;
            }
            index = if cost_l < cost_r { l } else { r };
        }

        let old_parent = self.nodes[index as usize].parent;
        let (p_min, p_max) = merge_bounds(
            self.nodes[index as usize].min,
            self.nodes[index as usize].max,
            min,
            max,
        );
        let parent = self.alloc(Node {
            min: p_min,
            max: p_max,
            parent: old_parent,
            left: index,
            right: leaf,
            height: self.nodes[index as usize].height + 1,
            user: 0,
        });

        if old_parent == NONE {
            self.root = parent;
        } else if self.nodes[old_parent as usize].left == index {
            self.nodes[old_parent as usize].left = parent;
        } else {
            self.nodes[old_parent as usize].right = parent;
        }

        self.nodes[index as usize].parent = parent;
        self.nodes[leaf as usize].parent = parent;

        self.refit(old_parent);
        leaf
    }

    /// Recomputes the boxes and heights from `i` up to the root.
    fn refit(&mut self, mut i: i32) {
        while i != NONE {
            let l = self.nodes[i as usize].left;
            let r = self.nodes[i as usize].right;
            if l != NONE {
                let (min, max) = merge_bounds(
                    self.nodes[l as usize].min,
                    self.nodes[l as usize].max,
                    self.nodes[r as usize].min,
                    self.nodes[r as usize].max,
                );
                self.nodes[i as usize].min = min;
                self.nodes[i as usize].max = max;
                self.nodes[i as usize].height =
                    1 + self.nodes[l as usize].height.max(self.nodes[r as usize].height);
            }
            i = self.nodes[i as usize].parent;
        }
    }

    /// Moves a leaf. Returns false when `id` is not a live leaf.
    pub fn update(&mut self, id: i32, aabb: Aabb) -> bool {
        if !self.valid(id) {
            return false;
        }
        self.nodes[id as usize].min = aabb.position;
        self.nodes[id as usize].max = aabb.position + aabb.size;
        self.refit(id);
        true
    }

    /// Removes a leaf. Returns false when `id` is not a live leaf.
    pub fn remove(&mut self, id: i32) -> bool {
        if !self.valid(id) {
            return false;
        }

        let parent = self.nodes[id as usize].parent;
        if parent == NONE {
            self.root = NONE;
            self.free_node(id);
            return true;
        }

        let grand = self.nodes[parent as usize].parent;
        let sibling = if self.nodes[parent as usize].left == id {
            self.nodes[parent as usize].right
        } else {
            self.nodes[parent as usize].left
        };

        if grand == NONE {
            self.root = sibling;
            self.nodes[sibling as usize].parent = NONE;
        } else {
            if self.nodes[grand as usize].left == parent {
                self.nodes[grand as usize].left = sibling;
            } else {
                self.nodes[grand as usize].right = sibling;
            }
            self.nodes[sibling as usize].parent = grand;
            self.refit(grand);
        }

        self.free_node(id);
        self.free_node(parent);
        true
    }

    pub fn get_leaf_count(&self) -> usize {
        if self.root == NONE {
            return 0;
        }
        let mut count = 0;
        let mut stack = vec![self.root];
        while let Some(i) = stack.pop() {
            if self.is_leaf(i) {
                count += 1;
            } else {
                stack.push(self.nodes[i as usize].left);
                stack.push(self.nodes[i as usize].right);
            }
        }
        count
    }

    pub fn get_max_depth(&self) -> i32 {
        if self.root == NONE {
            0
        } else {
            self.nodes[self.root as usize].height
        }
    }

    /// User data of every leaf whose box overlaps `aabb`.
    pub fn aabb_query(&self, aabb: &Aabb) -> Vec<u64> {
        let mut out = Vec::new();
        if self.root == NONE {
            return out;
        }
        let q_min = aabb.position;
        let q_max = aabb.position + aabb.size;
        let mut stack = vec![self.root];
        while let Some(i) = stack.pop() {
            let n = &self.nodes[i as usize];
            if !overlaps(n.min, n.max, q_min, q_max) {
                continue;
            }
            if n.left == NONE {
                out.push(n.user);
            } else {
                stack.push(n.left);
                stack.push(n.right);
            }
        }
        out
    }

    /// User data of every leaf the segment `from -> to` crosses.
    pub fn ray_query(&self, from: Vector3, to: Vector3) -> Vec<u64> {
        let mut out = Vec::new();
        if self.root == NONE {
            return out;
        }
        // Degenerate segments: treat as a point query.
        let len_sq = (to - from).length_squared();
        let mut stack = vec![self.root];
        while let Some(i) = stack.pop() {
            let n = &self.nodes[i as usize];
            let hit = if len_sq < 1e-20 {
                let p = from;
                p.x >= n.min.x
                    && p.x <= n.max.x
                    && p.y >= n.min.y
                    && p.y <= n.max.y
                    && p.z >= n.min.z
                    && p.z <= n.max.z
            } else {
                n.to_aabb().intersects_segment(from, to).is_some()
            };
            if !hit {
                continue;
            }
            if n.left == NONE {
                out.push(n.user);
            } else {
                stack.push(n.left);
                stack.push(n.right);
            }
        }
        out
    }

    /// Structural check used by the tests: parent links, heights, containment.
    pub fn validate(&self) -> Result<(), String> {
        if self.root == NONE {
            return Ok(());
        }
        if self.nodes[self.root as usize].parent != NONE {
            return Err("root has a parent".into());
        }
        let mut stack = vec![(self.root, 0usize)];
        while let Some((i, depth)) = stack.pop() {
            if depth > 10_000 {
                return Err("cycle or runaway depth".into());
            }
            let n = self.nodes[i as usize];
            let valid_index = |c: i32| c >= 0 && (c as usize) < self.nodes.len();
            if n.left == NONE {
                if n.right != NONE || n.height != 0 {
                    return Err(format!("leaf {i} has children or height"));
                }
                continue;
            }
            if !valid_index(n.left) || !valid_index(n.right) {
                return Err(format!("node {i} has a dangling child"));
            }
            if n.right == NONE {
                return Err(format!("node {i} has only a left child"));
            }
            let l = self.nodes[n.left as usize];
            let r = self.nodes[n.right as usize];
            if l.parent != i || r.parent != i {
                return Err(format!("node {i} child parent mismatch"));
            }
            let contains = |m: Vector3, mx: Vector3| {
                n.min.x <= m.x
                    && n.min.y <= m.y
                    && n.min.z <= m.z
                    && n.max.x >= mx.x
                    && n.max.y >= mx.y
                    && n.max.z >= mx.z
            };
            if !contains(l.min, l.max) || !contains(r.min, r.max) {
                return Err(format!(
                    "node {i} does not enclose its children: parent {:?}.{:?} left {:?}.{:?} right {:?}.{:?}",
                    n.min, n.max, l.min, l.max, r.min, r.max
                ));
            }
            if n.height != 1 + l.height.max(r.height) {
                return Err(format!("node {i} has a stale height"));
            }
            stack.push((n.left, depth + 1));
            stack.push((n.right, depth + 1));
        }
        Ok(())
    }
}

/// Inclusive overlap, the same predicate as `AABB::intersects(true)`.
fn overlaps(a_min: Vector3, a_max: Vector3, b_min: Vector3, b_max: Vector3) -> bool {
    a_min.x <= b_max.x
        && a_max.x >= b_min.x
        && a_min.y <= b_max.y
        && a_max.y >= b_min.y
        && a_min.z <= b_max.z
        && a_max.z >= b_min.z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::pcg::RandomPcg;

    fn box_at(x: f32, y: f32, z: f32, s: f32) -> Aabb {
        Aabb::new(Vector3::new(x, y, z), Vector3::new(s, s, s))
    }

    fn brute_aabb(entries: &[(u64, Aabb)], q: &Aabb) -> Vec<u64> {
        let mut v: Vec<u64> = entries
            .iter()
            .filter(|(_, b)| b.intersects_inclusive(q))
            .map(|(u, _)| *u)
            .collect();
        v.sort_unstable();
        v
    }

    fn brute_ray(entries: &[(u64, Aabb)], from: Vector3, to: Vector3) -> Vec<u64> {
        let mut v: Vec<u64> = entries
            .iter()
            .filter(|(_, b)| b.intersects_segment(from, to).is_some())
            .map(|(u, _)| *u)
            .collect();
        v.sort_unstable();
        v
    }

    fn sorted(mut v: Vec<u64>) -> Vec<u64> {
        v.sort_unstable();
        v
    }

    #[test]
    fn single_box_is_found() {
        let mut bvh = DynamicBvh::new();
        let id = bvh.insert(box_at(0.0, 0.0, 0.0, 2.0), 7);
        assert_eq!(bvh.get_leaf_count(), 1);
        assert_eq!(bvh.get_user_data(id), Some(7));
        assert_eq!(bvh.aabb_query(&box_at(1.0, 1.0, 1.0, 1.0)), vec![7]);
        assert!(bvh.aabb_query(&box_at(10.0, 10.0, 10.0, 1.0)).is_empty());
        assert_eq!(bvh.validate(), Ok(()));
    }

    #[test]
    fn remove_leaves_a_consistent_tree() {
        let mut bvh = DynamicBvh::new();
        let ids: Vec<i32> = (0..20)
            .map(|i| bvh.insert(box_at(i as f32, 0.0, 0.0, 1.0), i as u64))
            .collect();
        assert_eq!(bvh.get_leaf_count(), 20);
        assert_eq!(bvh.validate(), Ok(()));

        for &id in ids.iter().take(10) {
            assert!(bvh.remove(id));
        }
        assert_eq!(bvh.get_leaf_count(), 10);
        assert_eq!(bvh.validate(), Ok(()));

        let found = bvh.aabb_query(&Aabb::new(Vector3::splat(-1.0), Vector3::new(30.0, 3.0, 3.0)));
        assert!(found.iter().all(|u| *u >= 10), "{found:?}");

        for &id in ids.iter().skip(10) {
            assert!(bvh.remove(id));
            assert_eq!(bvh.validate(), Ok(()));
        }
        assert!(bvh.is_empty());
    }

    #[test]
    fn update_moves_a_box() {
        let mut bvh = DynamicBvh::new();
        let id = bvh.insert(box_at(0.0, 0.0, 0.0, 1.0), 1);
        bvh.insert(box_at(50.0, 0.0, 0.0, 1.0), 2);

        assert_eq!(bvh.aabb_query(&box_at(0.0, 0.0, 0.0, 1.0)), vec![1]);
        assert!(bvh.update(id, box_at(50.0, 0.0, 0.0, 1.0)));
        assert_eq!(bvh.validate(), Ok(()));

        assert_eq!(sorted(bvh.aabb_query(&box_at(50.0, 0.0, 0.0, 1.0))), vec![1, 2]);
        assert!(bvh.aabb_query(&box_at(0.0, 0.0, 0.0, 1.0)).is_empty());
    }

    #[test]
    fn random_queries_match_brute_force() {
        let mut rng = RandomPcg::with_seed(0x5EED, crate::math::pcg::PCG_DEFAULT_INC_64);
        let mut bvh = DynamicBvh::new();
        let mut entries: Vec<(u64, Aabb)> = Vec::new();

        for i in 0..200u64 {
            let b = box_at(
                rng.random_f32(-50.0, 50.0),
                rng.random_f32(-50.0, 50.0),
                rng.random_f32(-50.0, 50.0),
                rng.random_f32(0.2, 4.0),
            );
            entries.push((i, b));
            bvh.insert(b, i);
        }
        assert_eq!(bvh.validate(), Ok(()));
        assert_eq!(bvh.get_leaf_count(), 200);

        for _ in 0..200 {
            let q = box_at(
                rng.random_f32(-55.0, 55.0),
                rng.random_f32(-55.0, 55.0),
                rng.random_f32(-55.0, 55.0),
                rng.random_f32(0.5, 12.0),
            );
            assert_eq!(sorted(bvh.aabb_query(&q)), brute_aabb(&entries, &q));
        }

        for _ in 0..200 {
            let from = Vector3::new(
                rng.random_f32(-60.0, 60.0),
                rng.random_f32(-60.0, 60.0),
                rng.random_f32(-60.0, 60.0),
            );
            let to = Vector3::new(
                rng.random_f32(-60.0, 60.0),
                rng.random_f32(-60.0, 60.0),
                rng.random_f32(-60.0, 60.0),
            );
            assert_eq!(sorted(bvh.ray_query(from, to)), brute_ray(&entries, from, to));
        }
    }

    #[test]
    fn insert_remove_cycles_stay_consistent() {
        let mut rng = RandomPcg::with_seed(99, crate::math::pcg::PCG_DEFAULT_INC_64);
        let mut bvh = DynamicBvh::new();
        let mut live: Vec<(i32, u64, Aabb)> = Vec::new();
        let mut next_user = 0u64;

        for step in 0..600 {
            let insert = live.is_empty() || rng.randf() < 0.6;
            if insert {
                let b = box_at(
                    rng.random_f32(-20.0, 20.0),
                    rng.random_f32(-20.0, 20.0),
                    rng.random_f32(-20.0, 20.0),
                    rng.random_f32(0.3, 3.0),
                );
                let id = bvh.insert(b, next_user);
                live.push((id, next_user, b));
                next_user += 1;
            } else {
                let k = rng.rand_bounded(live.len() as u32) as usize;
                let (id, _, _) = live.remove(k);
                assert!(bvh.remove(id), "remove failed at step {step}");
            }

            if step % 25 == 0 {
                assert_eq!(bvh.validate(), Ok(()), "invalid tree at step {step}");
                assert_eq!(bvh.get_leaf_count(), live.len());
            }
        }

        let mut entries: Vec<(u64, Aabb)> = live.iter().map(|(_, u, b)| (*u, *b)).collect();
        entries.sort_by_key(|e| e.0);
        for _ in 0..50 {
            let q = box_at(
                rng.random_f32(-22.0, 22.0),
                rng.random_f32(-22.0, 22.0),
                rng.random_f32(-22.0, 22.0),
                rng.random_f32(1.0, 10.0),
            );
            assert_eq!(sorted(bvh.aabb_query(&q)), brute_aabb(&entries, &q));
        }
    }

    #[test]
    fn invalid_ids_are_rejected() {
        let mut bvh = DynamicBvh::new();
        assert!(!bvh.remove(-1));
        assert!(!bvh.remove(0));
        assert!(!bvh.update(42, Aabb::new(Vector3::ZERO, Vector3::ONE)));
        assert!(bvh.get_aabb(-1).is_none());
    }
}
