//! `scene/gui/graph_node.*` (slot state only).
//!
//! A `GraphNode` owns a title and a list of slots; each slot records both port
//! sides (enabled, type, colour, custom icon), whether the stylebox is drawn and
//! optional metadata. Defaults: empty title, no slots, no selected slot and
//! invalid connection types are respected.
//!
//! Drawing, port positioning, resizing and input are not ported; the slot table
//! is exposed with the engine's create/clear/field accessors.

use crate::math::color::Color;
use crate::servers::rid::Rid;
use crate::variant::Variant;

#[derive(Clone, Debug)]
pub struct GraphSlot {
    pub enable_left: bool,
    pub type_left: i32,
    pub color_left: Color,
    pub enable_right: bool,
    pub type_right: i32,
    pub color_right: Color,
    pub custom_left: Option<Rid>,
    pub custom_right: Option<Rid>,
    pub draw_stylebox: bool,
    pub metadata: Option<Variant>,
}

impl Default for GraphSlot {
    fn default() -> Self {
        Self {
            enable_left: false,
            type_left: 0,
            color_left: Color::new(1.0, 1.0, 1.0, 1.0),
            enable_right: false,
            type_right: 0,
            color_right: Color::new(1.0, 1.0, 1.0, 1.0),
            custom_left: None,
            custom_right: None,
            draw_stylebox: true,
            metadata: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GraphNodeData {
    title: String,
    slots: Vec<GraphSlot>,
    selected_slot: i32,
    ignore_invalid_connection_type: bool,
}

impl Default for GraphNodeData {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphNodeData {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            slots: Vec::new(),
            selected_slot: -1,
            ignore_invalid_connection_type: false,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn get_slot(&self, index: usize) -> Option<&GraphSlot> {
        self.slots.get(index)
    }

    fn ensure_slot(&mut self, index: usize) -> Option<&mut GraphSlot> {
        if self.slots.len() <= index {
            self.slots.resize(index + 1, GraphSlot::default());
        }
        self.slots.get_mut(index)
    }

    /// `set_slot`: grows the slot list as needed.
    #[allow(clippy::too_many_arguments)]
    pub fn set_slot(
        &mut self,
        index: usize,
        enable_left: bool,
        type_left: i32,
        color_left: Color,
        enable_right: bool,
        type_right: i32,
        color_right: Color,
        custom_left: Option<Rid>,
        custom_right: Option<Rid>,
        draw_stylebox: bool,
    ) {
        let slot = self.ensure_slot(index).expect("grown");
        slot.enable_left = enable_left;
        slot.type_left = type_left;
        slot.color_left = color_left;
        slot.enable_right = enable_right;
        slot.type_right = type_right;
        slot.color_right = color_right;
        slot.custom_left = custom_left;
        slot.custom_right = custom_right;
        slot.draw_stylebox = draw_stylebox;
    }

    /// `clear_slot`: resets the entry to its defaults.
    pub fn clear_slot(&mut self, index: usize) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                *slot = GraphSlot::default();
                true
            }
            None => false,
        }
    }

    pub fn clear_all_slots(&mut self) {
        self.slots.clear();
        self.selected_slot = -1;
    }

    pub fn is_slot_enabled_left(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .map(|slot| slot.enable_left)
            .unwrap_or(false)
    }

    pub fn set_slot_enabled_left(&mut self, index: usize, enabled: bool) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.enable_left = enabled;
                true
            }
            None => false,
        }
    }

    pub fn set_slot_type_left(&mut self, index: usize, kind: i32) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.type_left = kind;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_type_left(&self, index: usize) -> i32 {
        self.slots.get(index).map(|slot| slot.type_left).unwrap_or(0)
    }

    pub fn set_slot_color_left(&mut self, index: usize, color: Color) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.color_left = color;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_color_left(&self, index: usize) -> Color {
        self.slots
            .get(index)
            .map(|slot| slot.color_left)
            .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn set_slot_custom_icon_left(&mut self, index: usize, icon: Option<Rid>) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.custom_left = icon;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_custom_icon_left(&self, index: usize) -> Option<Rid> {
        self.slots.get(index).and_then(|slot| slot.custom_left)
    }

    pub fn set_slot_metadata_left(&mut self, index: usize, value: Variant) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.metadata = Some(value);
                true
            }
            None => false,
        }
    }

    pub fn get_slot_metadata_left(&self, index: usize) -> Option<Variant> {
        self.slots.get(index).and_then(|slot| slot.metadata.clone())
    }

    pub fn is_slot_enabled_right(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .map(|slot| slot.enable_right)
            .unwrap_or(false)
    }

    pub fn set_slot_enabled_right(&mut self, index: usize, enabled: bool) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.enable_right = enabled;
                true
            }
            None => false,
        }
    }

    pub fn set_slot_type_right(&mut self, index: usize, kind: i32) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.type_right = kind;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_type_right(&self, index: usize) -> i32 {
        self.slots
            .get(index)
            .map(|slot| slot.type_right)
            .unwrap_or(0)
    }

    pub fn set_slot_color_right(&mut self, index: usize, color: Color) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.color_right = color;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_color_right(&self, index: usize) -> Color {
        self.slots
            .get(index)
            .map(|slot| slot.color_right)
            .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn set_slot_custom_icon_right(&mut self, index: usize, icon: Option<Rid>) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.custom_right = icon;
                true
            }
            None => false,
        }
    }

    pub fn get_slot_custom_icon_right(&self, index: usize) -> Option<Rid> {
        self.slots.get(index).and_then(|slot| slot.custom_right)
    }

    pub fn set_slot_metadata_right(&mut self, index: usize, value: Variant) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.metadata = Some(value);
                true
            }
            None => false,
        }
    }

    pub fn get_slot_metadata_right(&self, index: usize) -> Option<Variant> {
        self.slots.get(index).and_then(|slot| slot.metadata.clone())
    }

    pub fn set_slot_draw_stylebox(&mut self, index: usize, draw: bool) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                slot.draw_stylebox = draw;
                true
            }
            None => false,
        }
    }

    pub fn is_slot_drawing_stylebox(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .map(|slot| slot.draw_stylebox)
            .unwrap_or(false)
    }

    pub fn set_selected_slot(&mut self, slot: i32) {
        self.selected_slot = slot;
    }

    pub fn get_selected_slot(&self) -> i32 {
        self.selected_slot
    }

    pub fn set_ignore_invalid_connection_type(&mut self, ignore: bool) {
        self.ignore_invalid_connection_type = ignore;
    }

    pub fn is_ignoring_invalid_connection_type(&self) -> bool {
        self.ignore_invalid_connection_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_slot_growth() {
        let mut node = GraphNodeData::new();
        assert_eq!(node.get_title(), "");
        assert_eq!(node.get_slot_count(), 0);
        assert_eq!(node.get_selected_slot(), -1);
        assert!(!node.is_ignoring_invalid_connection_type());
        assert!(node.get_slot(0).is_none());

        node.set_title("Node");
        node.set_slot(
            1,
            true,
            2,
            Color::new(1.0, 0.0, 0.0, 1.0),
            false,
            3,
            Color::new(0.0, 1.0, 0.0, 1.0),
            Some(Rid::from_u64(1)),
            None,
            false,
        );
        assert_eq!(node.get_title(), "Node");
        assert_eq!(node.get_slot_count(), 2, "slot 1 grows the list to two");
        // Slot 0 is the default entry.
        assert!(!node.is_slot_enabled_left(0));
        assert!(node.is_slot_enabled_left(1));
        assert_eq!(node.get_slot_type_left(1), 2);
        assert_eq!(node.get_slot_color_left(1), Color::new(1.0, 0.0, 0.0, 1.0));
        assert!(!node.is_slot_enabled_right(1));
        assert_eq!(node.get_slot_type_right(1), 3);
        assert_eq!(node.get_slot_custom_icon_left(1), Some(Rid::from_u64(1)));
        assert!(!node.is_slot_drawing_stylebox(1));

        node.set_selected_slot(1);
        assert_eq!(node.get_selected_slot(), 1);
        node.set_ignore_invalid_connection_type(true);
        assert!(node.is_ignoring_invalid_connection_type());
    }

    #[test]
    fn field_setters_and_clears() {
        let mut node = GraphNodeData::new();
        node.set_slot(0, false, 0, Color::new(1.0, 1.0, 1.0, 1.0), false, 0, Color::new(1.0, 1.0, 1.0, 1.0), None, None, true);
        assert!(node.set_slot_enabled_left(0, true));
        assert!(node.set_slot_enabled_right(0, true));
        assert!(node.set_slot_type_left(0, 4));
        assert!(node.set_slot_type_right(0, 5));
        assert!(node.set_slot_color_left(0, Color::new(0.5, 0.5, 0.5, 1.0)));
        assert!(node.set_slot_color_right(0, Color::new(0.25, 0.25, 0.25, 1.0)));
        assert!(node.set_slot_custom_icon_left(0, Some(Rid::from_u64(2))));
        assert!(node.set_slot_custom_icon_right(0, Some(Rid::from_u64(3))));
        assert!(node.set_slot_metadata_left(0, Variant::Int(7)));
        assert!(node.set_slot_metadata_right(0, Variant::String(crate::string::gstring::GoString::from_str("m"))));
        assert!(node.set_slot_draw_stylebox(0, false));

        assert!(node.is_slot_enabled_left(0));
        assert!(node.is_slot_enabled_right(0));
        assert_eq!(node.get_slot_type_left(0), 4);
        assert_eq!(node.get_slot_type_right(0), 5);
        assert_eq!(node.get_slot_color_left(0), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(node.get_slot_color_right(0), Color::new(0.25, 0.25, 0.25, 1.0));
        assert_eq!(node.get_slot_custom_icon_left(0), Some(Rid::from_u64(2)));
        assert_eq!(node.get_slot_custom_icon_right(0), Some(Rid::from_u64(3)));
        // The engine stores one metadata value per slot; the right setter wins.
        assert_eq!(
            node.get_slot_metadata_left(0),
            Some(Variant::String(crate::string::gstring::GoString::from_str("m")))
        );
        assert_eq!(node.get_slot_metadata_right(0), Some(Variant::String(crate::string::gstring::GoString::from_str("m"))));
        assert!(!node.is_slot_drawing_stylebox(0));

        // Out-of-range accessors fail or report defaults.
        assert!(!node.set_slot_enabled_left(9, true));
        assert!(!node.set_slot_type_right(9, 1));
        assert_eq!(node.get_slot_type_right(9), 0);
        assert_eq!(node.get_slot_metadata_left(9), None);
        assert!(!node.is_slot_drawing_stylebox(9));

        assert!(node.clear_slot(0));
        assert!(!node.is_slot_enabled_left(0));
        assert!(!node.clear_slot(9));
        node.clear_all_slots();
        assert_eq!(node.get_slot_count(), 0);
        assert_eq!(node.get_selected_slot(), -1);
    }
}
