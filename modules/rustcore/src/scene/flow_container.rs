//! `scene/gui/flow_container.*`: line wrapping and the container's options.
//!
//! Ported: the container's settings (`vertical`, `reverse_fill`, the three
//! alignment modes, the four last-wrap modes and the `fixed` flag) and the first
//! pass of `_resort`, which walks the children accumulating offsets until the
//! line would overflow, then records the line's child count, tallest/widest child,
//! consumed length, remaining stretch space, stretch-ratio total and whether the
//! line is "filled".
//!
//! The second pass (in-line stretch expansion, alignment offsets, child rects)
//! and the third pass (final placement) are not ported; `FlowLine` is exactly what
//! that second pass consumes.
//!
//! Theme separations are arguments here (the port has no theme cache).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlignmentMode {
    Begin,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LastWrapAlignmentMode {
    Inherit,
    Begin,
    Center,
    End,
}

#[derive(Clone, Copy, Debug)]
pub struct FlowChild {
    pub min_size: crate::math::vector2i::Vector2i,
    pub max_size: crate::math::vector2i::Vector2i,
    pub stretch_ratio: f32,
    pub expand_h: bool,
    pub expand_v: bool,
}

/// One line (or column, when vertical) as the first pass computes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowLine {
    pub child_count: i32,
    pub min_line_height: i32,
    pub min_line_length: i32,
    pub stretch_avail: i32,
    pub stretch_ratio_total: f32,
    pub is_filled: bool,
}

#[derive(Clone, Debug)]
pub struct FlowContainerData {
    vertical: bool,
    reverse_fill: bool,
    alignment: AlignmentMode,
    last_wrap_alignment: LastWrapAlignmentMode,
    fixed: bool,
    cached_size: i32,
    cached_line_count: i32,
    cached_line_max_child_count: i32,
}

