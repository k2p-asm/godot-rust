//! `scene/gui/spin_box.*`: the spin box's text form and options (no arrows).
//!
//! `SpinBox` owns a `LineEdit`; the constructor leaves the field editable,
//! selection-on-focus off, alignment left, empty prefix/suffix, `update_on_text_changed`
//! off and a zero custom arrow step with rounding off.
//!
//! Ported: every option setter/getter, including the ones that delegate to the
//! line edit (`editable`, `select_all_on_focus`, `horizontal_alignment`), and
//! `compose_text`, which reproduces `_update_text`'s `prefix + value + suffix`
//! shape (the engine additionally applies locale digit grouping — not ported).
//!
//! Arrow clicks, the drag timer, `apply()`'s string→number parsing and the theme
//! are not ported; `Range` values stay with the caller.

use crate::scene::base_button::HorizontalAlignment;
use crate::scene::line_edit::LineEditData;

#[derive(Clone, Debug)]
pub struct SpinBoxData {
    update_on_text_changed: bool,
    custom_arrow_step: f64,
    custom_arrow_round: bool,
    prefix: String,
    suffix: String,
    accepted: bool,
    pub line_edit: LineEditData,
}

impl Default for SpinBoxData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinBoxData {
    pub fn new() -> Self {
        let mut line_edit = LineEditData::new();
        line_edit.set_editable(true);
        Self {
            update_on_text_changed: false,
            custom_arrow_step: 0.0,
            custom_arrow_round: false,
            prefix: String::new(),
            suffix: String::new(),
            accepted: true,
            line_edit,
        }
    }

    pub fn set_prefix(&mut self, prefix: &str) {
        self.prefix = prefix.to_string();
    }

    pub fn get_prefix(&self) -> &str {
        &self.prefix
    }

    pub fn set_suffix(&mut self, suffix: &str) {
        self.suffix = suffix.to_string();
    }

    pub fn get_suffix(&self) -> &str {
        &self.suffix
    }

    pub fn set_update_on_text_changed(&mut self, enabled: bool) {
        self.update_on_text_changed = enabled;
    }

    pub fn get_update_on_text_changed(&self) -> bool {
        self.update_on_text_changed
    }

    pub fn set_select_all_on_focus(&mut self, enabled: bool) {
        self.line_edit.set_select_all_on_focus(enabled);
    }

    pub fn is_select_all_on_focus(&self) -> bool {
        self.line_edit.is_select_all_on_focus()
    }

    pub fn set_editable(&mut self, enabled: bool) {
        self.line_edit.set_editable(enabled);
    }

    pub fn is_editable(&self) -> bool {
        self.line_edit.is_editable()
    }

    pub fn set_horizontal_alignment(&mut self, alignment: HorizontalAlignment) {
        self.line_edit.set_alignment(alignment);
    }

    pub fn get_horizontal_alignment(&self) -> HorizontalAlignment {
        self.line_edit.get_alignment()
    }

    pub fn set_custom_arrow_step(&mut self, step: f64) {
        self.custom_arrow_step = step;
    }

    pub fn get_custom_arrow_step(&self) -> f64 {
        self.custom_arrow_step
    }

    pub fn set_custom_arrow_round(&mut self, round: bool) {
        self.custom_arrow_round = round;
    }

    pub fn get_custom_arrow_round(&self) -> bool {
        self.custom_arrow_round
    }

    /// `_update_text`: prefix, the formatted value, then suffix.
    pub fn compose_text(&self, value_text: &str) -> String {
        format!("{}{}{}", self.prefix, value_text, self.suffix)
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted
    }

    pub fn set_accepted(&mut self, accepted: bool) {
        self.accepted = accepted;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = SpinBoxData::new();
        assert_eq!(s.get_prefix(), "");
        assert_eq!(s.get_suffix(), "");
        assert!(!s.get_update_on_text_changed());
        assert!(!s.is_select_all_on_focus());
        assert!(s.is_editable(), "the constructor makes the field editable");
        assert_eq!(s.get_horizontal_alignment(), HorizontalAlignment::Left);
        assert_eq!(s.get_custom_arrow_step(), 0.0);
        assert!(!s.get_custom_arrow_round());
        assert!(s.is_accepted());
    }

    #[test]
    fn options_round_trip() {
        let mut s = SpinBoxData::new();
        s.set_prefix("$");
        s.set_suffix(" m");
        s.set_update_on_text_changed(true);
        s.set_select_all_on_focus(true);
        s.set_editable(false);
        s.set_horizontal_alignment(HorizontalAlignment::Right);
        s.set_custom_arrow_step(0.5);
        s.set_custom_arrow_round(true);

        assert_eq!(s.get_prefix(), "$");
        assert_eq!(s.get_suffix(), " m");
        assert!(s.get_update_on_text_changed());
        assert!(s.is_select_all_on_focus());
        assert!(!s.is_editable());
        assert_eq!(s.get_horizontal_alignment(), HorizontalAlignment::Right);
        assert_eq!(s.get_custom_arrow_step(), 0.5);
        assert!(s.get_custom_arrow_round());
        assert_eq!(s.compose_text("12"), "$12 m");
    }
}
