//! `scene/gui/margin_container.*` and `scene/gui/center_container.*`.
//!
//! `MarginContainer` keeps four theme margins (0 by default) and
//! `CenterContainer` a `use_top_left` flag (off). The engine exposes the margins
//! as theme constants; this port adds `set_margin`/`get_margin` with the engine's
//! `Side` order (left, top, right, bottom) so callers can drive them.
//!
//! Layout and sorting are not ported.

/// `Side`: left, top, right, bottom.
pub const SIDE_LEFT: i32 = 0;
pub const SIDE_TOP: i32 = 1;
pub const SIDE_RIGHT: i32 = 2;
pub const SIDE_BOTTOM: i32 = 3;

#[derive(Clone, Debug)]
pub struct MarginContainerData {
    /// Indexed by `Side`.
    margins: [i32; 4],
}

impl Default for MarginContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl MarginContainerData {
    pub fn new() -> Self {
        Self { margins: [0; 4] }
    }

    /// Out-of-range sides are refused (the engine would look up an invalid theme
    /// constant, which resolves to 0).
    pub fn set_margin(&mut self, side: i32, value: i32) -> bool {
        if !(0..4).contains(&side) {
            return false;
        }
        self.margins[side as usize] = value;
        true
    }

    pub fn get_margin(&self, side: i32) -> i32 {
        if !(0..4).contains(&side) {
            return 0;
        }
        self.margins[side as usize]
    }

    pub fn set_margins(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
        self.margins = [left, top, right, bottom];
    }

    pub fn get_margins(&self) -> [i32; 4] {
        self.margins
    }

    /// Horizontal + vertical insets, as used by the minimum-size calculation.
    pub fn get_combined_minimum_size(&self) -> (i32, i32) {
        (
            self.margins[SIDE_LEFT as usize] + self.margins[SIDE_RIGHT as usize],
            self.margins[SIDE_TOP as usize] + self.margins[SIDE_BOTTOM as usize],
        )
    }
}

#[derive(Clone, Debug)]
pub struct CenterContainerData {
    use_top_left: bool,
}

impl Default for CenterContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl CenterContainerData {
    pub fn new() -> Self {
        Self { use_top_left: false }
    }

    pub fn set_use_top_left(&mut self, enabled: bool) {
        self.use_top_left = enabled;
    }

    pub fn is_using_top_left(&self) -> bool {
        self.use_top_left
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_container_state() {
        let mut margin = MarginContainerData::new();
        assert_eq!(margin.get_margins(), [0, 0, 0, 0]);
        assert_eq!(margin.get_combined_minimum_size(), (0, 0));

        assert!(margin.set_margin(SIDE_LEFT, 8));
        assert!(margin.set_margin(SIDE_BOTTOM, 4));
        assert!(!margin.set_margin(4, 1));
        assert_eq!(margin.get_margin(SIDE_LEFT), 8);
        assert_eq!(margin.get_margin(SIDE_TOP), 0);
        assert_eq!(margin.get_margin(SIDE_RIGHT), 0);
        assert_eq!(margin.get_margin(SIDE_BOTTOM), 4);
        assert_eq!(margin.get_margin(-1), 0);
        assert_eq!(margin.get_combined_minimum_size(), (8, 4));

        margin.set_margins(1, 2, 3, 5);
        assert_eq!(margin.get_margins(), [1, 2, 3, 5]);
        assert_eq!(margin.get_combined_minimum_size(), (4, 7));
        assert_eq!(SIDE_LEFT, 0);
        assert_eq!(SIDE_BOTTOM, 3);
    }

    #[test]
    fn center_container_state() {
        let mut center = CenterContainerData::new();
        assert!(!center.is_using_top_left());
        center.set_use_top_left(true);
        assert!(center.is_using_top_left());
    }
}
