//! `scene/main/node.*`: the parent/child tree, without `Object`.
//!
//! Nodes are stored by an opaque id, with parent links and ordered child lists.
//! Ported behaviour: reparenting moves a node between lists, a node cannot be
//! its own ancestor, names are not unique (the engine allows duplicates and
//! resolves the first match), and `NodePath`-style resolution understands `.`
//! and `..`.
//!
//! Not ported: notifications, groups, ownership, and the `SceneTree` loop.

use crate::scene::node::NodeData;
use crate::string::string_name::StringName;
use std::collections::HashMap;

#[derive(Default)]
pub struct NodeTree {
    nodes: HashMap<u64, NodeData>,
    parents: HashMap<u64, u64>,
    children: HashMap<u64, Vec<u64>>,
    next_id: u64,
}

impl NodeTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `name` under `parent` (or as a root when `None`).
    pub fn add_node(&mut self, parent: Option<u64>, name: &str) -> Option<u64> {
        if let Some(p) = parent {
            if !self.nodes.contains_key(&p) {
                return None;
            }
        }

        self.next_id += 1;
        let id = self.next_id;
        let mut data = NodeData::new();
        data.set_name(StringName::new(name));
        data.set_inside_tree(true);
        self.nodes.insert(id, data);

        match parent {
            Some(p) => {
                self.parents.insert(id, p);
                self.children.entry(p).or_default().push(id);
            }
            None => {
                self.children.entry(id).or_default();
            }
        }
        Some(id)
    }

    pub fn get_node(&self, id: u64) -> Option<&NodeData> {
        self.nodes.get(&id)
    }

    /// Mutates a node's process mode (used by the scene tree and tests).
    pub fn set_process_mode(&mut self, id: u64, mode: crate::scene::node::ProcessMode) -> bool {
        match self.nodes.get_mut(&id) {
            Some(n) => {
                n.set_process_mode(mode);
                true
            }
            None => false,
        }
    }

    pub fn get_parent(&self, id: u64) -> Option<u64> {
        self.parents.get(&id).copied()
    }

    pub fn get_child_count(&self, parent: u64) -> usize {
        self.children.get(&parent).map(|c| c.len()).unwrap_or(0)
    }

    pub fn get_child(&self, parent: u64, index: usize) -> Option<u64> {
        self.children.get(&parent)?.get(index).copied()
    }

    /// `remove_child`: detaches the node from `parent`; the node itself stays
    /// alive, like the engine.
    pub fn remove_child(&mut self, parent: u64, child: u64) -> bool {
        let Some(list) = self.children.get_mut(&parent) else {
            return false;
        };
        let Some(pos) = list.iter().position(|c| *c == child) else {
            return false;
        };
        list.remove(pos);
        self.parents.remove(&child);
        true
    }

    /// `reparent`: moves `child` under `new_parent`, rejecting cycles.
    pub fn reparent(&mut self, child: u64, new_parent: u64) -> bool {
        if !self.nodes.contains_key(&child) || !self.nodes.contains_key(&new_parent) {
            return false;
        }
        if child == new_parent || self.is_ancestor(child, new_parent) {
            return false;
        }
        if let Some(old) = self.parents.get(&child).copied() {
            self.remove_child(old, child);
        }
        self.parents.insert(child, new_parent);
        self.children.entry(new_parent).or_default().push(child);
        true
    }

    /// True when `ancestor` is `node` or one of its parents.
    fn is_ancestor(&self, ancestor: u64, node: u64) -> bool {
        let mut cursor = Some(node);
        let mut guard = 0;
        while let Some(c) = cursor {
            if c == ancestor {
                return true;
            }
            cursor = self.parents.get(&c).copied();
            guard += 1;
            if guard > 65536 {
                break;
            }
        }
        false
    }

    /// `NodePath` resolution from `from`: `.` keeps the node, `..` goes to the
    /// parent, a name picks the first child with that name. A leading `/` is
    /// ignored, so callers pass the root id explicitly.
    pub fn resolve_path(&self, from: u64, path: &str) -> Option<u64> {
        let mut current = from;
        for token in path.split('/') {
            if token.is_empty() || token == "." {
                continue;
            }
            if token == ".." {
                current = self.parents.get(&current).copied()?;
                continue;
            }
            let list = self.children.get(&current)?;
            let mut found = None;
            for c in list {
                if let Some(n) = self.nodes.get(c) {
                    if n.get_name().as_string() == token {
                        found = Some(*c);
                        break;
                    }
                }
            }
            current = found?;
        }
        Some(current)
    }

    /// Slash-separated path from the root, e.g. `Root/Player/Sprite`.
    pub fn get_path(&self, id: u64) -> Option<String> {
        let mut parts = Vec::new();
        let mut cursor = Some(id);
        let mut guard = 0;
        while let Some(c) = cursor {
            let node = self.nodes.get(&c)?;
            parts.push(node.get_name().as_string());
            cursor = self.parents.get(&c).copied();
            guard += 1;
            if guard > 65536 {
                return None;
            }
        }
        parts.reverse();
        Some(parts.join("/"))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> (NodeTree, u64, u64) {
        let mut t = NodeTree::new();
        let root = t.add_node(None, "Root").unwrap();
        let player = t.add_node(Some(root), "Player").unwrap();
        (t, root, player)
    }

    #[test]
    fn add_and_query_children_in_order() {
        let (mut t, root, player) = tree();
        let a = t.add_node(Some(player), "A").unwrap();
        let b = t.add_node(Some(player), "B").unwrap();

        assert_eq!(t.get_child_count(player), 2);
        assert_eq!(t.get_child(player, 0), Some(a));
        assert_eq!(t.get_child(player, 1), Some(b));
        assert_eq!(t.get_parent(a), Some(player));
        assert_eq!(t.get_child_count(root), 1);
        assert_eq!(t.get_path(a), Some("Root/Player/A".to_string()));
    }

    #[test]
    fn remove_child_detaches_but_keeps_the_node() {
        let (mut t, _root, player) = tree();
        let a = t.add_node(Some(player), "A").unwrap();
        assert!(t.remove_child(player, a));
        assert_eq!(t.get_child_count(player), 0);
        assert_eq!(t.get_parent(a), None);
        assert!(t.get_node(a).is_some(), "the node itself must survive");
        assert!(!t.remove_child(player, a));
    }

    #[test]
    fn reparent_moves_between_lists_and_rejects_cycles() {
        let (mut t, root, player) = tree();
        let a = t.add_node(Some(player), "A").unwrap();
        let b = t.add_node(Some(player), "B").unwrap();

        // B becomes a child of A.
        assert!(t.reparent(b, a));
        assert_eq!(t.get_child_count(player), 1);
        assert_eq!(t.get_parent(b), Some(a));

        // A cannot become a child of its own descendant B.
        assert!(!t.reparent(a, b));
        // Nor of itself.
        assert!(!t.reparent(a, a));
        // Root is still the ancestor of everything.
        assert_eq!(t.get_path(b), Some("Root/Player/A/B".to_string()));
        assert_eq!(t.get_child_count(root), 1);
    }

    #[test]
    fn path_resolution_handles_dot_and_dotdot() {
        let (mut t, root, player) = tree();
        let a = t.add_node(Some(player), "A").unwrap();

        assert_eq!(t.resolve_path(root, "Player/A"), Some(a));
        assert_eq!(t.resolve_path(a, ".."), Some(player));
        assert_eq!(t.resolve_path(a, "../.."), Some(root));
        assert_eq!(t.resolve_path(a, "./"), Some(a));
        assert_eq!(t.resolve_path(root, "/Player/A/"), Some(a));
        assert_eq!(t.resolve_path(root, "Missing"), None);
        // ".." above the root fails instead of wrapping.
        assert_eq!(t.resolve_path(root, ".."), None);
    }

    #[test]
    fn duplicate_names_resolve_to_the_first_match() {
        let mut t = NodeTree::new();
        let root = t.add_node(None, "Root").unwrap();
        let first = t.add_node(Some(root), "Twin").unwrap();
        let _second = t.add_node(Some(root), "Twin").unwrap();
        assert_eq!(t.resolve_path(root, "Twin"), Some(first));
    }
}
