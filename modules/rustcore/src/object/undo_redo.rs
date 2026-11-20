//! `core/object/undo_redo.*`: the undo/redo action history.
//!
//! An action is a named pair of operation lists (`do` and `undo`). Operations
//! are either a method call, a property write or an object reference, exactly the
//! three kinds the engine stores.
//!
//! Ported: `create_action` with the three merge modes and the backward-undo
//! flag, `add_do_method`/`add_undo_method` (object + method + arguments),
//! `add_do_property`/`add_undo_property`, `add_do_reference`/`add_undo_reference`,
//! `commit_action`, `undo`/`redo` (dispatched through a caller closure, since the
//! port has no live `Object`), `has_undo`/`has_redo`, `get_version`,
//! `get_history_count`, `get_current_action_name`, `clear_history`, the
//! `max_steps` trimming and the force-keep-in-merge-ends counter.
//!
//! Merge semantics as modelled: `MERGE_DISABLE` always starts a new action;
//! `MERGE_ALL` appends to the previous action when the names match and no
//! force-keep is active; `MERGE_ENDS` only merges into a previous `MERGE_ENDS`
//! action under the same conditions (the engine's time-window check is not
//! modelled). Signals/callbacks are not ported.

use crate::object::ObjectId;
use crate::string::string_name::StringName;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MergeMode {
    Disable,
    Ends,
    All,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UndoRedoOp {
    Method {
        object: ObjectId,
        method: StringName,
        args: Vec<Variant>,
    },
    Property {
        object: ObjectId,
        property: StringName,
        value: Variant,
    },
    Reference {
        object: ObjectId,
    },
}

#[derive(Clone, Debug)]
pub struct UndoAction {
    pub name: String,
    pub do_ops: Vec<UndoRedoOp>,
    pub undo_ops: Vec<UndoRedoOp>,
    pub backward_undo_ops: bool,
    merge_mode: MergeMode,
}

impl UndoAction {
    fn new(name: &str, merge_mode: MergeMode, backward_undo_ops: bool) -> Self {
        Self {
            name: name.to_string(),
            do_ops: Vec::new(),
            undo_ops: Vec::new(),
            backward_undo_ops,
            merge_mode,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct UndoRedoData {
    actions: Vec<UndoAction>,
    redo_stack: Vec<UndoAction>,
    current: Option<UndoAction>,
    action_level: i32,
    max_steps: i32,
    force_keep_in_merge_ends: i32,
    version: u64,
}

impl UndoRedoData {
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Self::default()
        }
    }

    /// `create_action`.
    pub fn create_action(
        &mut self,
        name: &str,
        merge_mode: MergeMode,
        backward_undo_ops: bool,
    ) -> bool {
        if self.action_level != 0 {
            return false;
        }
        if merge_mode != MergeMode::Disable
            && self.force_keep_in_merge_ends == 0
            && self.current.is_none()
        {
            if let Some(last) = self.actions.last() {
                let mode_allows = match merge_mode {
                    MergeMode::All => true,
                    MergeMode::Ends => last.merge_mode == MergeMode::Ends,
                    MergeMode::Disable => false,
                };
                if mode_allows && last.name == name {
                    // Re-open the previous action for appending.
                    let last = self.actions.pop().expect("checked");
                    self.current = Some(last);
                    self.redo_stack.clear();
                    self.version += 1;
                    return true;
                }
            }
        }
        self.current = Some(UndoAction::new(name, merge_mode, backward_undo_ops));
        self.version += 1;
        true
    }

    pub fn is_committing_action(&self) -> bool {
        self.current.is_some()
    }

    fn current_mut(&mut self) -> Option<&mut UndoAction> {
        self.current.as_mut()
    }

    pub fn add_do_method(&mut self, object: ObjectId, method: &str, args: Vec<Variant>) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.do_ops.push(UndoRedoOp::Method {
                    object,
                    method: StringName::new(method),
                    args,
                });
                true
            }
            _ => false,
        }
    }

    pub fn add_undo_method(&mut self, object: ObjectId, method: &str, args: Vec<Variant>) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.undo_ops.push(UndoRedoOp::Method {
                    object,
                    method: StringName::new(method),
                    args,
                });
                true
            }
            _ => false,
        }
    }

    pub fn add_do_property(&mut self, object: ObjectId, property: &str, value: Variant) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.do_ops.push(UndoRedoOp::Property {
                    object,
                    property: StringName::new(property),
                    value,
                });
                true
            }
            _ => false,
        }
    }

    pub fn add_undo_property(&mut self, object: ObjectId, property: &str, value: Variant) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.undo_ops.push(UndoRedoOp::Property {
                    object,
                    property: StringName::new(property),
                    value,
                });
                true
            }
            _ => false,
        }
    }

    pub fn add_do_reference(&mut self, object: ObjectId) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.do_ops.push(UndoRedoOp::Reference { object });
                true
            }
            _ => false,
        }
    }

    pub fn add_undo_reference(&mut self, object: ObjectId) -> bool {
        match self.current_mut() {
            Some(action) if object != ObjectId::NONE => {
                action.undo_ops.push(UndoRedoOp::Reference { object });
                true
            }
            _ => false,
        }
    }

    pub fn start_force_keep_in_merge_ends(&mut self) {
        self.force_keep_in_merge_ends += 1;
    }

    pub fn end_force_keep_in_merge_ends(&mut self) {
        self.force_keep_in_merge_ends = self.force_keep_in_merge_ends.saturating_sub(1);
    }

    /// `commit_action`: closes the current action and adds it to the history.
    pub fn commit_action(&mut self) -> bool {
        let Some(action) = self.current.take() else {
            return false;
        };
        self.actions.push(action);
        self.redo_stack.clear();
        self.version += 1;
        if self.max_steps > 0 {
            while self.actions.len() > self.max_steps as usize {
                self.actions.remove(0);
            }
        }
        true
    }

    pub fn has_undo(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }

    pub fn get_history_count(&self) -> usize {
        self.actions.len()
    }

    pub fn get_redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn get_current_action_name(&self) -> String {
        self.actions
            .last()
            .map(|action| action.name.clone())
            .unwrap_or_default()
    }

    /// `undo`: applies the last action's undo list (reversed unless the action
    /// asked for forward order) and moves it to the redo stack.
    pub fn undo<F: FnMut(&UndoRedoOp)>(&mut self, mut apply: F) -> bool {
        let Some(action) = self.actions.pop() else {
            return false;
        };
        if action.backward_undo_ops {
            for op in &action.undo_ops {
                apply(op);
            }
        } else {
            for op in action.undo_ops.iter().rev() {
                apply(op);
            }
        }
        self.redo_stack.push(action);
        self.version += 1;
        true
    }

    /// `redo`: reapplies the last undone action.
    pub fn redo<F: FnMut(&UndoRedoOp)>(&mut self, mut apply: F) -> bool {
        let Some(action) = self.redo_stack.pop() else {
            return false;
        };
        for op in &action.do_ops {
            apply(op);
        }
        self.actions.push(action);
        self.version += 1;
        true
    }

    pub fn clear_history(&mut self) {
        self.actions.clear();
        self.redo_stack.clear();
        self.current = None;
        self.version += 1;
    }

    pub fn set_max_steps(&mut self, steps: i32) {
        self.max_steps = steps;
        if self.max_steps > 0 {
            while self.actions.len() > self.max_steps as usize {
                self.actions.remove(0);
            }
        }
    }

    pub fn get_max_steps(&self) -> i32 {
        self.max_steps
    }

    pub fn get_action_level(&self) -> i32 {
        self.action_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u64) -> ObjectId {
        ObjectId::from_u64(value)
    }

    #[test]
    fn action_commit_and_stacks() {
        let mut undo = UndoRedoData::new();
        assert!(!undo.has_undo());
        assert!(!undo.commit_action(), "nothing to commit");
        assert!(undo.create_action("move", MergeMode::Disable, false));
        assert!(undo.is_committing_action());
        assert!(undo.add_do_property(id(1), "position", Variant::Int(10)));
        assert!(undo.add_undo_property(id(1), "position", Variant::Int(0)));
        assert!(undo.add_do_method(id(1), "notify", vec![Variant::Int(1)]));
        assert!(undo.add_undo_method(id(1), "notify", vec![Variant::Int(2)]));
        assert!(undo.commit_action());
        assert!(!undo.is_committing_action());
        assert!(undo.has_undo());
        assert!(!undo.has_redo());
        assert_eq!(undo.get_history_count(), 1);
        assert_eq!(undo.get_current_action_name(), "move");
        let version = undo.get_version();
        assert!(version >= 2);
    }

    #[test]
    fn undo_is_reversed_and_redo_forward() {
        let mut undo = UndoRedoData::new();
        undo.create_action("a", MergeMode::Disable, false);
        // Two do ops; the undo list is applied in reverse.
        undo.add_do_property(id(1), "p", Variant::Int(1));
        undo.add_do_property(id(1), "q", Variant::Int(2));
        undo.add_undo_property(id(1), "p", Variant::Int(0));
        undo.add_undo_property(id(1), "q", Variant::Int(0));
        undo.commit_action();

        let mut seen = Vec::new();
        assert!(undo.undo(|op| {
            if let UndoRedoOp::Property { property, .. } = op {
                seen.push(property.as_string());
            }
        }));
        assert_eq!(seen, vec!["q", "p"]);
        assert!(!undo.has_undo());
        assert!(undo.has_redo());

        let mut redone = Vec::new();
        assert!(undo.redo(|op| {
            if let UndoRedoOp::Property { property, .. } = op {
                redone.push(property.as_string());
            }
        }));
        assert_eq!(redone, vec!["p", "q"]);
        assert!(undo.has_undo());
        assert!(!undo.has_redo());
        assert!(!undo.redo(|_| {}), "nothing left to redo");
    }

    #[test]
    fn backward_undo_ops_run_forward() {
        let mut undo = UndoRedoData::new();
        undo.create_action("a", MergeMode::Disable, true);
        undo.add_undo_property(id(1), "p", Variant::Int(1));
        undo.add_undo_property(id(1), "q", Variant::Int(2));
        undo.commit_action();
        let mut seen = Vec::new();
        undo.undo(|op| {
            if let UndoRedoOp::Property { property, .. } = op {
                seen.push(property.as_string());
            }
        });
        assert_eq!(seen, vec!["p", "q"]);
    }

    #[test]
    fn merging_actions() {
        let mut undo = UndoRedoData::new();
        undo.create_action("paint", MergeMode::All, false);
        undo.add_do_property(id(1), "a", Variant::Int(1));
        undo.add_undo_property(id(1), "a", Variant::Int(0));
        undo.commit_action();

        // Same name + MERGE_ALL appends to the previous action.
        undo.create_action("paint", MergeMode::All, false);
        undo.add_do_property(id(1), "b", Variant::Int(2));
        undo.add_undo_property(id(1), "b", Variant::Int(0));
        undo.commit_action();
        assert_eq!(undo.get_history_count(), 1);

        // A different name starts a new action.
        undo.create_action("erase", MergeMode::All, false);
        undo.commit_action();
        assert_eq!(undo.get_history_count(), 2);

        // MERGE_DISABLE never merges.
        undo.create_action("paint", MergeMode::Disable, false);
        undo.commit_action();
        assert_eq!(undo.get_history_count(), 3);

        // MERGE_ENDS merges into a preceding MERGE_ENDS action only.
        let mut ends = UndoRedoData::new();
        ends.create_action("x", MergeMode::Ends, false);
        ends.commit_action();
        ends.create_action("x", MergeMode::Ends, false);
        ends.commit_action();
        assert_eq!(ends.get_history_count(), 1);
        ends.create_action("x", MergeMode::All, false);
        ends.commit_action();
        assert_eq!(ends.get_history_count(), 1);
        ends.create_action("y", MergeMode::Ends, false);
        ends.commit_action();
        assert_eq!(ends.get_history_count(), 2);

        // The force-keep counter blocks merging.
        let mut forced = UndoRedoData::new();
        forced.create_action("z", MergeMode::All, false);
        forced.commit_action();
        forced.start_force_keep_in_merge_ends();
        forced.create_action("z", MergeMode::All, false);
        forced.commit_action();
        assert_eq!(forced.get_history_count(), 2);
        forced.end_force_keep_in_merge_ends();
        forced.create_action("z", MergeMode::All, false);
        forced.commit_action();
        assert_eq!(forced.get_history_count(), 2, "merged into the previous z");
    }

    #[test]
    fn max_steps_and_clear() {
        let mut undo = UndoRedoData::new();
        undo.set_max_steps(2);
        for name in ["a", "b", "c"] {
            undo.create_action(name, MergeMode::Disable, false);
            undo.commit_action();
        }
        assert_eq!(undo.get_history_count(), 2);
        assert_eq!(undo.get_current_action_name(), "c");

        undo.undo(|_| {});
        assert!(undo.has_redo());
        undo.clear_history();
        assert!(!undo.has_undo());
        assert!(!undo.has_redo());
        assert_eq!(undo.get_max_steps(), 2);
    }

    #[test]
    fn invalid_targets_and_no_action() {
        let mut undo = UndoRedoData::new();
        assert!(!undo.add_do_method(id(1), "x", Vec::new()));
        undo.create_action("a", MergeMode::Disable, false);
        assert!(!undo.add_do_method(ObjectId::NONE, "x", Vec::new()));
        assert!(!undo.add_do_property(ObjectId::NONE, "p", Variant::Nil));
        assert!(!undo.add_do_reference(ObjectId::NONE));
        assert!(undo.add_do_reference(id(2)));
        assert!(undo.commit_action());
        assert!(!undo.commit_action());
    }
}
