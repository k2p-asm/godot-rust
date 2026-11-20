//! `scene/main/scene_tree.*`: groups, pause state and process admission.
//!
//! Ported: the root node, group membership (`add_to_group` /
//! `get_nodes_in_group`), the paused flag, and `can_process` resolution where
//! `ProcessMode::Inherit` walks up the parent chain.
//!
//! Not ported: the main loop, `queue_free`, signals, and autoloads.

use crate::scene::node::ProcessMode;
use crate::scene::node_tree::NodeTree;
use crate::string::string_name::StringName;
use std::collections::HashMap;

pub struct SceneTreeData {
    pub tree: NodeTree,
    root: u64,
    groups: HashMap<StringName, Vec<u64>>,
    paused: bool,
    process_frames: u64,
}

impl Default for SceneTreeData {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneTreeData {
    pub fn new() -> Self {
        let mut tree = NodeTree::new();
        let root = tree.add_node(None, "root").expect("root node");
        Self {
            tree,
            root,
            groups: HashMap::new(),
            paused: false,
            process_frames: 0,
        }
    }

    pub fn get_root(&self) -> u64 {
        self.root
    }

    /// `add_to_group`: membership is a set, so duplicates are ignored.
    pub fn add_to_group(&mut self, node: u64, group: &str) -> bool {
        if self.tree.get_node(node).is_none() {
            return false;
        }
        let key = StringName::new(group);
        let list = self.groups.entry(key).or_default();
        if !list.contains(&node) {
            list.push(node);
        }
        true
    }

    pub fn remove_from_group(&mut self, node: u64, group: &str) -> bool {
        let key = StringName::new(group);
        match self.groups.get_mut(&key) {
            Some(list) => match list.iter().position(|n| *n == node) {
                Some(pos) => {
                    list.remove(pos);
                    true
                }
                None => false,
            },
            None => false,
        }
    }

    pub fn is_in_group(&self, node: u64, group: &str) -> bool {
        self.groups
            .get(&StringName::new(group))
            .map(|l| l.contains(&node))
            .unwrap_or(false)
    }

