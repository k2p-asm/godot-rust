//! `scene/resources/2d/tile_set.*`: the grid mapping, square cells only.
//!
//! `map_to_local` returns the **center** of the cell, `local_to_map` the cell
//! containing a local point. For `TILE_SHAPE_SQUARE` the engine's half-offset
//! correction blocks are skipped, leaving:
//!
//! - `map_to_local(p) = (p + 0.5) * tile_size`
//! - `local_to_map(p) = floor(p / tile_size)`
//!
//! Half-offset square, isometric and hexagon need the layout/corner-correction
//! matrix; they are rejected (`None`) rather than approximated.

use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileShape {
    Square,
    Isometric,
    HalfOffsetSquare,
    Hexagon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileLayout {
    Stacked,
    StackedOffset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileOffsetAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub struct TileSetData {
    tile_size: Vector2i,
    tile_shape: TileShape,
    tile_layout: TileLayout,
    tile_offset_axis: TileOffsetAxis,
}

impl Default for TileSetData {
    fn default() -> Self {
        Self::new()
    }
}

impl TileSetData {
    pub fn new() -> Self {
        Self {
            tile_size: Vector2i::new(16, 16),
            tile_shape: TileShape::Square,
            tile_layout: TileLayout::Stacked,
            tile_offset_axis: TileOffsetAxis::Horizontal,
        }
    }

    pub fn set_tile_size(&mut self, size: Vector2i) {
        self.tile_size = size;
    }

    pub fn get_tile_size(&self) -> Vector2i {
        self.tile_size
    }

    pub fn set_tile_shape(&mut self, shape: TileShape) {
        self.tile_shape = shape;
    }

    pub fn get_tile_shape(&self) -> TileShape {
        self.tile_shape
    }

    pub fn set_tile_layout(&mut self, layout: TileLayout) {
        self.tile_layout = layout;
    }

    pub fn get_tile_layout(&self) -> TileLayout {
        self.tile_layout
    }

    pub fn set_tile_offset_axis(&mut self, axis: TileOffsetAxis) {
        self.tile_offset_axis = axis;
    }

    pub fn get_tile_offset_axis(&self) -> TileOffsetAxis {
        self.tile_offset_axis
    }

    /// Cell center in local coordinates.
    ///
    /// Square cells ignore layout. For the half-offset shapes this implements
    /// the engine's `STACKED` / `STACKED_OFFSET` branches and the overlap ratio
    /// (0.5 isometric, 0.75 hexagon); `STAIRS_*` and `DIAMOND_*` are not
    /// ported and return `None`.
    pub fn map_to_local(&self, cell: Vector2i) -> Option<Vector2> {
        let mut ret = Vector2::new(cell.x as f32, cell.y as f32);

        match self.tile_shape {
            TileShape::Square => {}
            TileShape::HalfOffsetSquare | TileShape::Hexagon | TileShape::Isometric => {
                let odd = |v: i32| v.rem_euclid(2) != 0;
                let even = |v: i32| v.rem_euclid(2) == 0;
                match self.tile_offset_axis {
                    TileOffsetAxis::Horizontal => match self.tile_layout {
                        TileLayout::Stacked => {
                            if odd(cell.y) {
                                ret.x += 0.5;
                            }
                        }
                        TileLayout::StackedOffset => {
                            if even(cell.y) {
                                ret.x += 0.5;
                            }
                        }
                    },
                    TileOffsetAxis::Vertical => match self.tile_layout {
                        TileLayout::Stacked => {
                            if odd(cell.x) {
                                ret.y += 0.5;
                            }
                        }
                        TileLayout::StackedOffset => {
                            if even(cell.x) {
                                ret.y += 0.5;
                            }
                        }
                    },
                }

                let ratio = match (self.tile_offset_axis, self.tile_shape) {
                    (TileOffsetAxis::Horizontal, TileShape::Isometric) => 0.5,
                    (TileOffsetAxis::Horizontal, TileShape::Hexagon) => 0.75,
                    (TileOffsetAxis::Vertical, TileShape::Isometric) => 0.5,
                    (TileOffsetAxis::Vertical, TileShape::Hexagon) => 0.75,
                    _ => 1.0,
                };
                match self.tile_offset_axis {
                    TileOffsetAxis::Horizontal => ret.y *= ratio,
                    TileOffsetAxis::Vertical => ret.x *= ratio,
                }
            }
        }

        Some((ret + Vector2::new(0.5, 0.5)) * Vector2::new(self.tile_size.x as f32, self.tile_size.y as f32))
    }

    /// Cell containing `local`.
    ///
    /// Square cells floor directly. For the half-offset shapes this ports the
    /// engine's "smart floor" plus the top/bottom triangle correction; only
    /// `STACKED` / `STACKED_OFFSET` layouts are implemented, matching
    /// `map_to_local`.
    pub fn local_to_map(&self, local: Vector2) -> Option<Vector2i> {
        let ts = Vector2::new(self.tile_size.x as f32, self.tile_size.y as f32);
        let mut ret = Vector2::new(local.x / ts.x, local.y / ts.y);

        if self.tile_shape == TileShape::Square {
            return Some(Vector2i::new(ret.x.floor() as i32, ret.y.floor() as i32));
        }

        let ratio = match (self.tile_offset_axis, self.tile_shape) {
            (TileOffsetAxis::Horizontal, TileShape::Isometric) => 0.5,
            (TileOffsetAxis::Horizontal, TileShape::Hexagon) => 0.75,
            (TileOffsetAxis::Vertical, TileShape::Isometric) => 0.5,
            (TileOffsetAxis::Vertical, TileShape::Hexagon) => 0.75,
            _ => 1.0,
        };

        let odd = |v: f32| (v.floor() as i32).rem_euclid(2) != 0;
        let stacked_offset = self.tile_layout == TileLayout::StackedOffset;

        match self.tile_offset_axis {
            TileOffsetAxis::Horizontal => {
                ret.y /= ratio;
                let raw = ret;
                if odd(ret.y) != stacked_offset {
                    ret = Vector2::new((ret.x + 0.5).floor() - 0.5, ret.y.floor());
                } else {
                    ret = Vector2::new(ret.x.floor(), ret.y.floor());
                }

                let in_tile = raw - ret;
                let slope = 1.0 / ratio - 1.0;
                let tl = (in_tile - Vector2::new(0.5, 0.0))
                    .cross(Vector2::new(-0.5, slope))
                    <= 0.0;
                let tr = (in_tile - Vector2::new(0.5, 0.0))
                    .cross(Vector2::new(0.5, slope))
                    > 0.0;

                ret = Vector2::new(ret.x.floor(), ret.y.floor());
                match self.tile_layout {
                    TileLayout::Stacked => {
                        if tl {
                            ret += Vector2::new(if odd(ret.y) { 0.0 } else { -1.0 }, -1.0);
                        } else if tr {
                            ret += Vector2::new(if odd(ret.y) { 1.0 } else { 0.0 }, -1.0);
                        }
                    }
                    TileLayout::StackedOffset => {
                        if tl {
                            ret += Vector2::new(if odd(ret.y) { -1.0 } else { 0.0 }, -1.0);
                        } else if tr {
                            ret += Vector2::new(if odd(ret.y) { 0.0 } else { 1.0 }, -1.0);
                        }
                    }
                }
            }
            TileOffsetAxis::Vertical => {
                ret.x /= ratio;
                let raw = ret;
                if odd(ret.x) != stacked_offset {
                    ret = Vector2::new(ret.x.floor(), (ret.y + 0.5).floor() - 0.5);
                } else {
                    ret = Vector2::new(ret.x.floor(), ret.y.floor());
                }

                let in_tile = raw - ret;
                let slope = 1.0 / ratio - 1.0;
                let tl = (in_tile - Vector2::new(0.0, 0.5))
                    .cross(Vector2::new(slope, -0.5))
                    > 0.0;
                let bl = (in_tile - Vector2::new(0.0, 0.5))
                    .cross(Vector2::new(slope, 0.5))
                    <= 0.0;

                ret = Vector2::new(ret.x.floor(), ret.y.floor());
                match self.tile_layout {
                    TileLayout::Stacked => {
                        if tl {
                            ret += Vector2::new(-1.0, if odd(ret.x) { 0.0 } else { -1.0 });
                        } else if bl {
                            ret += Vector2::new(-1.0, if odd(ret.x) { 1.0 } else { 0.0 });
                        }
                    }
                    TileLayout::StackedOffset => {
                        if tl {
                            ret += Vector2::new(-1.0, if odd(ret.x) { -1.0 } else { 0.0 });
                        } else if bl {
                            ret += Vector2::new(-1.0, if odd(ret.x) { 0.0 } else { 1.0 });
                        }
                    }
                }
            }
        }

        Some(Vector2i::new(ret.x as i32, ret.y as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_cell_centers() {
        let t = TileSetData::new();
        assert_eq!(t.get_tile_size(), Vector2i::new(16, 16));
        assert_eq!(t.map_to_local(Vector2i::new(0, 0)), Some(Vector2::new(8.0, 8.0)));
        assert_eq!(t.map_to_local(Vector2i::new(-1, 3)), Some(Vector2::new(-8.0, 56.0)));
    }

    #[test]
    fn local_to_map_floors_negatives_correctly() {
        let t = TileSetData::new();
        assert_eq!(t.local_to_map(Vector2::new(15.9, 0.0)), Some(Vector2i::new(0, 0)));
        assert_eq!(t.local_to_map(Vector2::new(16.0, 32.0)), Some(Vector2i::new(1, 2)));
        // -0.1 / 16 = -0.006 -> floor = -1, not 0.
        assert_eq!(t.local_to_map(Vector2::new(-0.1, -0.1)), Some(Vector2i::new(-1, -1)));
    }

    #[test]
    fn round_trip_cell_center() {
        let t = TileSetData::new();
        for cell in [Vector2i::new(0, 0), Vector2i::new(5, -3), Vector2i::new(-7, 2)] {
            let local = t.map_to_local(cell).unwrap();
            assert_eq!(t.local_to_map(local), Some(cell));
        }
    }

    #[test]
    fn isometric_stacked_offsets_odd_rows() {
        let mut t = TileSetData::new();
        t.set_tile_shape(TileShape::Isometric);
        // (1,1): odd row adds 0.5 to x, then y is halved.
        assert_eq!(t.map_to_local(Vector2i::new(1, 1)), Some(Vector2::new(32.0, 16.0)));
        // (0,0): even row, no shift.
        assert_eq!(t.map_to_local(Vector2i::new(0, 0)), Some(Vector2::new(8.0, 8.0)));
    }

    #[test]
    fn hexagon_vertical_stacked_offset_shifts_even_columns() {
        let mut t = TileSetData::new();
        t.set_tile_shape(TileShape::Hexagon);
        t.set_tile_offset_axis(TileOffsetAxis::Vertical);
        t.set_tile_layout(TileLayout::StackedOffset);
        // (1,1): odd column, no shift; x scaled by 0.75.
        assert_eq!(t.map_to_local(Vector2i::new(1, 1)), Some(Vector2::new(20.0, 24.0)));
        // (0,0): even column, +0.5 y (the 0.5 goes in before the center offset).
        assert_eq!(t.map_to_local(Vector2i::new(0, 0)), Some(Vector2::new(8.0, 16.0)));
    }

    #[test]
    fn half_offset_round_trips_through_cell_centers() {
        for shape in [TileShape::Isometric, TileShape::Hexagon, TileShape::HalfOffsetSquare] {
            for axis in [TileOffsetAxis::Horizontal, TileOffsetAxis::Vertical] {
                for layout in [TileLayout::Stacked, TileLayout::StackedOffset] {
                    let mut t = TileSetData::new();
                    t.set_tile_shape(shape);
                    t.set_tile_offset_axis(axis);
                    t.set_tile_layout(layout);
                    for cell in [Vector2i::new(0, 0), Vector2i::new(1, 0), Vector2i::new(0, 1), Vector2i::new(3, 2), Vector2i::new(-2, 4)] {
                        let local = t.map_to_local(cell).unwrap();
                        assert_eq!(t.local_to_map(local), Some(cell), "{shape:?} {axis:?} {layout:?} {cell:?}");
                    }
                }
            }
        }
    }
}
