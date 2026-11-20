//! `scene/main/node.*`: the data half of `Node`.
//!
//! Stored here: name and owner, the unique-name flag, process mode and
//! priorities, the process-thread group, the `inside_tree` flag, and the
//! `set_process`-family booleans.
//!
//! Not ported: the child list, groups, signals, notifications, and the
//! `process_owner` pointer. `can_process` therefore takes the parent's
//! resolved mode as an argument instead of walking the tree.

use crate::object::ObjectId;
use crate::string::string_name::StringName;

/// `Node::ProcessMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessMode {
    Inherit,
    Pausable,
    WhenPaused,
    Always,
    Disabled,
}

/// `Node::ProcessThreadGroup`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessThreadGroup {
    Inherit,
    MainThread,
    SubThread,
}

#[derive(Clone, Debug)]
pub struct NodeData {
    name: StringName,
    owner: Option<ObjectId>,
    unique_name_in_owner: bool,
    process_mode: ProcessMode,
    process_priority: i32,
    process_physics_priority: i32,
    process_thread_group: ProcessThreadGroup,
    inside_tree: bool,
    process: bool,
    physics_process: bool,
    process_internal: bool,
    physics_process_internal: bool,
    input: bool,
    shortcut_input: bool,
    unhandled_input: bool,
    unhandled_key_input: bool,
}

impl Default for NodeData {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeData {
    pub fn new() -> Self {
        Self {
            name: StringName::new(""),
            owner: None,
            unique_name_in_owner: false,
            process_mode: ProcessMode::Inherit,
            process_priority: 0,
            process_physics_priority: 0,
            process_thread_group: ProcessThreadGroup::Inherit,
            inside_tree: false,
            process: false,
            physics_process: false,
            process_internal: false,
            physics_process_internal: false,
            input: false,
            shortcut_input: false,
            unhandled_input: false,
            unhandled_key_input: false,
        }
    }

    pub fn set_name(&mut self, name: StringName) {
        self.name = name;
    }

    pub fn get_name(&self) -> StringName {
        self.name
    }

    pub fn set_owner(&mut self, owner: Option<ObjectId>) {
        self.owner = owner;
    }

    pub fn get_owner(&self) -> Option<ObjectId> {
        self.owner
    }

    pub fn set_unique_name_in_owner(&mut self, unique: bool) {
        self.unique_name_in_owner = unique;
    }

    pub fn is_unique_name_in_owner(&self) -> bool {
        self.unique_name_in_owner
    }

    pub fn set_process_mode(&mut self, mode: ProcessMode) {
        self.process_mode = mode;
    }

    pub fn get_process_mode(&self) -> ProcessMode {
        self.process_mode
    }

    pub fn set_process_priority(&mut self, priority: i32) {
        self.process_priority = priority;
    }

    pub fn get_process_priority(&self) -> i32 {
        self.process_priority
    }

    pub fn set_physics_process_priority(&mut self, priority: i32) {
        self.process_physics_priority = priority;
    }

    pub fn get_physics_process_priority(&self) -> i32 {
        self.process_physics_priority
    }

    pub fn set_process_thread_group(&mut self, group: ProcessThreadGroup) {
        self.process_thread_group = group;
    }

    pub fn get_process_thread_group(&self) -> ProcessThreadGroup {
        self.process_thread_group
    }

    /// The tree sets this on enter/exit; the rest of `Node::_notification`
    /// (groups, process registration) is out of scope here.
    pub fn set_inside_tree(&mut self, inside: bool) {
        self.inside_tree = inside;
    }

    pub fn is_inside_tree(&self) -> bool {
        self.inside_tree
    }

    pub fn set_process(&mut self, enable: bool) {
        self.process = enable;
    }

    pub fn is_processing(&self) -> bool {
        self.process
    }

    pub fn set_physics_process(&mut self, enable: bool) {
        self.physics_process = enable;
    }

    pub fn is_physics_processing(&self) -> bool {
        self.physics_process
    }

    pub fn set_process_internal(&mut self, enable: bool) {
        self.process_internal = enable;
    }

    pub fn is_processing_internal(&self) -> bool {
        self.process_internal
    }

    pub fn set_physics_process_internal(&mut self, enable: bool) {
        self.physics_process_internal = enable;
    }

    pub fn is_physics_processing_internal(&self) -> bool {
        self.physics_process_internal
    }

    pub fn set_process_input(&mut self, enable: bool) {
        self.input = enable;
    }

    pub fn is_processing_input(&self) -> bool {
        self.input
    }

    pub fn set_process_shortcut_input(&mut self, enable: bool) {
        self.shortcut_input = enable;
    }

    pub fn is_processing_shortcut_input(&self) -> bool {
        self.shortcut_input
    }

    pub fn set_process_unhandled_input(&mut self, enable: bool) {
        self.unhandled_input = enable;
    }

    pub fn is_processing_unhandled_input(&self) -> bool {
        self.unhandled_input
    }

    pub fn set_process_unhandled_key_input(&mut self, enable: bool) {
        self.unhandled_key_input = enable;
    }

    pub fn is_processing_unhandled_key_input(&self) -> bool {
        self.unhandled_key_input
    }

