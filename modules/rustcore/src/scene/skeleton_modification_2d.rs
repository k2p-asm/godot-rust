//! `scene/resources/2d/skeleton/skeleton_modification_{stack,}2d.*` (state).
//!
//! `SkeletonModificationStack2D` holds the modification list plus the skeleton
//! handle, the setup flag, `enabled` (off by default), `strength` (1.0), the
//! execution mode (process or physics) and the editor-gizmo dirty flag.
//!
//! `SkeletonModification2D` holds its own `enabled` (on), setup flag, execution
//! mode (0 = process) and the editor gizmo flag.
//!
//! Ported: the list operations (`add`, `delete`, `set`, `count`,
//! `enable_all_modifications`) and every flag. `setup()`/`execute()` only update
//! the setup flags; the per-modification solvers, gizmos and the stack back
//! pointer's evaluation are not ported. `clamp_angle` is not ported (it depends on
//! the solver's bounds semantics).

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ModificationExecutionMode {
    Process,
    PhysicsProcess,
}

#[derive(Clone, Debug)]
pub struct SkeletonModification2DData {
    execution_mode: ModificationExecutionMode,
    enabled: bool,
    is_setup: bool,
    editor_draw_gizmo: bool,
}

impl Default for SkeletonModification2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonModification2DData {
    pub fn new() -> Self {
        Self {
            execution_mode: ModificationExecutionMode::Process,
            enabled: true,
            is_setup: false,
            editor_draw_gizmo: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    /// `ERR_FAIL_INDEX(p_mode, 2)`.
    pub fn set_execution_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => ModificationExecutionMode::Process,
            1 => ModificationExecutionMode::PhysicsProcess,
            _ => return false,
        };
        self.execution_mode = parsed;
        true
    }

    pub fn get_execution_mode(&self) -> ModificationExecutionMode {
        self.execution_mode
    }

    pub fn set_is_setup(&mut self, is_setup: bool) {
        self.is_setup = is_setup;
    }

    pub fn get_is_setup(&self) -> bool {
        self.is_setup
    }

    pub fn set_editor_draw_gizmo(&mut self, draw: bool) {
        self.editor_draw_gizmo = draw;
    }

    pub fn get_editor_draw_gizmo(&self) -> bool {
        self.editor_draw_gizmo
    }
}

#[derive(Clone, Debug)]
pub struct SkeletonModificationStack2DData {
    skeleton: Option<Rid>,
    is_setup: bool,
    enabled: bool,
    strength: f32,
    execution_mode: ModificationExecutionMode,
    modifications: Vec<SkeletonModification2DData>,
    editor_gizmo_dirty: bool,
}

impl Default for SkeletonModificationStack2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonModificationStack2DData {
    pub fn new() -> Self {
        Self {
            skeleton: None,
            is_setup: false,
            enabled: false,
            strength: 1.0,
            execution_mode: ModificationExecutionMode::Process,
            modifications: Vec::new(),
            editor_gizmo_dirty: false,
        }
    }

    pub fn get_modification_count(&self) -> usize {
        self.modifications.len()
    }

    pub fn get_modification(&self, index: usize) -> Option<&SkeletonModification2DData> {
        self.modifications.get(index)
    }

    pub fn get_modification_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut SkeletonModification2DData> {
        self.modifications.get_mut(index)
    }

    pub fn add_modification(&mut self, modification: SkeletonModification2DData) -> usize {
        self.modifications.push(modification);
        self.modifications.len() - 1
    }

    pub fn delete_modification(&mut self, index: usize) -> bool {
        if index >= self.modifications.len() {
            return false;
        }
        self.modifications.remove(index);
        true
    }

    pub fn set_modification(
        &mut self,
        index: usize,
        modification: SkeletonModification2DData,
    ) -> bool {
        match self.modifications.get_mut(index) {
            Some(slot) => {
                *slot = modification;
                true
            }
            None => false,
        }
    }

    /// `set_modification_count`: grows with default modifications, shrinks by
    /// dropping the tail.
    pub fn set_modification_count(&mut self, count: usize) {
        self.modifications
            .resize_with(count, SkeletonModification2DData::new);
    }

    /// `enable_all_modifications`.
    pub fn enable_all_modifications(&mut self, enable: bool) {
        for modification in self.modifications.iter_mut() {
            modification.enabled = enable;
        }
    }

    pub fn set_skeleton(&mut self, skeleton: Option<Rid>) {
        self.skeleton = skeleton;
    }

