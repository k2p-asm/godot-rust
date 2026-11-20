//! `core/io/resource.*`: resource identity and change emission state.
//!
//! Ported: `name`, `path`/path cache, `scene_unique_id`, `local_to_scene`, the
//! built-in path test (`empty`, contains `::`, or starts with `local://`) and the
//! block/unblock change-emission state machine:
//!
//! - `emit_changed` while blocked only marks a pending emission;
//! - `block` only moves `Unblocked -> Blocked` (a pending state stays pending);
//! - `unblock` returns to `Unblocked` and emits once if something was pending.
//!
//! Resource cache takeover, duplication, local-scene setup and the `changed`
//! signal delivery are not ported; `changed_count` counts emissions for tests.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EmitChangedState {
    Unblocked,
    Blocked,
    BlockedPendingEmit,
}

#[derive(Clone, Debug)]
pub struct ResourceData {
    name: String,
    path_cache: String,
    scene_unique_id: String,
    local_to_scene: bool,
    emit_changed_state: EmitChangedState,
    changed_count: u64,
}

impl Default for ResourceData {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceData {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            path_cache: String::new(),
            scene_unique_id: String::new(),
            local_to_scene: false,
            emit_changed_state: EmitChangedState::Unblocked,
            changed_count: 0,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets the cached path; take-over only matters with a resource cache.
    pub fn set_path(&mut self, path: &str, take_over: bool) {
        if self.path_cache == path {
            return;
        }
        let _ = take_over && !path.is_empty();
        self.path_cache = path.to_string();
    }

    pub fn set_path_cache(&mut self, path: &str) {
        self.path_cache = path.to_string();
    }

    pub fn get_path(&self) -> &str {
        &self.path_cache
    }

    /// Empty, containing `::`, or a `local://` path.
    pub fn is_built_in(&self) -> bool {
        self.path_cache.is_empty()
            || self.path_cache.contains("::")
            || self.path_cache.starts_with("local://")
    }

    pub fn set_scene_unique_id(&mut self, id: &str) {
        self.scene_unique_id = id.to_string();
    }

    pub fn get_scene_unique_id(&self) -> &str {
        &self.scene_unique_id
    }

    pub fn set_local_to_scene(&mut self, enable: bool) {
        self.local_to_scene = enable;
    }

    pub fn is_local_to_scene(&self) -> bool {
        self.local_to_scene
    }

    /// Emits immediately unless blocked; returns whether it emitted.
    pub fn emit_changed(&mut self) -> bool {
        if self.emit_changed_state != EmitChangedState::Unblocked {
            self.emit_changed_state = EmitChangedState::BlockedPendingEmit;
            return false;
        }
        self.changed_count += 1;
        true
    }

    pub fn block_emit_changed(&mut self) {
        if self.emit_changed_state == EmitChangedState::Unblocked {
            self.emit_changed_state = EmitChangedState::Blocked;
        }
    }

    /// Unblocks and flushes one pending emission.
    pub fn unblock_emit_changed(&mut self) -> bool {
        let emit = self.emit_changed_state == EmitChangedState::BlockedPendingEmit;
        self.emit_changed_state = EmitChangedState::Unblocked;
        if emit {
            return self.emit_changed();
        }
        false
    }

    pub fn is_emit_changed_blocked(&self) -> bool {
        self.emit_changed_state != EmitChangedState::Unblocked
    }

    pub fn get_changed_count(&self) -> u64 {
        self.changed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_defaults() {
        let mut r = ResourceData::new();
        assert_eq!(r.get_name(), "");
        assert_eq!(r.get_path(), "");
        assert_eq!(r.get_scene_unique_id(), "");
        assert!(!r.is_local_to_scene());
        assert!(r.is_built_in());
        assert!(!r.is_emit_changed_blocked());
        assert_eq!(r.get_changed_count(), 0);

        r.set_name("MyRes");
        r.set_scene_unique_id("abc123");
        r.set_local_to_scene(true);
        assert_eq!(r.get_name(), "MyRes");
        assert_eq!(r.get_scene_unique_id(), "abc123");
        assert!(r.is_local_to_scene());
    }

    #[test]
    fn paths_and_built_in_detection() {
        let mut r = ResourceData::new();
        r.set_path("res://a.tres", false);
        assert_eq!(r.get_path(), "res://a.tres");
        assert!(!r.is_built_in());

        // Cache-only setter bypasses any cache logic.
        r.set_path_cache("res://b.tres");
        assert_eq!(r.get_path(), "res://b.tres");

        for built_in in ["", "res://scene.tscn::1", "local://x"] {
            r.set_path_cache(built_in);
            assert!(r.is_built_in(), "{built_in}");
        }

        // Setting the same path is a no-op; empty paths stay empty.
        r.set_path("", false);
        assert_eq!(r.get_path(), "");
    }

    #[test]
    fn emit_blocking_state_machine() {
        let mut r = ResourceData::new();
        assert!(r.emit_changed());
        assert_eq!(r.get_changed_count(), 1);

        r.block_emit_changed();
        assert!(r.is_emit_changed_blocked());
        assert!(!r.emit_changed());
        assert_eq!(r.get_changed_count(), 1);
        // Blocking again while pending keeps the pending emission.
        r.block_emit_changed();
        assert!(r.unblock_emit_changed(), "flush pending");
        assert_eq!(r.get_changed_count(), 2);
        assert!(!r.is_emit_changed_blocked());

        // Unblocking with nothing pending does not emit.
        r.block_emit_changed();
        assert!(!r.unblock_emit_changed());
        assert_eq!(r.get_changed_count(), 2);

        // A normal emission after unblocking works again.
        assert!(r.emit_changed());
        assert_eq!(r.get_changed_count(), 3);
    }
}
