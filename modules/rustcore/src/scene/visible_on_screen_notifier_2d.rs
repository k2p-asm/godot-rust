//! `scene/2d/visible_on_screen_notifier_2d.*`: visibility-notifier state.
//!
//! `VisibleOnScreenNotifier2D` keeps its rect (default `(-10, -10, 20, 20)`),
//! the editor `show_rect` flag (on) and the last on-screen state (off). The
//! engine flips the latter from the RenderingServer callback; the port exposes
//! `set_on_screen` for that hook and for tests.
//!
//! `VisibleOnScreenEnabler2D` adds the enable mode (inherit by default) and the
//! enable node path (default `".."`).
//!
//! Not ported: the RenderingServer notifier registration, the `screen_entered`
//! /`screen_exited` signals and the process-mode toggling.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct VisibleOnScreenNotifier2DData {
    rect: Rect2,
    show_rect: bool,
    on_screen: bool,
}

impl Default for VisibleOnScreenNotifier2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenNotifier2DData {
    pub fn new() -> Self {
        Self {
            rect: Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)),
            show_rect: true,
            on_screen: false,
        }
    }

    pub fn set_rect(&mut self, rect: Rect2) {
        self.rect = rect;
    }

    pub fn get_rect(&self) -> Rect2 {
        self.rect
    }

    pub fn set_show_rect(&mut self, show: bool) {
        self.show_rect = show;
    }

    pub fn is_showing_rect(&self) -> bool {
        self.show_rect
    }

    pub fn is_on_screen(&self) -> bool {
        self.on_screen
    }

    /// Hook used by the visibility callback (and by tests).
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
pub struct VisibleOnScreenEnabler2DData {
    pub notifier: VisibleOnScreenNotifier2DData,
    enable_mode: EnableMode,
    enable_node_path: String,
}

impl Default for VisibleOnScreenEnabler2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenEnabler2DData {
    pub fn new() -> Self {
        Self {
            notifier: VisibleOnScreenNotifier2DData::new(),
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

    /// The path is stored as text; the engine keeps a `NodePath`.
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
        let mut n = VisibleOnScreenNotifier2DData::new();
        assert_eq!(
            n.get_rect(),
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        );
        assert!(n.is_showing_rect());
        assert!(!n.is_on_screen());

        n.set_on_screen(true);
        assert!(n.is_on_screen());
        n.set_on_screen(false);
        assert!(!n.is_on_screen());

        n.set_rect(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));
        n.set_show_rect(false);
        assert_eq!(n.get_rect().position, Vector2::new(1.0, 2.0));
        assert_eq!(n.get_rect().size, Vector2::new(3.0, 4.0));
        assert!(!n.is_showing_rect());
    }

    #[test]
    fn enabler_defaults_and_flags() {
        let mut e = VisibleOnScreenEnabler2DData::new();
        assert_eq!(e.get_enable_mode(), EnableMode::Inherit);
        assert_eq!(e.get_enable_node_path(), "..");
        assert!(!e.notifier.is_on_screen());

        e.set_enable_mode(EnableMode::WhenPaused);
        e.set_enable_node_path("../Target");
        assert_eq!(e.get_enable_mode(), EnableMode::WhenPaused);
        assert_eq!(e.get_enable_node_path(), "../Target");

        e.set_enable_node_path("");
        assert_eq!(e.get_enable_node_path(), "");

        assert_eq!(EnableMode::Inherit as i32, 0);
        assert_eq!(EnableMode::Always as i32, 1);
        assert_eq!(EnableMode::WhenPaused as i32, 2);
    }
}
