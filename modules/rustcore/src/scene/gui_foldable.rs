//! `scene/gui/foldable_container.*`: foldable containers and their group.
//!
//! Ported state: `FoldableContainer`'s folded flag, title, language, title text
//! direction/alignment, overrun behaviour and title position (top by default);
//! `FoldableGroup`'s membership rules — unfolding one container folds the others
//! unless `allow_folding_all` is set — and `get_expanded_container`.
//!
//! Title drawing, text shaping, theme handling and the notification path are not
//! ported.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TitlePosition {
    Top = 0,
    Bottom = 1,
}

#[derive(Clone, Debug)]
pub struct FoldableContainerData {
    id: u64,
    folded: bool,
    title: String,
    language: String,
    title_text_direction: i32,
    title_alignment: i32,
    overrun_behavior: i32,
    title_position: TitlePosition,
}

impl FoldableContainerData {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            folded: false,
            title: String::new(),
            language: String::new(),
            title_text_direction: 0,
            title_alignment: 0,
            overrun_behavior: 0,
            title_position: TitlePosition::Top,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_folded(&mut self, folded: bool) {
        self.folded = folded;
    }

    pub fn is_folded(&self) -> bool {
        self.folded
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_title_text_direction(&mut self, direction: i32) {
        self.title_text_direction = direction;
    }

    pub fn get_title_text_direction(&self) -> i32 {
        self.title_text_direction
    }

    pub fn set_title_alignment(&mut self, alignment: i32) {
        self.title_alignment = alignment;
    }

    pub fn get_title_alignment(&self) -> i32 {
        self.title_alignment
    }

    pub fn set_title_position(&mut self, position: i32) -> bool {
        let parsed = match position {
            0 => TitlePosition::Top,
            1 => TitlePosition::Bottom,
            _ => return false,
        };
        self.title_position = parsed;
        true
    }

    pub fn get_title_position(&self) -> TitlePosition {
        self.title_position
    }

    pub fn set_text_overrun_behavior(&mut self, behavior: i32) {
        self.overrun_behavior = behavior;
    }

    pub fn get_text_overrun_behavior(&self) -> i32 {
        self.overrun_behavior
    }
}

/// `FoldableGroup`: keeps its members and enforces at most one expanded
/// container unless `allow_folding_all` is enabled.
#[derive(Clone, Debug, Default)]
pub struct FoldableGroupData {
    containers: BTreeSet<u64>,
    allow_folding_all: bool,
    /// The member that was last unfolded (its container object holds the flag in
    /// this port; the group tracks the id for `get_expanded_container`).
    expanded: Option<u64>,
}

impl FoldableGroupData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_container(&mut self, id: u64) {
        self.containers.insert(id);
    }

    pub fn remove_container(&mut self, id: u64) {
        self.containers.remove(&id);
        if self.expanded == Some(id) {
            self.expanded = None;
        }
    }

    pub fn get_containers(&self) -> Vec<u64> {
        self.containers.iter().copied().collect()
    }

    pub fn get_container_count(&self) -> usize {
        self.containers.len()
    }

    pub fn set_allow_folding_all(&mut self, enabled: bool) {
        self.allow_folding_all = enabled;
    }

    pub fn is_allow_folding_all(&self) -> bool {
        self.allow_folding_all
    }

    /// `get_expanded_container`: the id of the single expanded member.
    pub fn get_expanded_container(&self) -> Option<u64> {
        self.expanded
    }

    /// Records a fold change for `id`; when it unfolds and folding-all is not
    /// allowed, the previously expanded member is reported back so the caller can
    /// fold it.
    pub fn set_folded(&mut self, id: u64, folded: bool) -> Option<u64> {
        if !self.containers.contains(&id) {
            return None;
        }
        if folded {
            if self.expanded == Some(id) {
                self.expanded = None;
            }
            return None;
        }
        let previous = if !self.allow_folding_all {
            self.expanded.filter(|expanded| *expanded != id)
        } else {
            None
        };
        self.expanded = Some(id);
        previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_defaults_and_setters() {
        let mut container = FoldableContainerData::new(1);
        assert!(!container.is_folded());
        assert_eq!(container.get_title(), "");
        assert_eq!(container.get_language(), "");
        assert_eq!(container.get_title_text_direction(), 0);
        assert_eq!(container.get_title_alignment(), 0);
        assert_eq!(container.get_title_position(), TitlePosition::Top);
        assert_eq!(container.get_text_overrun_behavior(), 0);
        assert_eq!(container.get_id(), 1);

        container.set_folded(true);
        container.set_title("Options");
        container.set_language("en");
        container.set_title_text_direction(2);
        container.set_title_alignment(1);
        container.set_text_overrun_behavior(3);
        assert!(container.set_title_position(1));
        assert!(!container.set_title_position(2));
        assert!(container.is_folded());
        assert_eq!(container.get_title(), "Options");
        assert_eq!(container.get_language(), "en");
        assert_eq!(container.get_title_text_direction(), 2);
        assert_eq!(container.get_title_alignment(), 1);
        assert_eq!(container.get_text_overrun_behavior(), 3);
        assert_eq!(container.get_title_position(), TitlePosition::Bottom);
        assert_eq!(TitlePosition::Top as i32, 0);
        assert_eq!(TitlePosition::Bottom as i32, 1);
    }

    #[test]
    fn group_membership() {
        let mut group = FoldableGroupData::new();
        assert!(group.get_containers().is_empty());
        group.add_container(1);
        group.add_container(2);
        group.add_container(1);
        assert_eq!(group.get_containers(), vec![1, 2]);
        assert_eq!(group.get_container_count(), 2);
        group.remove_container(1);
        assert_eq!(group.get_containers(), vec![2]);
    }

    #[test]
    fn group_folds_the_previous_container() {
        let mut group = FoldableGroupData::new();
        group.add_container(1);
        group.add_container(2);
        assert_eq!(group.get_expanded_container(), None);

        // Unfolding the first member records it.
        assert_eq!(group.set_folded(1, false), None);
        assert_eq!(group.get_expanded_container(), Some(1));

        // Unfolding the second reports the first so the caller can fold it.
        assert_eq!(group.set_folded(2, false), Some(1));
        assert_eq!(group.get_expanded_container(), Some(2));

        // Folding the expanded member clears the record.
        group.set_folded(2, true);
        assert_eq!(group.get_expanded_container(), None);

        // Unknown members are ignored.
        assert_eq!(group.set_folded(9, false), None);
        assert_eq!(group.get_expanded_container(), None);
    }

    #[test]
    fn allow_folding_all_keeps_several_open() {
        let mut group = FoldableGroupData::new();
        group.add_container(1);
        group.add_container(2);
        group.set_allow_folding_all(true);
        assert!(group.is_allow_folding_all());

        assert_eq!(group.set_folded(1, false), None);
        // No previous container is reported.
        assert_eq!(group.set_folded(2, false), None);
        assert_eq!(group.get_expanded_container(), Some(2));

        group.set_allow_folding_all(false);
        assert_eq!(group.set_folded(1, false), Some(2));
        assert_eq!(group.get_expanded_container(), Some(1));
    }
}