impl Default for FlowContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowContainerData {
    pub fn new() -> Self {
        Self {
            vertical: false,
            reverse_fill: false,
            alignment: AlignmentMode::Begin,
            last_wrap_alignment: LastWrapAlignmentMode::Inherit,
            fixed: false,
            cached_size: 0,
            cached_line_count: 0,
            cached_line_max_child_count: 0,
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

    pub fn set_fixed(&mut self, fixed: bool) {
        self.fixed = fixed;
    }

    pub fn is_fixed(&self) -> bool {
        self.fixed
    }

    pub fn set_reverse_fill(&mut self, reverse: bool) {
        self.reverse_fill = reverse;
    }

    pub fn is_reverse_fill(&self) -> bool {
        self.reverse_fill
    }

    pub fn set_alignment(&mut self, alignment: AlignmentMode) {
        self.alignment = alignment;
    }

    pub fn get_alignment(&self) -> AlignmentMode {
        self.alignment
    }

    pub fn set_last_wrap_alignment(&mut self, alignment: LastWrapAlignmentMode) {
        self.last_wrap_alignment = alignment;
    }

    pub fn get_last_wrap_alignment(&self) -> LastWrapAlignmentMode {
        self.last_wrap_alignment
    }

    pub fn get_cached_size(&self) -> i32 {
        self.cached_size
    }

    pub fn get_cached_line_count(&self) -> i32 {
        self.cached_line_count
    }

    pub fn get_cached_line_max_child_count(&self) -> i32 {
        self.cached_line_max_child_count
    }

    /// First pass of `_resort`: splits `children` into lines inside
    /// `container_size` (the axis selected by `vertical`) and caches the counts.
    pub fn compute_lines(
        &mut self,
        children: &[FlowChild],
        container_size: i32,
        h_separation: i32,
        v_separation: i32,
    ) -> Vec<FlowLine> {
        let mut lines: Vec<FlowLine> = Vec::new();
        let mut ofs_x = 0i32;
        let mut ofs_y = 0i32;
        let mut line_height = 0i32;
        let mut line_length;
        let mut line_stretch_ratio_total = 0.0f32;
        let mut children_in_current_line = 0i32;
        let mut last_child: Option<FlowChild> = None;

        for child in children {
            let child_min = child.min_size;
            if self.vertical {
                if children_in_current_line > 0 {
                    ofs_y += v_separation;
                }
                if ofs_y + child_min.y > container_size {
                    line_length = ofs_y - v_separation;
                    lines.push(FlowLine {
                        child_count: children_in_current_line,
                        min_line_height: line_height,
                        min_line_length: line_length,
                        stretch_avail: container_size - line_length,
                        stretch_ratio_total: line_stretch_ratio_total,
                        is_filled: true,
                    });
                    // New column.
                    ofs_x += line_height + h_separation;
                    ofs_y = 0;
                    line_height = 0;
                    line_stretch_ratio_total = 0.0;
                    children_in_current_line = 0;
                }
                line_height = line_height.max(child_min.x);
                if child.expand_v {
                    line_stretch_ratio_total += child.stretch_ratio;
                }
                ofs_y += child_min.y;
            } else {
                if children_in_current_line > 0 {
                    ofs_x += h_separation;
                }
                if ofs_x + child_min.x > container_size {
                    line_length = ofs_x - h_separation;
                    lines.push(FlowLine {
                        child_count: children_in_current_line,
                        min_line_height: line_height,
                        min_line_length: line_length,
                        stretch_avail: container_size - line_length,
                        stretch_ratio_total: line_stretch_ratio_total,
                        is_filled: true,
                    });
                    // New line.
                    ofs_y += line_height + v_separation;
                    ofs_x = 0;
                    line_height = 0;
                    line_stretch_ratio_total = 0.0;
                    children_in_current_line = 0;
                }
                line_height = line_height.max(child_min.y);
                if child.expand_h {
                    line_stretch_ratio_total += child.stretch_ratio;
                }
                ofs_x += child_min.x;
            }

            last_child = Some(*child);
            children_in_current_line += 1;
        }

        line_length = if self.vertical { ofs_y } else { ofs_x };
        let is_filled = match last_child {
            None => false,
            Some(child) => {
                if self.vertical {
                    ofs_y + child.min_size.y > container_size
                } else {
                    ofs_x + child.min_size.x > container_size
                }
            }
        };
        lines.push(FlowLine {
            child_count: children_in_current_line,
            min_line_height: line_height,
            min_line_length: line_length,
            stretch_avail: container_size - line_length,
            stretch_ratio_total: line_stretch_ratio_total,
            is_filled,
        });

        self.cached_size = container_size;
        self.cached_line_count = lines.len() as i32;
        self.cached_line_max_child_count = lines.iter().map(|l| l.child_count).max().unwrap_or(0);

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2i::Vector2i;

    fn child(width: i32, height: i32) -> FlowChild {
        FlowChild {
            min_size: Vector2i::new(width, height),
            max_size: Vector2i::new(-1, -1),
            stretch_ratio: 1.0,
            expand_h: false,
            expand_v: false,
        }
    }

    #[test]
    fn defaults() {
        let mut f = FlowContainerData::new();
        assert!(!f.is_vertical());
        assert!(!f.is_reverse_fill());
        assert_eq!(f.get_alignment(), AlignmentMode::Begin);
        assert_eq!(f.get_last_wrap_alignment(), LastWrapAlignmentMode::Inherit);
        assert!(!f.is_fixed());
        assert_eq!(f.get_cached_size(), 0);
        assert_eq!(f.get_cached_line_count(), 0);
        assert_eq!(f.get_cached_line_max_child_count(), 0);

        assert!(f.set_vertical(true));
        f.set_fixed(true);
        assert!(!f.set_vertical(false));
        assert!(f.is_vertical());

        assert_eq!(AlignmentMode::Begin as i32, 0);
        assert_eq!(AlignmentMode::Center as i32, 1);
        assert_eq!(AlignmentMode::End as i32, 2);
        assert_eq!(LastWrapAlignmentMode::Inherit as i32, 0);
        assert_eq!(LastWrapAlignmentMode::Begin as i32, 1);
        assert_eq!(LastWrapAlignmentMode::Center as i32, 2);
        assert_eq!(LastWrapAlignmentMode::End as i32, 3);
    }

    #[test]
    fn horizontal_wrapping() {
        let mut f = FlowContainerData::new();
        // Three 40x10 children in 100 px with 5 px separation:
        // first two fit (40 + 5 + 40 = 85), the third wraps.
        let children = [child(40, 10), child(40, 10), child(40, 10)];
        let lines = f.compute_lines(&children, 100, 5, 5);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].child_count, 2);
        assert_eq!(lines[0].min_line_height, 10);
        assert_eq!(lines[0].min_line_length, 85);
        assert_eq!(lines[0].stretch_avail, 15);
        assert!(lines[0].is_filled);
        assert_eq!(lines[1].child_count, 1);
        assert_eq!(lines[1].min_line_length, 40);
        assert!(!lines[1].is_filled, "the last line is not filled");

        assert_eq!(f.get_cached_size(), 100);
        assert_eq!(f.get_cached_line_count(), 2);
        assert_eq!(f.get_cached_line_max_child_count(), 2);
    }

    #[test]
    fn stretch_ratios_accumulate_per_line() {
        let mut f = FlowContainerData::new();
        let mut a = child(40, 10);
        a.expand_h = true;
        a.stretch_ratio = 2.0;
        let mut b = child(40, 10);
        b.expand_h = true;
        b.stretch_ratio = 3.0;
        let mut c = child(40, 10);
        c.expand_h = false;
        c.stretch_ratio = 10.0;

        let lines = f.compute_lines(&[a, b, c], 500, 0, 0);
        assert_eq!(lines.len(), 1);
        assert!((lines[0].stretch_ratio_total - 5.0).abs() < 1e-6);

        // Vertical containers accumulate the vertical expand flag instead.
        let mut v = FlowContainerData::new();
        v.set_vertical(true);
        let mut d = child(10, 40);
        d.expand_v = true;
        d.stretch_ratio = 1.5;
        let lines = v.compute_lines(&[d], 500, 0, 0);
        assert!((lines[0].stretch_ratio_total - 1.5).abs() < 1e-6);
    }

    #[test]
    fn vertical_wrapping_uses_columns() {
        let mut f = FlowContainerData::new();
        f.set_vertical(true);
        // Children are 10x40; in 100 px, two fit (40 + 5 + 40 = 85), the third
        // starts a new column whose width is the widest child (10).
        let children = [child(10, 40), child(10, 40), child(10, 40)];
        let lines = f.compute_lines(&children, 100, 5, 5);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].child_count, 2);
        assert_eq!(lines[0].min_line_height, 10, "column width");
        assert_eq!(lines[0].min_line_length, 85);
        assert_eq!(lines[1].child_count, 1);
    }

    #[test]
    fn empty_and_overflowing_children() {
        let mut f = FlowContainerData::new();
        // No children still leaves one (empty) line, like the engine's final push.
        let lines = f.compute_lines(&[], 100, 0, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].child_count, 0);
        assert_eq!(lines[0].min_line_length, 0);
        assert_eq!(lines[0].stretch_avail, 100);
        assert!(!lines[0].is_filled);

        // A child wider than the container starts alone and is "filled".
        let lines = f.compute_lines(&[child(200, 10)], 100, 0, 0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].child_count, 0);
        assert_eq!(lines[1].child_count, 1);
        assert!(lines[1].is_filled);
    }
}
