//! `scene/gui/subviewport_container.*`: the sub-viewport container's state.
//!
//! Defaults: `stretch` off, `shrink` 1, `mouse_target` off. The accessors mirror
//! the engine's, with `set_stretch_shrink` clamping to at least 1 like
//! `MAX(1, p_shrink)` in the header's user code path.
//!
//! Viewport sizing, input propagation and notifications are not ported.

#[derive(Clone, Debug)]
pub struct SubViewportContainerData {
    stretch: bool,
    shrink: i32,
    mouse_target: bool,
}

impl Default for SubViewportContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl SubViewportContainerData {
    pub fn new() -> Self {
        Self {
            stretch: false,
            shrink: 1,
            mouse_target: false,
        }
    }

    pub fn set_stretch(&mut self, enabled: bool) {
        self.stretch = enabled;
    }

    pub fn is_stretch_enabled(&self) -> bool {
        self.stretch
    }

    /// `set_stretch_shrink`: values below 1 are clamped.
    pub fn set_stretch_shrink(&mut self, shrink: i32) {
        self.shrink = shrink.max(1);
    }

    pub fn get_stretch_shrink(&self) -> i32 {
        self.shrink
    }

    pub fn set_mouse_target(&mut self, enabled: bool) {
        self.mouse_target = enabled;
    }

    pub fn is_mouse_target_enabled(&self) -> bool {
        self.mouse_target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let container = SubViewportContainerData::new();
        assert!(!container.is_stretch_enabled());
        assert_eq!(container.get_stretch_shrink(), 1);
        assert!(!container.is_mouse_target_enabled());
    }

    #[test]
    fn setters_and_shrink_clamp() {
        let mut container = SubViewportContainerData::new();
        container.set_stretch(true);
        container.set_mouse_target(true);
        assert!(container.is_stretch_enabled());
        assert!(container.is_mouse_target_enabled());

        container.set_stretch_shrink(4);
        assert_eq!(container.get_stretch_shrink(), 4);
        container.set_stretch_shrink(0);
        assert_eq!(container.get_stretch_shrink(), 1, "shrink is at least 1");
        container.set_stretch_shrink(-3);
        assert_eq!(container.get_stretch_shrink(), 1);
    }
}
