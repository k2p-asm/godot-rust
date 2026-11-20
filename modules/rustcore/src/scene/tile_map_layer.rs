//! `scene/2d/tile_map_layer.*`: cell storage and the invalid-value rules.
//!
//! A cell holds `(source_id, atlas_coords, alternative_tile)`. The engine treats
//! `-1` / `(-1,-1)` / `-1` as "invalid": setting a cell with **some but not all**
//! fields invalid erases it, and reads from empty cells return those sentinels.
//!
//! `get_used_cells` is sorted here for deterministic output; the engine returns
//! hash-map order.

use crate::math::vector2i::Vector2i;
use std::collections::HashMap;

pub const INVALID_SOURCE: i32 = -1;
pub const INVALID_ALTERNATIVE: i32 = -1;

pub fn invalid_atlas_coords() -> Vector2i {
    Vector2i::new(-1, -1)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileMapCell {
    pub source_id: i32,
    pub atlas_coords: Vector2i,
    pub alternative_tile: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TileMapLayerData {
    cells: HashMap<(i32, i32), TileMapCell>,
}

impl TileMapLayerData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `set_cell`. Partially invalid input erases the cell instead of storing a
    /// half-invalid one.
    pub fn set_cell(
        &mut self,
        coords: Vector2i,
        source_id: i32,
        atlas_coords: Vector2i,
        alternative_tile: i32,
    ) {
        let any_invalid = source_id == INVALID_SOURCE
            || atlas_coords == invalid_atlas_coords()
            || alternative_tile == INVALID_ALTERNATIVE;
        let all_invalid = source_id == INVALID_SOURCE
            && atlas_coords == invalid_atlas_coords()
            && alternative_tile == INVALID_ALTERNATIVE;

        if any_invalid {
            if !all_invalid {
                // The engine normalizes the partial case to "erase".
                self.cells.remove(&(coords.x, coords.y));
            } else {
                self.cells.remove(&(coords.x, coords.y));
            }
            return;
        }

        self.cells.insert(
            (coords.x, coords.y),
            TileMapCell {
                source_id,
                atlas_coords,
                alternative_tile,
            },
        );
    }

    pub fn erase_cell(&mut self, coords: Vector2i) {
        self.cells.remove(&(coords.x, coords.y));
    }

    pub fn get_cell(&self, coords: Vector2i) -> Option<TileMapCell> {
        self.cells.get(&(coords.x, coords.y)).copied()
    }

    pub fn get_cell_source_id(&self, coords: Vector2i) -> i32 {
        self.get_cell(coords).map(|c| c.source_id).unwrap_or(INVALID_SOURCE)
    }

    pub fn get_cell_atlas_coords(&self, coords: Vector2i) -> Vector2i {
        self.get_cell(coords)
            .map(|c| c.atlas_coords)
            .unwrap_or_else(invalid_atlas_coords)
    }

    pub fn get_cell_alternative_tile(&self, coords: Vector2i) -> i32 {
        self.get_cell(coords)
            .map(|c| c.alternative_tile)
            .unwrap_or(INVALID_ALTERNATIVE)
    }

    /// Used cells, sorted by `(x, y)` for deterministic output.
    pub fn get_used_cells(&self) -> Vec<Vector2i> {
        let mut out = Vec::new();
        for (k, c) in &self.cells {
            if c.source_id != INVALID_SOURCE {
                out.push(Vector2i::new(k.0, k.1));
            }
        }
        out.sort_by_key(|v| (v.x, v.y));
        out
    }

    /// `get_used_cells_by_id`: an invalid filter value acts as a wildcard.
    pub fn get_used_cells_by_id(
        &self,
        source_id: i32,
        atlas_coords: Vector2i,
        alternative_tile: i32,
    ) -> Vec<Vector2i> {
        let mut out = Vec::new();
        for (k, c) in &self.cells {
            if c.source_id == INVALID_SOURCE {
                continue;
            }
            let source_ok = source_id == INVALID_SOURCE || source_id == c.source_id;
            let atlas_ok = atlas_coords == invalid_atlas_coords() || atlas_coords == c.atlas_coords;
            let alt_ok =
                alternative_tile == INVALID_ALTERNATIVE || alternative_tile == c.alternative_tile;
            if source_ok && atlas_ok && alt_ok {
                out.push(Vector2i::new(k.0, k.1));
            }
        }
        out.sort_by_key(|v| (v.x, v.y));
        out
    }

    pub fn get_cell_count(&self) -> usize {
        self.cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(x: i32, y: i32) -> Vector2i {
        Vector2i::new(x, y)
    }

    #[test]
    fn set_and_read_cells() {
        let mut t = TileMapLayerData::new();
        t.set_cell(cell(0, 0), 0, cell(1, 2), 0);
        assert_eq!(t.get_cell_source_id(cell(0, 0)), 0);
        assert_eq!(t.get_cell_atlas_coords(cell(0, 0)), cell(1, 2));
        assert_eq!(t.get_cell_alternative_tile(cell(0, 0)), 0);
    }

    #[test]
    fn empty_cells_return_sentinels() {
        let t = TileMapLayerData::new();
        assert_eq!(t.get_cell_source_id(cell(5, 5)), INVALID_SOURCE);
        assert_eq!(t.get_cell_atlas_coords(cell(5, 5)), cell(-1, -1));
        assert_eq!(t.get_cell_alternative_tile(cell(5, 5)), INVALID_ALTERNATIVE);
    }

    #[test]
    fn partially_invalid_cells_are_erased() {
        let mut t = TileMapLayerData::new();
        t.set_cell(cell(0, 0), 0, cell(0, 0), 0);
        // source invalid, atlas/alt valid -> erase
        t.set_cell(cell(0, 0), INVALID_SOURCE, cell(0, 0), 0);
        assert_eq!(t.get_cell_source_id(cell(0, 0)), INVALID_SOURCE);
        assert_eq!(t.get_cell_count(), 0);

        // all invalid -> erase (same result, no panic)
        t.set_cell(cell(0, 0), INVALID_SOURCE, cell(-1, -1), INVALID_ALTERNATIVE);
        assert_eq!(t.get_cell_count(), 0);
    }

    #[test]
    fn erase_removes_the_cell() {
        let mut t = TileMapLayerData::new();
        t.set_cell(cell(1, 1), 2, cell(0, 0), 0);
        t.erase_cell(cell(1, 1));
        assert_eq!(t.get_cell_count(), 0);
    }

    #[test]
    fn used_cells_are_sorted() {
        let mut t = TileMapLayerData::new();
        for (x, y) in [(2, 0), (0, 5), (0, -1), (1, 1)] {
            t.set_cell(cell(x, y), 0, cell(0, 0), 0);
        }
        assert_eq!(
            t.get_used_cells(),
            vec![cell(0, -1), cell(0, 5), cell(1, 1), cell(2, 0)]
        );
    }

    #[test]
    fn used_cells_by_id_treats_invalid_as_wildcard() {
        let mut t = TileMapLayerData::new();
        t.set_cell(cell(0, 0), 0, cell(1, 0), 0);
        t.set_cell(cell(1, 0), 1, cell(1, 0), 0);

        assert_eq!(t.get_used_cells_by_id(0, cell(-1, -1), -1), vec![cell(0, 0)]);
        assert_eq!(
            t.get_used_cells_by_id(INVALID_SOURCE, cell(1, 0), -1),
            vec![cell(0, 0), cell(1, 0)]
        );
    }
}
