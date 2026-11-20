//! `scene/3d/visible_on_screen_notifier_3d.*`: 3D visibility-notifier state.
//!
//! The notifier keeps an AABB (default `(-1,-1,-1)` + `(2,2,2)`), the last
//! on-screen state (off) and the VisualInstance layer/sorting state. The enabler
//! adds the three enable modes (inherit by default) and the `..` node path.
//! Server registration and the enter/exit signals are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;
use crate::scene::visual_instance_3d::VisualInstance3DData;

#[derive(Clone, Copy, Debug)]
pub struct VisibleOnScreenNotifier3DData {
    pub visual: VisualInstance3DData,
    aabb: Aabb,
    on_screen: bool,
}

impl Default for VisibleOnScreenNotifier3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenNotifier3DData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            aabb: Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(2.0, 2.0, 2.0)),
            on_screen: false,
        }
    }

    pub fn set_aabb(&mut self, aabb: Aabb) {
        self.aabb = aabb;
    }

    pub fn get_aabb(&self) -> Aabb {
        self.aabb
    }

    pub fn is_on_screen(&self) -> bool {
        self.on_screen
    }

    /// Hook used by the visibility callback (and tests).
    pub fn set_on_screen(&mut self, on_screen: bool) {
        self.on_screen = on_screen;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EnableMode {
    Inherit,
    Always,
    WhenPaused,
}

#[derive(Clone, Debug)]
pub struct VisibleOnScreenEnabler3DData {
    pub notifier: VisibleOnScreenNotifier3DData,
    enable_mode: EnableMode,
    enable_node_path: String,
}

impl Default for VisibleOnScreenEnabler3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenEnabler3DData {
    pub fn new() -> Self {
        Self {
            notifier: VisibleOnScreenNotifier3DData::new(),
            enable_mode: EnableMode::Inherit,
            enable_node_path: "..".to_string(),
        }
    }

    pub fn set_enable_mode(&mut self, mode: EnableMode) {
        self.enable_mode = mode;
    }

    pub fn get_enable_mode(&self) -> EnableMode {
        self.enable_mode
    }

    pub fn set_enable_node_path(&mut self, path: &str) {
        self.enable_node_path = path.to_string();
    }

    pub fn get_enable_node_path(&self) -> &str {
        &self.enable_node_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notifier_defaults_and_state() {
        let mut n = VisibleOnScreenNotifier3DData::new();
        assert_eq!(
            n.get_aabb(),
            Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(2.0, 2.0, 2.0))
        );
        assert!(!n.is_on_screen());
        assert_eq!(n.visual.get_layer_mask(), 1);

        n.set_on_screen(true);
        assert!(n.is_on_screen());
        n.set_aabb(Aabb::new(Vector3::ZERO, Vector3::new(1.0, 2.0, 3.0)));
        assert_eq!(n.get_aabb().size, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn enabler_defaults() {
        let mut e = VisibleOnScreenEnabler3DData::new();
        assert_eq!(e.get_enable_mode(), EnableMode::Inherit);
        assert_eq!(e.get_enable_node_path(), "..");
        assert!(!e.notifier.is_on_screen());

        e.set_enable_mode(EnableMode::Always);
        e.set_enable_node_path("../Target");
        assert_eq!(e.get_enable_mode(), EnableMode::Always);
        assert_eq!(e.get_enable_node_path(), "../Target");

        assert_eq!(EnableMode::Inherit as i32, 0);
        assert_eq!(EnableMode::Always as i32, 1);
        assert_eq!(EnableMode::WhenPaused as i32, 2);
    }
}
