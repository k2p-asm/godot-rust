//! `scene/gui/grid_container.*`: fixed-column grid layout.
//!
//! A column is as wide as its widest child minimum; a row as tall as its
//! tallest. Leftover space is shared equally among columns/rows that contain an
//! expanding child, and `h_separation`/`v_separation` (4 by default) sit
//! between cells.
//!
//! Not ported: per-child `SIZE_SHRINK_*`, RTL and the `Container` plumbing.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct GridChild {
    pub min_size: Vector2,
    pub expand_h: bool,
    pub expand_v: bool,
}

impl GridChild {
    pub fn new(min_size: Vector2) -> Self {
        Self {
            min_size,
            expand_h: false,
            expand_v: false,
        }
    }
}

pub struct GridContainerData {
    columns: i32,
    h_separation: i32,
    v_separation: i32,
}

impl Default for GridContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl GridContainerData {
    pub fn new() -> Self {
        Self {
            columns: 1,
            h_separation: 4,
            v_separation: 4,
        }
    }

    /// `set_columns`: at least 1, like the engine's clamped property.
    pub fn set_columns(&mut self, columns: i32) {
        self.columns = columns.max(1);
    }

    pub fn get_columns(&self) -> i32 {
        self.columns
    }

    pub fn set_h_separation(&mut self, separation: i32) {
        self.h_separation = separation;
    }

    pub fn get_h_separation(&self) -> i32 {
        self.h_separation
    }

    pub fn set_v_separation(&mut self, separation: i32) {
        self.v_separation = separation;
    }

    pub fn get_v_separation(&self) -> i32 {
        self.v_separation
    }

    pub fn layout(&self, container: Vector2, children: &[GridChild]) -> Vec<Rect2> {
        let mut rects = Vec::with_capacity(children.len());
        if children.is_empty() {
            return rects;
        }

        let cols = self.columns.max(1) as usize;
        let rows = (children.len() + cols - 1) / cols;
        let hsep = self.h_separation as f32;
        let vsep = self.v_separation as f32;

        let mut col_min = vec![0.0f32; cols];
        let mut row_min = vec![0.0f32; rows];
        let mut col_expand = vec![false; cols];
        let mut row_expand = vec![false; rows];

        for (i, child) in children.iter().enumerate() {
            let c = i % cols;
            let r = i / cols;
            col_min[c] = col_min[c].max(child.min_size.x);
            row_min[r] = row_min[r].max(child.min_size.y);
            col_expand[c] |= child.expand_h;
            row_expand[r] |= child.expand_v;
        }

        let used_x: f32 = col_min.iter().sum::<f32>() + hsep * (cols as f32 - 1.0);
        let used_y: f32 = row_min.iter().sum::<f32>() + vsep * (rows as f32 - 1.0);
        let extra_x = (container.x - used_x).max(0.0);
        let extra_y = (container.y - used_y).max(0.0);

        let expanding_cols = col_expand.iter().filter(|e| **e).count();
        let expanding_rows = row_expand.iter().filter(|e| **e).count();
        if expanding_cols > 0 {
            let share = extra_x / expanding_cols as f32;
            for (c, expanding) in col_expand.iter().enumerate() {
                if *expanding {
                    col_min[c] += share;
                }
            }
        }
        if expanding_rows > 0 {
            let share = extra_y / expanding_rows as f32;
            for (r, expanding) in row_expand.iter().enumerate() {
                if *expanding {
                    row_min[r] += share;
                }
            }
        }

        let mut x_at = Vec::with_capacity(cols);
        let mut x = 0.0f32;
        for w in &col_min {
            x_at.push(x);
            x += w + hsep;
        }
        let mut y_at = Vec::with_capacity(rows);
        let mut y = 0.0f32;
        for h in &row_min {
            y_at.push(y);
            y += h + vsep;
        }

        for i in 0..children.len() {
            let c = i % cols;
            let r = i / cols;
            rects.push(Rect2::new(
                Vector2::new(x_at[c], y_at[r]),
                Vector2::new(col_min[c], row_min[r]),
            ));
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kids(mins: &[(f32, f32)]) -> Vec<GridChild> {
        mins.iter()
            .map(|(w, h)| GridChild::new(Vector2::new(*w, *h)))
            .collect()
    }

    #[test]
    fn columns_take_the_widest_child_and_rows_the_tallest() {
        let mut g = GridContainerData::new();
        g.set_columns(2);
        assert_eq!(g.get_h_separation(), 4);
        assert_eq!(g.get_v_separation(), 4);

        let k = kids(&[(10.0, 10.0), (30.0, 20.0), (20.0, 50.0)]);
        let r = g.layout(Vector2::new(100.0, 100.0), &k);

        // Row-major fill: column 0 holds children 0 and 2 (max 20), column 1
        // holds child 1 (30). Row heights are 20 and 50.
        assert_eq!(r[0], Rect2::new(Vector2::new(0.0, 0.0), Vector2::new(20.0, 20.0)));
        assert_eq!(r[1], Rect2::new(Vector2::new(24.0, 0.0), Vector2::new(30.0, 20.0)));
        assert_eq!(r[2], Rect2::new(Vector2::new(0.0, 24.0), Vector2::new(20.0, 50.0)));
    }

    #[test]
    fn expand_columns_share_the_leftover() {
        let mut g = GridContainerData::new();
        g.set_columns(2);
        let mut k = kids(&[(10.0, 10.0), (30.0, 20.0), (20.0, 50.0)]);
        k[0].expand_h = true; // column 0
        k[2].expand_v = true; // row 1

        let r = g.layout(Vector2::new(100.0, 100.0), &k);
        // extra_x = 100 - (20+30+4) = 46 -> column 0 becomes 66 wide.
        assert_eq!(r[0].size.x, 66.0);
        assert_eq!(r[1].position.x, 70.0);
        // extra_y = 100 - (20+50+4) = 26 -> row 1 is 76 tall.
        assert_eq!(r[2].size.y, 76.0);
        // Children that do not expand in that direction keep the cell size.
        assert_eq!(r[2].position.y, 24.0);
    }

    #[test]
    fn one_column_stacks_rows() {
        let g = GridContainerData::new();
        let k = kids(&[(10.0, 10.0), (10.0, 15.0)]);
        let r = g.layout(Vector2::new(50.0, 100.0), &k);
        assert_eq!(r[0].position, Vector2::ZERO);
        assert_eq!(r[1].position, Vector2::new(0.0, 14.0));
        // Cell width follows the widest minimum, not the container.
        assert_eq!(r[0].size.x, 10.0);
    }

    #[test]
    fn columns_are_clamped_and_zero_children_is_empty() {
        let mut g = GridContainerData::new();
        g.set_columns(0);
        assert_eq!(g.get_columns(), 1);
        assert!(g.layout(Vector2::ONE, &[]).is_empty());
    }

    #[test]
    fn separation_is_between_cells_only() {
        let mut g = GridContainerData::new();
        g.set_columns(3);
        g.set_h_separation(10);
        g.set_v_separation(6);
        let k = kids(&[(5.0, 5.0), (5.0, 5.0), (5.0, 5.0)]);
        let r = g.layout(Vector2::new(100.0, 20.0), &k);
        assert_eq!(r[0].position.x, 0.0);
        assert_eq!(r[1].position.x, 15.0);
        assert_eq!(r[2].position.x, 30.0);
        // Only one row, so no vertical separation is applied.
        assert!(r.iter().all(|rect| rect.position.y == 0.0));
    }
}
