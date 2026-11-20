//! `scene/resources/2d/navigation_polygon.*`: 2D navigation source/settings.
//!
//! Defaults: cell size 1 m (`NavigationDefaults2D`), zero border, agent radius
//! 10, watershed partitioning, mesh-instance geometry, root-children sources,
//! parsed collision mask `0xFFFFFFFF` and group
//! `navigation_polygon_source_geometry_group`.
//!
//! Validation matches the C++: the cell size is stored unvalidated (the engine
//! forwards it to the navigation mesh), `border_size` and `agent_radius` reject
//! negatives, the enum setters reject their `MAX` sentinel and collision layers
//! are `1..=32`.
//!
//! Outlines, vertices and polygons are stored as given, with index-checked
//! accessors. Baking and the internal `NavigationMesh` mirror are not ported.

use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SamplePartitionType {
    Watershed,
    Monotone,
    Layers,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ParsedGeometryType {
    MeshInstances,
    StaticColliders,
    Both,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SourceGeometryMode {
    RootNodeChildren,
    GroupsWithChildren,
    GroupsExplicit,
    Max,
}

#[derive(Clone, Debug)]
pub struct NavigationPolygonData {
    cell_size: f32,
    border_size: f32,
    agent_radius: f32,
    partition_type: SamplePartitionType,
    parsed_geometry_type: ParsedGeometryType,
    source_geometry_mode: SourceGeometryMode,
    source_geometry_group_name: String,
    parsed_collision_mask: u32,
    vertices: Vec<Vector2>,
    polygons: Vec<Vec<i32>>,
    outlines: Vec<Vec<Vector2>>,
}

impl Default for NavigationPolygonData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationPolygonData {
    pub fn new() -> Self {
        Self {
            cell_size: 1.0,
            border_size: 0.0,
            agent_radius: 10.0,
            partition_type: SamplePartitionType::Watershed,
            parsed_geometry_type: ParsedGeometryType::MeshInstances,
            source_geometry_mode: SourceGeometryMode::RootNodeChildren,
            source_geometry_group_name: "navigation_polygon_source_geometry_group".to_string(),
            parsed_collision_mask: 0xFFFFFFFF,
            vertices: Vec::new(),
            polygons: Vec::new(),
            outlines: Vec::new(),
        }
    }

    /// Stored as given; the engine only forwards it to the navigation mesh.
    pub fn set_cell_size(&mut self, size: f32) {
        self.cell_size = size;
    }

    pub fn get_cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn set_border_size(&mut self, size: f32) -> bool {
        if size < 0.0 {
            return false;
        }
        self.border_size = size;
        true
    }

    pub fn get_border_size(&self) -> f32 {
        self.border_size
    }

    pub fn set_agent_radius(&mut self, radius: f32) -> bool {
        if radius < 0.0 {
            return false;
        }
        self.agent_radius = radius;
        true
    }

    pub fn get_agent_radius(&self) -> f32 {
        self.agent_radius
    }

    pub fn set_sample_partition_type(&mut self, value: SamplePartitionType) -> bool {
        if value == SamplePartitionType::Max {
            return false;
        }
        self.partition_type = value;
        true
    }

    pub fn get_sample_partition_type(&self) -> SamplePartitionType {
        self.partition_type
    }

    pub fn set_parsed_geometry_type(&mut self, value: ParsedGeometryType) -> bool {
        if value == ParsedGeometryType::Max {
            return false;
        }
        self.parsed_geometry_type = value;
        true
    }

    pub fn get_parsed_geometry_type(&self) -> ParsedGeometryType {
        self.parsed_geometry_type
    }

    pub fn set_source_geometry_mode(&mut self, value: SourceGeometryMode) -> bool {
        if value == SourceGeometryMode::Max {
            return false;
        }
        self.source_geometry_mode = value;
        true
    }

    pub fn get_source_geometry_mode(&self) -> SourceGeometryMode {
        self.source_geometry_mode
    }

    pub fn set_source_geometry_group_name(&mut self, name: &str) {
        self.source_geometry_group_name = name.to_string();
    }

    pub fn get_source_geometry_group_name(&self) -> &str {
        &self.source_geometry_group_name
    }

    pub fn set_parsed_collision_mask(&mut self, mask: u32) {
        self.parsed_collision_mask = mask;
    }

    pub fn get_parsed_collision_mask(&self) -> u32 {
        self.parsed_collision_mask
    }

    /// Layers are `1..=32`.
    pub fn set_parsed_collision_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.parsed_collision_mask |= bit;
        } else {
            self.parsed_collision_mask &= !bit;
        }
        true
    }

    pub fn get_parsed_collision_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        (self.parsed_collision_mask & (1u32 << (layer - 1))) != 0
    }

    // -- outlines -----------------------------------------------------------

    pub fn add_outline(&mut self, outline: Vec<Vector2>) {
        self.outlines.push(outline);
    }

    pub fn add_outline_at_index(&mut self, outline: Vec<Vector2>, index: usize) -> bool {
        if index > self.outlines.len() {
            return false;
        }
        self.outlines.insert(index, outline);
        true
    }

    pub fn get_outline(&self, index: usize) -> Option<&[Vector2]> {
        self.outlines.get(index).map(|o| o.as_slice())
    }

    pub fn get_outline_count(&self) -> usize {
        self.outlines.len()
    }

    pub fn set_outline(&mut self, index: usize, outline: Vec<Vector2>) -> bool {
        match self.outlines.get_mut(index) {
            Some(slot) => {
                *slot = outline;
                true
            }
            None => false,
        }
    }

    pub fn remove_outline(&mut self, index: usize) -> bool {
        if index >= self.outlines.len() {
            return false;
        }
        self.outlines.remove(index);
        true
    }

    pub fn clear_outlines(&mut self) {
        self.outlines.clear();
    }

    pub fn get_outlines(&self) -> &[Vec<Vector2>] {
        &self.outlines
    }

    // -- polygons and vertices ---------------------------------------------

    pub fn add_polygon(&mut self, polygon: Vec<i32>) {
        self.polygons.push(polygon);
    }

    pub fn get_polygon_count(&self) -> usize {
        self.polygons.len()
    }

    pub fn get_polygon(&self, index: usize) -> Option<&[i32]> {
        self.polygons.get(index).map(|p| p.as_slice())
    }

    pub fn clear_polygons(&mut self) {
        self.polygons.clear();
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vector2>) {
        self.vertices = vertices;
    }

    pub fn get_vertices(&self) -> &[Vector2] {
        &self.vertices
    }

    /// `clear`: drops the outlines, polygons and vertices.
    pub fn clear(&mut self) {
        self.outlines.clear();
        self.polygons.clear();
        self.vertices.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = NavigationPolygonData::new();
        assert_eq!(p.get_cell_size(), 1.0);
        assert_eq!(p.get_border_size(), 0.0);
        assert_eq!(p.get_agent_radius(), 10.0);
        assert_eq!(p.get_sample_partition_type(), SamplePartitionType::Watershed);
        assert_eq!(p.get_parsed_geometry_type(), ParsedGeometryType::MeshInstances);
        assert_eq!(
            p.get_source_geometry_mode(),
            SourceGeometryMode::RootNodeChildren
        );
        assert_eq!(
            p.get_source_geometry_group_name(),
            "navigation_polygon_source_geometry_group"
        );
        assert_eq!(p.get_parsed_collision_mask(), 0xFFFFFFFF);
        assert_eq!(p.get_outline_count(), 0);
        assert_eq!(p.get_polygon_count(), 0);
        assert!(p.get_vertices().is_empty());
    }

    #[test]
    fn size_and_enum_validation() {
        let mut p = NavigationPolygonData::new();
        // The cell size is unvalidated.
        p.set_cell_size(-2.0);
        assert_eq!(p.get_cell_size(), -2.0);

        assert!(!p.set_border_size(-0.1));
        assert!(p.set_border_size(0.5));
        assert_eq!(p.get_border_size(), 0.5);

        assert!(!p.set_agent_radius(-1.0));
        assert!(p.set_agent_radius(0.0));
        assert_eq!(p.get_agent_radius(), 0.0);

        assert!(!p.set_sample_partition_type(SamplePartitionType::Max));
        assert!(!p.set_parsed_geometry_type(ParsedGeometryType::Max));
        assert!(!p.set_source_geometry_mode(SourceGeometryMode::Max));
        assert!(p.set_sample_partition_type(SamplePartitionType::Layers));
        assert!(p.set_parsed_geometry_type(ParsedGeometryType::Both));
        assert!(p.set_source_geometry_mode(SourceGeometryMode::GroupsWithChildren));
        assert_eq!(p.get_sample_partition_type(), SamplePartitionType::Layers);
        assert_eq!(p.get_parsed_geometry_type(), ParsedGeometryType::Both);
        assert_eq!(
            p.get_source_geometry_mode(),
            SourceGeometryMode::GroupsWithChildren
        );

        assert_eq!(SamplePartitionType::Watershed as i32, 0);
        assert_eq!(SamplePartitionType::Max as i32, 3);
        assert_eq!(ParsedGeometryType::StaticColliders as i32, 1);
        assert_eq!(ParsedGeometryType::Max as i32, 3);
        assert_eq!(SourceGeometryMode::GroupsExplicit as i32, 2);
        assert_eq!(SourceGeometryMode::Max as i32, 3);
    }

    #[test]
    fn collision_layers() {
        let mut p = NavigationPolygonData::new();
        p.set_parsed_collision_mask(0);
        assert!(p.set_parsed_collision_mask_value(1, true));
        assert!(p.set_parsed_collision_mask_value(32, true));
        assert_eq!(p.get_parsed_collision_mask(), 1 | (1 << 31));
        assert!(p.get_parsed_collision_mask_value(32));
        assert!(!p.get_parsed_collision_mask_value(2));
        assert!(!p.set_parsed_collision_mask_value(0, true));
        assert!(!p.set_parsed_collision_mask_value(33, true));
        assert!(!p.get_parsed_collision_mask_value(33));
    }

    #[test]
    fn outlines() {
        let mut p = NavigationPolygonData::new();
        let square = vec![
            Vector2::ZERO,
            Vector2::new(10.0, 0.0),
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, 10.0),
        ];
        p.add_outline(square.clone());
        assert_eq!(p.get_outline_count(), 1);
        assert_eq!(p.get_outline(0), Some(&square[..]));

        let hole = vec![
            Vector2::new(2.0, 2.0),
            Vector2::new(4.0, 2.0),
            Vector2::new(4.0, 4.0),
        ];
        assert!(p.add_outline_at_index(hole.clone(), 0));
        assert_eq!(p.get_outline(0), Some(&hole[..]));
        assert!(!p.add_outline_at_index(Vec::new(), 5));

        assert!(p.set_outline(1, hole.clone()));
        assert_eq!(p.get_outline(1), Some(&hole[..]));
        assert!(!p.set_outline(9, Vec::new()));
        assert!(!p.remove_outline(9));
        assert!(p.remove_outline(0));
        assert_eq!(p.get_outline_count(), 1);

        p.clear_outlines();
        assert_eq!(p.get_outline_count(), 0);
    }

    #[test]
    fn polygons_vertices_and_clear() {
        let mut p = NavigationPolygonData::new();
        p.set_vertices(vec![Vector2::ZERO, Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)]);
        p.add_polygon(vec![0, 1, 2]);
        assert_eq!(p.get_vertices().len(), 3);
        assert_eq!(p.get_polygon_count(), 1);
        assert_eq!(p.get_polygon(0), Some(&[0, 1, 2][..]));
        assert!(p.get_polygon(4).is_none());

        p.add_outline(vec![Vector2::ZERO]);
        p.clear_polygons();
        assert_eq!(p.get_polygon_count(), 0);
        assert_eq!(p.get_outline_count(), 1);

        p.clear();
        assert_eq!(p.get_outline_count(), 0);
        assert!(p.get_vertices().is_empty());
    }
}