    /// `_can_process`'s mode resolution: `Inherit` takes the parent's resolved
    /// mode; with no parent (or an unresolved one) it behaves as `Pausable`.
    pub fn effective_process_mode(&self, parent_mode: Option<ProcessMode>) -> ProcessMode {
        if self.process_mode != ProcessMode::Inherit {
            return self.process_mode;
        }
        match parent_mode {
            Some(m) if m != ProcessMode::Inherit => m,
            _ => ProcessMode::Pausable,
        }
    }

    /// `Node::can_process`, with the tree state passed in.
    pub fn can_process(
        &self,
        paused: bool,
        suspended: bool,
        parent_mode: Option<ProcessMode>,
    ) -> bool {
        if !self.inside_tree || suspended {
            return false;
        }
        match self.effective_process_mode(parent_mode) {
            ProcessMode::Disabled => false,
            ProcessMode::Always => true,
            ProcessMode::WhenPaused => paused,
            ProcessMode::Pausable => !paused,
            // Resolution above guarantees this never survives.
            ProcessMode::Inherit => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_engine() {
        let n = NodeData::new();
        assert_eq!(n.get_name(), StringName::new(""));
        assert!(n.get_owner().is_none());
        assert!(!n.is_unique_name_in_owner());
        assert_eq!(n.get_process_mode(), ProcessMode::Inherit);
        assert_eq!(n.get_process_priority(), 0);
        assert_eq!(n.get_physics_process_priority(), 0);
        assert_eq!(n.get_process_thread_group(), ProcessThreadGroup::Inherit);
        assert!(!n.is_inside_tree());
        assert!(!n.is_processing());
        assert!(!n.is_physics_processing());
        assert!(!n.is_processing_input());
        assert!(!n.is_processing_unhandled_key_input());
    }

    #[test]
    fn inheritance_resolves_to_pausable_without_a_parent() {
        let n = NodeData::new();
        assert_eq!(n.effective_process_mode(None), ProcessMode::Pausable);
        assert_eq!(
            n.effective_process_mode(Some(ProcessMode::Always)),
            ProcessMode::Always
        );
        // An unresolved parent also falls back to pausable.
        assert_eq!(
            n.effective_process_mode(Some(ProcessMode::Inherit)),
            ProcessMode::Pausable
        );

        let mut always = NodeData::new();
        always.set_process_mode(ProcessMode::Always);
        // An explicit mode ignores the parent.
        assert_eq!(
            always.effective_process_mode(Some(ProcessMode::Disabled)),
            ProcessMode::Always
        );
    }

    #[test]
    fn can_process_matrix() {
        let mut n = NodeData::new();
        // Outside the tree: never.
        assert!(!n.can_process(false, false, None));

        n.set_inside_tree(true);
        // Inherit -> Pausable.
        assert!(n.can_process(false, false, None));
        assert!(!n.can_process(true, false, None));
        // Suspended tree stops everything.
        assert!(!n.can_process(false, true, None));

        n.set_process_mode(ProcessMode::Always);
        assert!(n.can_process(true, false, None));

        n.set_process_mode(ProcessMode::WhenPaused);
        assert!(!n.can_process(false, false, None));
        assert!(n.can_process(true, false, None));

        n.set_process_mode(ProcessMode::Disabled);
        assert!(!n.can_process(false, false, None));
        assert!(!n.can_process(true, false, None));
    }

    #[test]
    fn inheritance_follows_the_parent_mode() {
        let mut n = NodeData::new();
        n.set_inside_tree(true);

        // Parent always: inherit is always, even paused.
        assert!(n.can_process(true, false, Some(ProcessMode::Always)));
        // Parent disabled: inherit is disabled.
        assert!(!n.can_process(false, false, Some(ProcessMode::Disabled)));
        // Parent when-paused: only while paused.
        assert!(n.can_process(true, false, Some(ProcessMode::WhenPaused)));
        assert!(!n.can_process(false, false, Some(ProcessMode::WhenPaused)));
    }

    #[test]
    fn process_flags_are_independent() {
        let mut n = NodeData::new();
        n.set_process(true);
        n.set_physics_process(true);
        n.set_process_internal(true);
        n.set_physics_process_internal(true);
        n.set_process_input(true);
        n.set_process_shortcut_input(true);
        n.set_process_unhandled_input(true);
        n.set_process_unhandled_key_input(true);

        assert!(n.is_processing());
        assert!(n.is_physics_processing());
        assert!(n.is_processing_internal());
        assert!(n.is_physics_processing_internal());
        assert!(n.is_processing_input());
        assert!(n.is_processing_shortcut_input());
        assert!(n.is_processing_unhandled_input());
        assert!(n.is_processing_unhandled_key_input());

        n.set_process(false);
        assert!(!n.is_processing());
        // Turning one off does not touch the others.
        assert!(n.is_physics_processing());
    }

    #[test]
    fn name_owner_and_unique_flag() {
        let mut n = NodeData::new();
        n.set_name(StringName::new("Player"));
        n.set_unique_name_in_owner(true);
        let id = ObjectId::from_u64(77);
        n.set_owner(Some(id));

        assert_eq!(n.get_name(), StringName::new("Player"));
        assert!(n.is_unique_name_in_owner());
        assert_eq!(n.get_owner(), Some(id));

        n.set_owner(None);
        assert!(n.get_owner().is_none());
    }
}
