//! `scene/gui/color_picker_shape.*` (state): the picker shape base.
//!
//! Ported state: the drag flag and the owning color picker handle. The shader
//! caches, the control initialisation and the cursor/drawing helpers are not
//! ported.
//!
//! `can_handle` needs `InputEvent` hit testing against the shape controls, so it
//! stays pending.

use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct ColorPickerShapeData {
    color_picker: Option<Rid>,
    is_dragging: bool,
}

impl ColorPickerShapeData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_color_picker(&mut self, picker: Option<Rid>) {
        self.color_picker = picker;
    }

    pub fn get_color_picker(&self) -> Option<Rid> {
        self.color_picker
    }

    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut shape = ColorPickerShapeData::new();
        assert!(shape.get_color_picker().is_none());
        assert!(!shape.is_dragging());

        shape.set_color_picker(Some(Rid::from_u64(11)));
        shape.set_dragging(true);
        assert_eq!(shape.get_color_picker(), Some(Rid::from_u64(11)));
        assert!(shape.is_dragging());

        shape.set_dragging(false);
        shape.set_color_picker(None);
        assert!(!shape.is_dragging());
        assert!(shape.get_color_picker().is_none());
    }
}
