//! `scene/gui/split_container.*`: split offsets and the dragger clamp algorithm.
//!
//! State: orientation (`vertical`), `collapsed`, the dragger visibility
//! (`VISIBLE`), the per-dragger split offsets, the drag-area options and the
//! `fixed` flag that subclasses set (`HSplitContainer`/`VSplitContainer` refuse
//! orientation changes once fixed).
//!
//! Ported: every setter/getter plus `_update_dragger_positions`, the scalar core
//! of the layout: desired positions are `default + offset` clamped to each
//! dragger's valid range; when nothing is prioritised, overlaps are pushed to the
//! right, and when a dragger is prioritised its constraints propagate left and
//! then right (including each child's maximum size when one is set). Collapsed
//! containers use the defaults clamped to their ranges. RTL inversion is applied
//! last. Split offsets are rewritten back from the clamped positions when a
//! dragger is prioritised, exactly like the C++.
//!
//! Theme separation, child rect fitting, input handling and the dragger controls
//! are not ported; the algorithm takes the separation and child sizes as
//! arguments.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DraggerVisibility {
    Visible,
    Hidden,
    HiddenCollapsed,
}

#[derive(Clone, Debug)]
pub struct SplitContainerData {
    vertical: bool,
    collapsed: bool,
    dragger_visibility: DraggerVisibility,
    split_offsets: Vec<i32>,
    show_drag_area: bool,
    drag_area_margin_begin: i32,
    drag_area_margin_end: i32,
    drag_area_offset: i32,
    fixed: bool,
}

impl Default for SplitContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl SplitContainerData {
    pub fn new() -> Self {
        Self {
            vertical: false,
            collapsed: false,
            dragger_visibility: DraggerVisibility::Visible,
            split_offsets: Vec::new(),
            show_drag_area: false,
            drag_area_margin_begin: 0,
            drag_area_margin_end: 0,
            drag_area_offset: 0,
            fixed: false,
        }
    }

    pub fn set_vertical(&mut self, vertical: bool) -> bool {
        if self.fixed {
            return false;
        }
        self.vertical = vertical;
        true
    }

    pub fn is_vertical(&self) -> bool {
        self.vertical
    }

    /// Used by `HSplitContainer`/`VSplitContainer` constructors.
    pub fn set_fixed(&mut self, fixed: bool) {
        self.fixed = fixed;
    }

