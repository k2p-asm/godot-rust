//! `scene/gui/graph_frame.*`: the graph frame's state.
//!
//! `GraphFrame` is a `GraphElement` panel that moves its child `GraphNode`s.
//! Ported defaults: empty title, autoshrink on with a 40 px margin, drag margin
//! 16, tinting off and the engine's tint `(0.3, 0.3, 0.3, 0.75)`.
//!
//! Dragging, resizing, autoshrink computation and drawing are not ported.

use crate::math::color::Color;

#[derive(Clone, Debug)]
pub struct GraphFrameData {
    title: String,
    autoshrink_enabled: bool,
    autoshrink_margin: i32,
    drag_margin: i32,
    tint_color_enabled: bool,
    tint_color: Color,
}

impl Default for GraphFrameData {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphFrameData {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            autoshrink_enabled: true,
            autoshrink_margin: 40,
            drag_margin: 16,
            tint_color_enabled: false,
            tint_color: Color::new(0.3, 0.3, 0.3, 0.75),
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_autoshrink_enabled(&mut self, enabled: bool) {
        self.autoshrink_enabled = enabled;
    }

    pub fn is_autoshrink_enabled(&self) -> bool {
        self.autoshrink_enabled
    }

    pub fn set_autoshrink_margin(&mut self, margin: i32) {
        self.autoshrink_margin = margin;
    }

    pub fn get_autoshrink_margin(&self) -> i32 {
        self.autoshrink_margin
    }

    pub fn set_drag_margin(&mut self, margin: i32) {
        self.drag_margin = margin;
    }

    pub fn get_drag_margin(&self) -> i32 {
        self.drag_margin
    }

    pub fn set_tint_color_enabled(&mut self, enabled: bool) {
        self.tint_color_enabled = enabled;
    }

    pub fn is_tint_color_enabled(&self) -> bool {
        self.tint_color_enabled
    }

    pub fn set_tint_color(&mut self, color: Color) {
        self.tint_color = color;
    }

    pub fn get_tint_color(&self) -> Color {
        self.tint_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut frame = GraphFrameData::new();
        assert_eq!(frame.get_title(), "");
        assert!(frame.is_autoshrink_enabled());
        assert_eq!(frame.get_autoshrink_margin(), 40);
        assert_eq!(frame.get_drag_margin(), 16);
        assert!(!frame.is_tint_color_enabled());
        assert_eq!(frame.get_tint_color(), Color::new(0.3, 0.3, 0.3, 0.75));

        frame.set_title("Group");
        frame.set_autoshrink_enabled(false);
        frame.set_autoshrink_margin(64);
        frame.set_drag_margin(8);
        frame.set_tint_color_enabled(true);
        frame.set_tint_color(Color::new(1.0, 0.5, 0.25, 1.0));

        assert_eq!(frame.get_title(), "Group");
        assert!(!frame.is_autoshrink_enabled());
        assert_eq!(frame.get_autoshrink_margin(), 64);
        assert_eq!(frame.get_drag_margin(), 8);
        assert!(frame.is_tint_color_enabled());
        assert_eq!(frame.get_tint_color(), Color::new(1.0, 0.5, 0.25, 1.0));
    }
}
