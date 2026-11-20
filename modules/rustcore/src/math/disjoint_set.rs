//! Port of `core/math/disjoint_set.h`: union-find with path compression and
//! union by rank.
//!
//! The C++ version stores heap-allocated nodes in a `HashMap`. Here the nodes
//! live in a `Vec` addressed by index and a `HashMap<T, usize>` maps values to
//! node indices, which avoids the allocation-per-element and the destructor
//! loop without changing the algorithm.
//!
//! `get_representatives` returns insertion order, and `get_members` uses the
//! same walk-and-compress order as the original, so results are deterministic.

use std::collections::HashMap;
use std::hash::Hash;

pub struct DisjointSet<T: Eq + Hash + Clone> {
    index: HashMap<T, usize>,
    objects: Vec<T>,
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl<T: Eq + Hash + Clone> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone> DisjointSet<T> {
    pub fn new() -> Self {
        Self { index: HashMap::new(), objects: Vec::new(), parent: Vec::new(), rank: Vec::new() }
    }

    /// `insert` / `insert_or_get`: adds the value if it is new.
    pub fn insert(&mut self, object: T) {
        self.insert_or_get(object);
    }

    fn insert_or_get(&mut self, object: T) -> usize {
        if let Some(&i) = self.index.get(&object) {
            return i;
        }
        let i = self.objects.len();
        self.objects.push(object.clone());
        self.index.insert(object, i);
        self.parent.push(i);
        self.rank.push(0);
        i
    }

    /// Find with path compression (`get_parent`).
    fn find(&mut self, mut i: usize) -> usize {
        let mut root = i;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // Second pass: compress.
        while self.parent[i] != root {
            let next = self.parent[i];
            self.parent[i] = root;
            i = next;
        }
        root
    }

    /// `create_union`: merges the sets of `a` and `b`, inserting them if needed.
    pub fn create_union(&mut self, a: T, b: T) {
        let x = self.insert_or_get(a);
        let y = self.insert_or_get(b);

        let mut x_root = self.find(x);
        let mut y_root = self.find(y);

        if x_root == y_root {
            return; // Already in the same set.
        }

        if self.rank[x_root] < self.rank[y_root] {
            std::mem::swap(&mut x_root, &mut y_root);
        }

        self.parent[y_root] = x_root;
        if self.rank[x_root] == self.rank[y_root] {
            self.rank[x_root] += 1;
        }
    }

    /// `get_representatives`, in insertion order.
    pub fn get_representatives(&mut self) -> Vec<T> {
        let n = self.objects.len();
        let mut out = Vec::new();
        for i in 0..n {
            if self.parent[i] == i {
                out.push(self.objects[i].clone());
            }
        }
        out
    }

    /// `get_members(representative)`. Empty when the value is unknown or is not
    /// a representative, matching `ERR_FAIL_NULL` / `ERR_FAIL_COND`.
    pub fn get_members(&mut self, representative: &T) -> Vec<T> {
        let Some(&rep) = self.index.get(representative) else {
            return Vec::new();
        };
        if self.parent[rep] != rep {
            return Vec::new();
        }

        let n = self.objects.len();
        let mut out = Vec::new();
        for i in 0..n {
            if self.find(i) == rep {
                out.push(self.objects[i].clone());
            }
        }
        out
    }

    /// Number of distinct elements (tests and diagnostics).
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unions_form_expected_components() {
        let mut ds = DisjointSet::new();
        ds.create_union(1, 2);
        ds.create_union(2, 3);
        ds.create_union(5, 6);

        // {1,2,3} and {5,6}, plus 9 never added yet.
        assert_eq!(ds.len(), 5);
        let reps = ds.get_representatives();
        assert_eq!(reps.len(), 2);

        let members_1 = ds.get_members(&reps[0]);
        assert_eq!(members_1.len(), 3);
        let members_2 = ds.get_members(&reps[1]);
        assert_eq!(members_2.len(), 2);
    }

    #[test]
    fn redundant_union_is_a_no_op() {
        let mut ds = DisjointSet::new();
        ds.create_union(1, 2);
        ds.create_union(2, 1); // Already connected.
        assert_eq!(ds.get_representatives().len(), 1);
        assert_eq!(ds.len(), 2);
    }

    #[test]
    fn insert_is_idempotent() {
        let mut ds = DisjointSet::<i32>::new();
        ds.insert(4);
        ds.insert(4);
        assert_eq!(ds.len(), 1);
        assert_eq!(ds.get_representatives(), vec![4]);
    }

    #[test]
    fn members_of_unknown_or_non_representative_are_empty() {
        let mut ds = DisjointSet::new();
        ds.create_union(1, 2);
        // 1 is not a representative (2 is, or vice versa) and 99 is unknown.
        assert!(ds.get_members(&99).is_empty());
        let rep = ds.get_representatives()[0];
        assert_eq!(ds.get_members(&rep).len(), 2);
        let other = if rep == 1 { 2 } else { 1 };
        assert!(ds.get_members(&other).is_empty());
    }

    #[test]
    fn path_compression_keeps_groups_correct_on_long_chains() {
        // A long chain 0-1-2-...-n: with compression every lookup stays
        // consistent, which is what the engine relies on for mesh islands.
        let mut ds = DisjointSet::new();
        for i in 0..1000 {
            ds.create_union(i, i + 1);
        }
        let reps = ds.get_representatives();
        assert_eq!(reps.len(), 1);
        assert_eq!(ds.get_members(&reps[0]).len(), 1001);
    }

    #[test]
    fn string_keys_work() {
        let mut ds = DisjointSet::new();
        ds.create_union("a".to_string(), "b".to_string());
        ds.create_union("c".to_string(), "d".to_string());
        assert_eq!(ds.get_representatives().len(), 2);
        assert_eq!(ds.len(), 4);
    }
}