    pub fn is_fixed(&self) -> bool {
        self.fixed
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn set_dragger_visibility(&mut self, visibility: DraggerVisibility) {
        self.dragger_visibility = visibility;
    }

    pub fn get_dragger_visibility(&self) -> DraggerVisibility {
        self.dragger_visibility
    }

    /// `set_split_offset(p_offset, p_index)`; the vector grows as needed.
    pub fn set_split_offset(&mut self, offset: i32, index: usize) {
        if self.split_offsets.len() <= index {
            self.split_offsets.resize(index + 1, 0);
        }
        self.split_offsets[index] = offset;
    }

    pub fn get_split_offset(&self, index: usize) -> i32 {
        self.split_offsets.get(index).copied().unwrap_or(0)
    }

    pub fn get_split_offsets(&self) -> &[i32] {
        &self.split_offsets
    }

    pub fn set_show_drag_area(&mut self, show: bool) {
        self.show_drag_area = show;
    }

    pub fn is_showing_drag_area(&self) -> bool {
        self.show_drag_area
    }

    pub fn set_drag_area_margin_begin(&mut self, margin: i32) {
        self.drag_area_margin_begin = margin;
    }

    pub fn get_drag_area_margin_begin(&self) -> i32 {
        self.drag_area_margin_begin
    }

    pub fn set_drag_area_margin_end(&mut self, margin: i32) {
        self.drag_area_margin_end = margin;
    }

    pub fn get_drag_area_margin_end(&self) -> i32 {
        self.drag_area_margin_end
    }

    pub fn set_drag_area_offset(&mut self, offset: i32) {
        self.drag_area_offset = offset;
    }

    pub fn get_drag_area_offset(&self) -> i32 {
        self.drag_area_offset
    }

    /// `_update_dragger_positions`, reduced to the scalar math.
    ///
    /// - `default_positions[i]` is dragger `i`'s position when every offset is 0;
    /// - `valid_ranges[i]` is its `(min, max)` clamp;
    /// - `min_sizes[i]` / `max_sizes[i]` describe child `i` (`max_sizes < 0` means
    ///   unlimited).
    ///
    /// Returns the final dragger positions (after RTL inversion) and, when
    /// `priority` is set, rewrites the matching split offsets.
    #[allow(clippy::too_many_arguments)]
    pub fn update_dragger_positions(
        &mut self,
        default_positions: &[i32],
        valid_ranges: &[(i32, i32)],
        min_sizes: &[i32],
        max_sizes: &[i32],
        separation: i32,
        size: i32,
        rtl: bool,
        priority: Option<usize>,
    ) -> Vec<i32> {
        let count = default_positions.len();
        let mut positions = vec![0i32; count];

        if self.split_offsets.len() < count {
            self.split_offsets.resize(count, 0);
        }

        if self.collapsed {
            for i in 0..count {
                let (lo, hi) = valid_ranges.get(i).copied().unwrap_or((i32::MIN, i32::MAX));
                positions[i] = default_positions[i].clamp(lo, hi);
                if priority.is_some() {
                    self.split_offsets[i] = positions[i] - default_positions[i];
                }
            }
            if self.vertical_is_rtl(rtl) {
                for position in positions.iter_mut() {
                    *position = size - *position - separation;
                }
            }
            return positions;
        }

        for i in 0..count {
            let (lo, hi) = valid_ranges.get(i).copied().unwrap_or((i32::MIN, i32::MAX));
            positions[i] = (default_positions[i] + self.split_offsets[i]).clamp(lo, hi);
        }

        match priority {
            None => {
                for i in 0..count.saturating_sub(1) {
                    let check_min_size = min_sizes.get(i + 1).copied().unwrap_or(0);
                    let push_pos = positions[i] + separation + check_min_size;
                    if positions[i + 1] < push_pos {
                        positions[i + 1] = push_pos;
                        let (lo, hi) =
                            valid_ranges.get(i).copied().unwrap_or((i32::MIN, i32::MAX));
                        positions[i] = positions[i].clamp(lo, hi);
                    }
                }
            }
            Some(index) => {
                let dragging_position = positions[index];

                for i in (0..index).rev() {
                    let right = if i == index - 1 {
                        dragging_position
                    } else {
                        positions[i + 1]
                    };
                    let min_size = min_sizes.get(i + 1).copied().unwrap_or(0);
                    let max_position = right - separation - min_size;
                    if positions[i] > max_position {
                        positions[i] = max_position;
                    }
                    let max_size = max_sizes.get(i + 1).copied().unwrap_or(-1);
                    if max_size >= 0 {
                        let min_position = right - separation - max_size;
                        if positions[i] < min_position {
                            positions[i] = min_position;
                        }
                    }
                }

                for i in index + 1..count {
                    let left = if i == index + 1 {
                        dragging_position
                    } else {
                        positions[i - 1]
                    };
                    let min_size = min_sizes.get(i).copied().unwrap_or(0);
                    let min_position = left + separation + min_size;
                    if positions[i] < min_position {
                        positions[i] = min_position;
                    }
                    let max_size = max_sizes.get(i).copied().unwrap_or(-1);
                    if max_size >= 0 {
                        let max_position = left + separation + max_size;
                        if positions[i] > max_position {
                            positions[i] = max_position;
                        }
                    }
                }

                for i in 0..count {
                    self.split_offsets[i] = positions[i] - default_positions[i];
                }
            }
        }

        if self.vertical_is_rtl(rtl) {
            for position in positions.iter_mut() {
                *position = size - *position - separation;
            }
        }

        positions
    }

    /// RTL only mirrors horizontal containers.
    fn vertical_is_rtl(&self, rtl: bool) -> bool {
        !self.vertical && rtl
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_settings() {
        let mut s = SplitContainerData::new();
        assert!(!s.is_vertical());
        assert!(!s.is_collapsed());
        assert_eq!(s.get_dragger_visibility(), DraggerVisibility::Visible);
        assert!(s.get_split_offsets().is_empty());
        assert_eq!(s.get_split_offset(0), 0);
        assert!(!s.is_showing_drag_area());
        assert_eq!(s.get_drag_area_margin_begin(), 0);
        assert_eq!(s.get_drag_area_offset(), 0);
        assert!(!s.is_fixed());

        s.set_split_offset(5, 1);
        assert_eq!(s.get_split_offset(0), 0);
        assert_eq!(s.get_split_offset(1), 5);
        assert_eq!(s.get_split_offsets(), &[0, 5]);

        s.set_collapsed(true);
        s.set_dragger_visibility(DraggerVisibility::HiddenCollapsed);
        s.set_show_drag_area(true);
        s.set_drag_area_margin_begin(2);
        s.set_drag_area_margin_end(3);
        s.set_drag_area_offset(4);
        assert!(s.is_collapsed());
        assert_eq!(
            s.get_dragger_visibility(),
            DraggerVisibility::HiddenCollapsed
        );
        assert!(s.is_showing_drag_area());
        assert_eq!(s.get_drag_area_margin_end(), 3);
        assert_eq!(s.get_drag_area_offset(), 4);

        assert_eq!(DraggerVisibility::Visible as i32, 0);
        assert_eq!(DraggerVisibility::Hidden as i32, 1);
        assert_eq!(DraggerVisibility::HiddenCollapsed as i32, 2);
    }

    #[test]
    fn orientation_is_rejected_when_fixed() {
        let mut s = SplitContainerData::new();
        assert!(s.set_vertical(true));
        assert!(s.is_vertical());
        s.set_fixed(true);
        assert!(!s.set_vertical(false), "fixed containers keep their axis");
        assert!(s.is_vertical());
    }

    #[test]
    fn no_priority_pushes_overlaps_right() {
        let mut s = SplitContainerData::new();
        // Two draggers at 100 and 110, but the middle child needs 40 px.
        let positions = s.update_dragger_positions(
            &[100, 110],
            &[(0, 1000), (0, 1000)],
            &[0, 40, 0],
            &[-1, -1, -1],
            4,
            1000,
            false,
            None,
        );
        // Second dragger moves to 100 + 4 + 40.
        assert_eq!(positions, vec![100, 144]);

        // A split offset shifts the desired position first.
        s.set_split_offset(30, 0);
        let positions = s.update_dragger_positions(
            &[100, 300],
            &[(0, 1000), (0, 1000)],
            &[0, 40, 0],
            &[-1, -1, -1],
            4,
            1000,
            false,
            None,
        );
        assert_eq!(positions, vec![130, 300]);
    }

    #[test]
    fn values_clamp_to_valid_ranges() {
        let mut s = SplitContainerData::new();
        s.set_split_offset(-500, 0);
        let positions = s.update_dragger_positions(
            &[100],
            &[(50, 200)],
            &[0, 0],
            &[-1, -1],
            0,
            1000,
            false,
            None,
        );
        assert_eq!(positions, vec![50]);

        s.set_split_offset(900, 0);
        let positions = s.update_dragger_positions(
            &[100],
            &[(50, 200)],
            &[0, 0],
            &[-1, -1],
            0,
            1000,
            false,
            None,
        );
        assert_eq!(positions, vec![200]);
    }

    #[test]
    fn priority_propagates_and_writes_offsets() {
        let mut s = SplitContainerData::new();
        // Middle child's minimum is 50 and the separation 5, so the left dragger
        // cannot pass 100 - 5 - 50 = 45.
        let positions = s.update_dragger_positions(
            &[100, 100],
            &[(0, 1000), (0, 1000)],
            &[0, 50, 0],
            &[-1, -1, -1],
            5,
            1000,
            false,
            Some(1),
        );
        assert_eq!(positions, vec![45, 100]);
        // The clamped offset is written back for dragger 0.
        assert_eq!(s.get_split_offset(0), -55);
        assert_eq!(s.get_split_offset(1), 0);

        // A maximum size pulls the dragger back.
        let mut s = SplitContainerData::new();
        let positions = s.update_dragger_positions(
            &[100, 400],
            &[(0, 1000), (0, 1000)],
            &[0, 0, 0],
            &[-1, 100, -1],
            0,
            1000,
            false,
            Some(1),
        );
        // Child 1 cannot exceed 100 px: left dragger >= 400 - 100.
        assert_eq!(positions, vec![300, 400]);
    }

    #[test]
    fn collapsed_uses_defaults_and_rtl_mirrors() {
        let mut s = SplitContainerData::new();
        s.set_collapsed(true);
        let positions = s.update_dragger_positions(
            &[100, 300],
            &[(0, 250), (0, 1000)],
            &[0, 0, 0],
            &[-1, -1, -1],
            10,
            1000,
            false,
            None,
        );
        assert_eq!(positions, vec![100, 300]);

        // Horizontal RTL mirrors: size - position - separation.
        let positions = s.update_dragger_positions(
            &[100, 300],
            &[(0, 1000), (0, 1000)],
            &[0, 0, 0],
            &[-1, -1, -1],
            10,
            1000,
            true,
            None,
        );
        assert_eq!(positions, vec![890, 690]);

        // Vertical containers ignore RTL.
        let mut v = SplitContainerData::new();
        v.set_vertical(true);
        let positions = v.update_dragger_positions(
            &[100],
            &[(0, 1000)],
            &[0, 0],
            &[-1, -1],
            10,
            1000,
            true,
            None,
        );
        assert_eq!(positions, vec![100]);
    }
}
