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

#[derive(Clone, Copy, Debug)]
pub struct TileSetData {
    tile_size: Vector2i,
    tile_shape: TileShape,
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

    /// Cell center in local coordinates.
    pub fn map_to_local(&self, cell: Vector2i) -> Option<Vector2> {
        if self.tile_shape != TileShape::Square {
            return None;
        }
        Some(Vector2::new(
            (cell.x as f32 + 0.5) * self.tile_size.x as f32,
            (cell.y as f32 + 0.5) * self.tile_size.y as f32,
        ))
    }

    /// Cell containing `local`, flooring like the engine's square path.
    pub fn local_to_map(&self, local: Vector2) -> Option<Vector2i> {
        if self.tile_shape != TileShape::Square {
            return None;
        }
        Some(Vector2i::new(
            (local.x / self.tile_size.x as f32).floor() as i32,
            (local.y / self.tile_size.y as f32).floor() as i32,
        ))
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
    fn non_square_shapes_are_rejected() {
        let mut t = TileSetData::new();
        t.set_tile_shape(TileShape::Isometric);
        assert!(t.map_to_local(Vector2i::new(1, 1)).is_none());
        assert!(t.local_to_map(Vector2::ONE).is_none());
    }
}