    /// Nodes in a group, sorted by id for deterministic output (the engine's
    /// order is registration order).
    pub fn get_nodes_in_group(&self, group: &str) -> Vec<u64> {
        let mut v = self
            .groups
            .get(&StringName::new(group))
            .cloned()
            .unwrap_or_default();
        v.sort_unstable();
        v
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Resolves `ProcessMode::Inherit` up the chain; an unresolved chain behaves
    /// as `Pausable`, matching `Node::_can_process`.
    pub fn resolved_process_mode(&self, node: u64) -> Option<ProcessMode> {
        let mut mode = self.tree.get_node(node)?.get_process_mode();
        let mut cursor = self.tree.get_parent(node);
        let mut guard = 0;
        while mode == ProcessMode::Inherit {
            let Some(parent) = cursor else {
                return Some(ProcessMode::Pausable);
            };
            mode = self.tree.get_node(parent)?.get_process_mode();
            cursor = self.tree.get_parent(parent);
            guard += 1;
            if guard > 65536 {
                return None;
            }
        }
        Some(mode)
    }

    /// `Node::can_process` under this tree's pause state.
    pub fn can_process(&self, node: u64) -> bool {
        if self.tree.get_node(node).map(|n| !n.is_inside_tree()).unwrap_or(true) {
            return false;
        }
        match self.resolved_process_mode(node) {
            Some(ProcessMode::Disabled) | None => false,
            Some(ProcessMode::Always) => true,
            Some(ProcessMode::WhenPaused) => self.paused,
            Some(ProcessMode::Pausable) => !self.paused,
            Some(ProcessMode::Inherit) => false,
        }
    }

    /// `queue_free` plus the frame-end flush: the node and its descendants are
    /// removed from the tree and from every group.
    pub fn queue_free(&mut self, node: u64) -> bool {
        self.tree.queue_delete(node)
    }

    pub fn flush_delete_queue(&mut self) -> usize {
        let removed = self.tree.flush_deletes();
        if removed > 0 {
            for list in self.groups.values_mut() {
                list.retain(|n| self.tree.contains_node(*n));
            }
            self.groups.retain(|_, l| !l.is_empty());
        }
        removed
    }

    /// Advances the frame counter (the loop itself is not ported).
    pub fn process_frame(&mut self) {
        self.process_frames += 1;
    }

    pub fn get_process_frames(&self) -> u64 {
        self.process_frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::node_tree::NodeTree;

    #[test]
    fn root_and_basic_groups() {
        let mut st = SceneTreeData::new();
        let root = st.get_root();
        let a = st.tree.add_node(Some(root), "A").unwrap();
        let b = st.tree.add_node(Some(root), "B").unwrap();

        assert!(st.add_to_group(a, "enemies"));
        assert!(st.add_to_group(b, "enemies"));
        // Duplicate membership is ignored.
        assert!(st.add_to_group(a, "enemies"));
        assert_eq!(st.get_nodes_in_group("enemies"), vec![a, b].into_iter().collect::<Vec<_>>());
        assert_eq!(st.get_nodes_in_group("enemies").len(), 2);

        assert!(st.is_in_group(b, "enemies"));
        assert!(st.remove_from_group(b, "enemies"));
        assert!(!st.remove_from_group(b, "enemies"));
        assert_eq!(st.get_nodes_in_group("enemies"), vec![a]);
        assert!(st.get_nodes_in_group("missing").is_empty());
    }

    #[test]
    fn group_membership_requires_a_live_node() {
        let mut st = SceneTreeData::new();
        assert!(!st.add_to_group(9999, "g"));
    }

    #[test]
    fn pause_admission_defaults_to_pausable() {
        let mut st = SceneTreeData::new();
        let n = st.tree.add_node(Some(st.get_root()), "N").unwrap();
        // Fresh nodes inherit, which resolves to Pausable.
        assert_eq!(st.resolved_process_mode(n), Some(ProcessMode::Pausable));
        assert!(st.can_process(n));
        st.set_paused(true);
        assert!(!st.can_process(n));
    }

    #[test]
    fn inherit_resolves_up_the_chain() {
        // Build nodes with explicit modes through a small local helper.
        let mut st = SceneTreeData::new();
        let root = st.get_root();
        let parent = st.tree.add_node(Some(root), "Parent").unwrap();
        let child = st.tree.add_node(Some(parent), "Child").unwrap();

        st.tree.set_process_mode(parent, ProcessMode::Always);
        assert_eq!(st.resolved_process_mode(child), Some(ProcessMode::Always));
        st.set_paused(true);
        assert!(st.can_process(child), "inherit from an Always parent processes while paused");

        st.tree.set_process_mode(parent, ProcessMode::Pausable);
        assert!(!st.can_process(child));
        st.set_paused(false);
        assert!(st.can_process(child));

        st.tree.set_process_mode(child, ProcessMode::Disabled);
        assert!(!st.can_process(child));
    }

    #[test]
    fn when_paused_only_runs_in_pause() {
        let mut st = SceneTreeData::new();
        let n = st.tree.add_node(Some(st.get_root()), "N").unwrap();
        st.tree.set_process_mode(n, ProcessMode::WhenPaused);
        assert!(!st.can_process(n));
        st.set_paused(true);
        assert!(st.can_process(n));
    }

    #[test]
    fn queue_free_removes_nodes_and_group_entries() {
        let mut st = SceneTreeData::new();
        let root = st.get_root();
        let a = st.tree.add_node(Some(root), "A").unwrap();
        let b = st.tree.add_node(Some(a), "B").unwrap();
        st.add_to_group(a, "g");
        st.add_to_group(b, "g");
        st.add_to_group(root, "g");

        assert!(st.queue_free(a));
        // Membership survives until the flush.
        assert!(st.is_in_group(a, "g"));
        assert_eq!(st.flush_delete_queue(), 2);
        assert!(!st.is_in_group(a, "g"));
        assert!(!st.is_in_group(b, "g"));
        assert!(st.is_in_group(root, "g"));
        assert_eq!(st.get_nodes_in_group("g"), vec![root]);
    }

    #[test]
    fn frame_counter() {
        let mut st = SceneTreeData::new();
        assert_eq!(st.get_process_frames(), 0);
        st.process_frame();
        st.process_frame();
        assert_eq!(st.get_process_frames(), 2);
    }
}
