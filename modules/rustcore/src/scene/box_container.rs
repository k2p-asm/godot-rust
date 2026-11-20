//! `scene/gui/box_container.*`: automatic single-axis layout.
//!
//! Children get their minimum size along the main axis, the leftover space is
//! shared among `expand` children by `stretch_ratio`, and the cross axis fills
//! the container. `separation` (default 4) sits between children. Alignment
//! shifts the whole row/column when the leftovers are not taken.
//!
//! Not ported: nested containers, `SIZE_SHRINK_*` flags, RTL, and the
//! `Container` notification plumbing.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Begin,
    Center,
    End,
}

#[derive(Clone, Copy, Debug)]
pub struct BoxChild {
    pub min_size: Vector2,
    pub expand: bool,
    pub stretch_ratio: f32,
}

impl BoxChild {
    pub fn new(min_size: Vector2) -> Self {
        Self {
            min_size,
            expand: false,
            stretch_ratio: 1.0,
        }
    }
}

pub struct BoxContainerData {
    vertical: bool,
    alignment: Alignment,
    separation: i32,
}

impl Default for BoxContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxContainerData {
    pub fn new() -> Self {
        Self {
            vertical: false,
            alignment: Alignment::Begin,
            separation: 4,
        }
    }

    pub fn set_vertical(&mut self, vertical: bool) {
        self.vertical = vertical;
    }

    pub fn is_vertical(&self) -> bool {
        self.vertical
    }

    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }

    pub fn get_alignment(&self) -> Alignment {
        self.alignment
    }

    pub fn set_separation(&mut self, separation: i32) {
        self.separation = separation;
    }

    pub fn get_separation(&self) -> i32 {
        self.separation
    }

    /// Lays out `children` inside `container` and returns one rect per child.
    pub fn layout(&self, container: Vector2, children: &[BoxChild]) -> Vec<Rect2> {
        let mut rects = Vec::with_capacity(children.len());
        if children.is_empty() {
            return rects;
        }

        let main = if self.vertical { container.y } else { container.x };
        let cross = if self.vertical { container.x } else { container.y };
        let sep = self.separation as f32;

        let main_min = |c: &BoxChild| if self.vertical { c.min_size.y } else { c.min_size.x };

        let total_min: f32 = children.iter().map(main_min).sum::<f32>()
            + sep * (children.len() as f32 - 1.0);
        let extra = (main - total_min).max(0.0);

        let ratio_sum: f32 = children
            .iter()
            .filter(|c| c.expand)
            .map(|c| if c.stretch_ratio > 0.0 { c.stretch_ratio } else { 1.0 })
            .sum();

        let mut cursor = match self.alignment {
            Alignment::Begin => 0.0,
            Alignment::Center => (main - total_min).max(0.0) / 2.0,
            Alignment::End => (main - total_min).max(0.0),
        };

        for child in children {
            let mut size_main = main_min(child);
            if child.expand && ratio_sum > 0.0 {
                let ratio = if child.stretch_ratio > 0.0 {
                    child.stretch_ratio
                } else {
                    1.0
                };
                size_main += extra * (ratio / ratio_sum);
            }

            let rect = if self.vertical {
                Rect2::new(Vector2::new(0.0, cursor), Vector2::new(cross, size_main))
            } else {
                Rect2::new(Vector2::new(cursor, 0.0), Vector2::new(size_main, cross))
            };
            rects.push(rect);
            cursor += size_main + sep;
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTAINER: Vector2 = Vector2 { x: 100.0, y: 40.0 };

    fn children(mins: &[(f32, f32)]) -> Vec<BoxChild> {
        mins.iter()
            .map(|(w, h)| BoxChild::new(Vector2::new(*w, *h)))
            .collect()
    }

    #[test]
    fn horizontal_layout_uses_min_sizes_and_separation() {
        let c = BoxContainerData::new();
        assert_eq!(c.get_separation(), 4);
        let kids = children(&[(50.0, 20.0), (30.0, 20.0)]);
        let r = c.layout(CONTAINER, &kids);

        assert_eq!(r[0].position, Vector2::ZERO);
        assert_eq!(r[0].size, Vector2::new(50.0, 40.0));
        assert_eq!(r[1].position, Vector2::new(54.0, 0.0));
        assert_eq!(r[1].size, Vector2::new(30.0, 40.0));
    }

    #[test]
    fn expand_children_share_the_leftover_by_ratio() {
        let c = BoxContainerData::new();
        let mut kids = children(&[(40.0, 10.0), (40.0, 10.0)]);
        for k in &mut kids {
            k.expand = true;
        }
        // total min = 80 + 4 = 84; extra = 16, split 1:1.
        let r = c.layout(CONTAINER, &kids);
        assert!((r[0].size.x - 48.0).abs() < 1e-4, "{:?}", r[0].size);
        assert!((r[1].size.x - 48.0).abs() < 1e-4, "{:?}", r[1].size);
        assert!((r[1].position.x - 52.0).abs() < 1e-4, "{:?}", r[1].position);

        // With different ratios the extra follows proportionally.
        kids[0].stretch_ratio = 3.0;
        kids[1].stretch_ratio = 1.0;
        let r = c.layout(CONTAINER, &kids);
        assert!((r[0].size.x - 52.0).abs() < 1e-4, "{:?}", r[0].size);
        assert!((r[1].size.x - 44.0).abs() < 1e-4, "{:?}", r[1].size);
    }

    #[test]
    fn vertical_orientation_swaps_the_axes() {
        let mut c = BoxContainerData::new();
        c.set_vertical(true);
        let kids = children(&[(10.0, 10.0), (10.0, 15.0)]);
        let r = c.layout(Vector2::new(40.0, 100.0), &kids);

        assert_eq!(r[0].position, Vector2::ZERO);
        assert_eq!(r[0].size, Vector2::new(40.0, 10.0));
        assert_eq!(r[1].position, Vector2::new(0.0, 14.0));
        assert_eq!(r[1].size, Vector2::new(40.0, 15.0));
    }

    #[test]
    fn alignment_shifts_the_row() {
        let mut c = BoxContainerData::new();
        let kids = children(&[(40.0, 10.0), (40.0, 10.0)]);
        // total min = 84, container 100.
        c.set_alignment(Alignment::Center);
        let r = c.layout(CONTAINER, &kids);
        assert!((r[0].position.x - 8.0).abs() < 1e-4, "{:?}", r[0].position);

        c.set_alignment(Alignment::End);
        let r = c.layout(CONTAINER, &kids);
        assert!((r[0].position.x - 16.0).abs() < 1e-4, "{:?}", r[0].position);
    }

    #[test]
    fn empty_and_oversized_children_do_not_overflow_the_cursor() {
        let c = BoxContainerData::new();
        assert!(c.layout(CONTAINER, &[]).is_empty());

        // Minimums exceed the container: no expansion, rects keep their size.
        let kids = children(&[(80.0, 10.0), (80.0, 10.0)]);
        let r = c.layout(CONTAINER, &kids);
        assert_eq!(r[0].size.x, 80.0);
        assert_eq!(r[1].size.x, 80.0);
        assert_eq!(r[1].position.x, 84.0);
    }
}