    pub fn get_skeleton(&self) -> Option<Rid> {
        self.skeleton
    }

    /// `setup()`: marks the stack (and its modifications) as set up.
    pub fn setup(&mut self) {
        self.is_setup = true;
        for modification in self.modifications.iter_mut() {
            modification.is_setup = true;
        }
    }

    pub fn get_is_setup(&self) -> bool {
        self.is_setup
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn get_strength(&self) -> f32 {
        self.strength
    }

    pub fn set_execution_mode(&mut self, mode: ModificationExecutionMode) {
        self.execution_mode = mode;
    }

    pub fn get_execution_mode(&self) -> ModificationExecutionMode {
        self.execution_mode
    }

    pub fn set_editor_gizmos_dirty(&mut self, dirty: bool) {
        self.editor_gizmo_dirty = dirty;
    }

    pub fn is_editor_gizmos_dirty(&self) -> bool {
        self.editor_gizmo_dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modification_defaults() {
        let mut m = SkeletonModification2DData::new();
        assert!(m.get_enabled());
        assert!(!m.get_is_setup());
        assert!(!m.get_editor_draw_gizmo());
        assert_eq!(m.get_execution_mode(), ModificationExecutionMode::Process);

        m.set_enabled(false);
        assert!(!m.get_enabled());
        assert!(m.set_execution_mode(1));
        assert_eq!(
            m.get_execution_mode(),
            ModificationExecutionMode::PhysicsProcess
        );
        assert!(!m.set_execution_mode(2));
        m.set_is_setup(true);
        m.set_editor_draw_gizmo(true);
        assert!(m.get_is_setup());
        assert!(m.get_editor_draw_gizmo());

        assert_eq!(ModificationExecutionMode::Process as i32, 0);
        assert_eq!(ModificationExecutionMode::PhysicsProcess as i32, 1);
    }

    #[test]
    fn stack_defaults_and_modifications() {
        let mut s = SkeletonModificationStack2DData::new();
        assert!(s.get_skeleton().is_none());
        assert!(!s.get_is_setup());
        assert!(!s.get_enabled());
        assert_eq!(s.get_strength(), 1.0);
        assert_eq!(s.get_modification_count(), 0);
        assert!(s.get_modification(0).is_none());
        assert!(!s.is_editor_gizmos_dirty());

        assert_eq!(s.add_modification(SkeletonModification2DData::new()), 0);
        assert_eq!(s.add_modification(SkeletonModification2DData::new()), 1);
        assert_eq!(s.get_modification_count(), 2);
        assert!(s.get_modification(1).is_some());

        assert!(s.set_modification(
            0,
            SkeletonModification2DData {
                enabled: false,
                ..SkeletonModification2DData::new()
            }
        ));
        assert!(!s.get_modification(0).unwrap().get_enabled());
        assert!(!s.set_modification(9, SkeletonModification2DData::new()));

        // Counting up fills with enabled defaults, counting down drops the tail.
        s.set_modification_count(4);
        assert_eq!(s.get_modification_count(), 4);
        assert!(s.get_modification(3).unwrap().get_enabled());
        s.set_modification_count(1);
        assert_eq!(s.get_modification_count(), 1);

        s.enable_all_modifications(false);
        assert!(!s.get_modification(0).unwrap().get_enabled());
        assert!(s.delete_modification(0));
        assert!(!s.delete_modification(5));
        assert_eq!(s.get_modification_count(), 0);
    }

    #[test]
    fn stack_flags_and_setup() {
        let mut s = SkeletonModificationStack2DData::new();
        s.set_skeleton(Some(Rid::from_u64(3)));
        s.set_enabled(true);
        s.set_strength(0.5);
        s.set_execution_mode(ModificationExecutionMode::PhysicsProcess);
        s.set_editor_gizmos_dirty(true);

        assert_eq!(s.get_skeleton(), Some(Rid::from_u64(3)));
        assert!(s.get_enabled());
        assert_eq!(s.get_strength(), 0.5);
        assert_eq!(
            s.get_execution_mode(),
            ModificationExecutionMode::PhysicsProcess
        );
        assert!(s.is_editor_gizmos_dirty());

        s.add_modification(SkeletonModification2DData::new());
        assert!(!s.get_modification(0).unwrap().get_is_setup());
        s.setup();
        assert!(s.get_is_setup());
        assert!(s.get_modification(0).unwrap().get_is_setup());
    }
}
