//! `scene/resources/navigation_mesh.*`: baked-mesh parameters and geometry.
//!
//! Defaults: cell size/height 0.25 m, zero border, agent 1.5 m tall with a 0.5 m
//! radius, 0.25 m max climb, 45° max slope, regions 2/20, edge length 0, edge
//! error 1.3, six vertices per polygon, detail sampling 6/1, watershed
//! partitioning, mesh-instance geometry from the root node's children, collision
//! mask `0xFFFFFFFF`, group `navigation_mesh_source_group`, all filters off and a
//! zero baking AABB.
//!
//! Every setter validates exactly like the C++: cell size/height `> 0`, most
//! distances `>= 0`, max slope `0..=90`, vertices per polygon `>= 3`, detail
//! sample distance `>= 0.1`, collision layers `1..=32`, and the enum setters
//! reject their `MAX` sentinel. Vertices and polygons are stored as given.
//!
//! Baking (`bake_from_source_geometry_data`), the merged mesh and the
//! `changed` signal are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;

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
pub struct NavigationMeshData {
    // Rasterization.
    cell_size: f32,
    cell_height: f32,
    border_size: f32,
    // Agents.
    agent_height: f32,
    agent_radius: f32,
    agent_max_climb: f32,
    agent_max_slope: f32,
    // Regions.
    region_min_size: f32,
    region_merge_size: f32,
    // Edges.
    edge_max_length: f32,
    edge_max_error: f32,
    // Polygons.
    vertices_per_polygon: f32,
    // Details.
    detail_sample_distance: f32,
    detail_sample_max_error: f32,
    // Partitioning / geometry sources.
    partition_type: SamplePartitionType,
    parsed_geometry_type: ParsedGeometryType,
    source_geometry_mode: SourceGeometryMode,
    source_group_name: String,
    collision_mask: u32,
    // Filters.
    filter_low_hanging_obstacles: bool,
    filter_ledge_spans: bool,
    filter_walkable_low_height_spans: bool,
    filter_baking_aabb: Aabb,
    filter_baking_aabb_offset: Vector3,
    // Geometry.
    vertices: Vec<Vector3>,
    polygons: Vec<Vec<i32>>,
}

impl Default for NavigationMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationMeshData {
    pub fn new() -> Self {
        Self {
            cell_size: 0.25,
            cell_height: 0.25,
            border_size: 0.0,
            agent_height: 1.5,
            agent_radius: 0.5,
            agent_max_climb: 0.25,
            agent_max_slope: 45.0,
            region_min_size: 2.0,
            region_merge_size: 20.0,
            edge_max_length: 0.0,
            edge_max_error: 1.3,
            vertices_per_polygon: 6.0,
            detail_sample_distance: 6.0,
            detail_sample_max_error: 1.0,
            partition_type: SamplePartitionType::Watershed,
            parsed_geometry_type: ParsedGeometryType::MeshInstances,
            source_geometry_mode: SourceGeometryMode::RootNodeChildren,
            source_group_name: "navigation_mesh_source_group".to_string(),
            collision_mask: 0xFFFFFFFF,
            filter_low_hanging_obstacles: false,
            filter_ledge_spans: false,
            filter_walkable_low_height_spans: false,
            filter_baking_aabb: Aabb::default(),
            filter_baking_aabb_offset: Vector3::ZERO,
            vertices: Vec::new(),
            polygons: Vec::new(),
        }
    }

    // -- rasterization ------------------------------------------------------

    /// `p_value <= 0` is rejected.
    pub fn set_cell_size(&mut self, value: f32) -> bool {
        if value <= 0.0 {
            return false;
        }
        self.cell_size = value;
        true
    }

    pub fn get_cell_size(&self) -> f32 {
        self.cell_size
    }

    pub fn set_cell_height(&mut self, value: f32) -> bool {
        if value <= 0.0 {
            return false;
        }
        self.cell_height = value;
        true
    }

    pub fn get_cell_height(&self) -> f32 {
        self.cell_height
    }

    pub fn set_border_size(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.border_size = value;
        true
    }

    pub fn get_border_size(&self) -> f32 {
        self.border_size
    }

    // -- agents -------------------------------------------------------------

    pub fn set_agent_height(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.agent_height = value;
        true
    }

    pub fn get_agent_height(&self) -> f32 {
        self.agent_height
    }

    pub fn set_agent_radius(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.agent_radius = value;
        true
    }

    pub fn get_agent_radius(&self) -> f32 {
        self.agent_radius
    }

    pub fn set_agent_max_climb(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.agent_max_climb = value;
        true
    }

    pub fn get_agent_max_climb(&self) -> f32 {
        self.agent_max_climb
    }

    /// `0..=90` degrees.
    pub fn set_agent_max_slope(&mut self, value: f32) -> bool {
        if !(0.0..=90.0).contains(&value) {
            return false;
        }
        self.agent_max_slope = value;
        true
    }

    pub fn get_agent_max_slope(&self) -> f32 {
        self.agent_max_slope
    }

    // -- regions ------------------------------------------------------------

    pub fn set_region_min_size(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.region_min_size = value;
        true
    }

    pub fn get_region_min_size(&self) -> f32 {
        self.region_min_size
    }

    pub fn set_region_merge_size(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.region_merge_size = value;
        true
    }

    pub fn get_region_merge_size(&self) -> f32 {
        self.region_merge_size
    }

    // -- edges --------------------------------------------------------------

    pub fn set_edge_max_length(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.edge_max_length = value;
        true
    }

    pub fn get_edge_max_length(&self) -> f32 {
        self.edge_max_length
    }

    pub fn set_edge_max_error(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.edge_max_error = value;
        true
    }

    pub fn get_edge_max_error(&self) -> f32 {
        self.edge_max_error
    }

    // -- polygons and details ----------------------------------------------

    /// `p_value < 3` is rejected.
    pub fn set_vertices_per_polygon(&mut self, value: f32) -> bool {
        if value < 3.0 {
            return false;
        }
        self.vertices_per_polygon = value;
        true
    }

    pub fn get_vertices_per_polygon(&self) -> f32 {
        self.vertices_per_polygon
    }

    /// `p_value < 0.1` is rejected.
    pub fn set_detail_sample_distance(&mut self, value: f32) -> bool {
        if value < 0.1 {
            return false;
        }
        self.detail_sample_distance = value;
        true
    }

    pub fn get_detail_sample_distance(&self) -> f32 {
        self.detail_sample_distance
    }

    pub fn set_detail_sample_max_error(&mut self, value: f32) -> bool {
        if value < 0.0 {
            return false;
        }
        self.detail_sample_max_error = value;
        true
    }

    pub fn get_detail_sample_max_error(&self) -> f32 {
        self.detail_sample_max_error
    }

    // -- partitioning and sources ------------------------------------------

    /// `ERR_FAIL_INDEX(value, SAMPLE_PARTITION_MAX)`.
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

    /// `ERR_FAIL_INDEX(value, PARSED_GEOMETRY_MAX)`.
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

    /// `ERR_FAIL_INDEX(value, SOURCE_GEOMETRY_MAX)`.
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

    pub fn set_source_group_name(&mut self, name: &str) {
        self.source_group_name = name.to_string();
    }

    pub fn get_source_group_name(&self) -> &str {
        &self.source_group_name
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_mask = mask;
    }

    pub fn get_collision_mask(&self) -> u32 {
        self.collision_mask
    }

    /// Layer numbers are `1..=32`.
    pub fn set_collision_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.collision_mask |= bit;
        } else {
            self.collision_mask &= !bit;
        }
        true
    }

    /// Invalid layers read as `false`.
    pub fn get_collision_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        (self.collision_mask & (1u32 << (layer - 1))) != 0
    }

    // -- filters ------------------------------------------------------------

    pub fn set_filter_low_hanging_obstacles(&mut self, enable: bool) {
        self.filter_low_hanging_obstacles = enable;
    }

    pub fn get_filter_low_hanging_obstacles(&self) -> bool {
        self.filter_low_hanging_obstacles
    }

    pub fn set_filter_ledge_spans(&mut self, enable: bool) {
        self.filter_ledge_spans = enable;
    }

    pub fn get_filter_ledge_spans(&self) -> bool {
        self.filter_ledge_spans
    }

    pub fn set_filter_walkable_low_height_spans(&mut self, enable: bool) {
        self.filter_walkable_low_height_spans = enable;
    }

    pub fn get_filter_walkable_low_height_spans(&self) -> bool {
        self.filter_walkable_low_height_spans
    }

    pub fn set_filter_baking_aabb(&mut self, aabb: Aabb) {
        self.filter_baking_aabb = aabb;
    }

    pub fn get_filter_baking_aabb(&self) -> Aabb {
        self.filter_baking_aabb
    }

    pub fn set_filter_baking_aabb_offset(&mut self, offset: Vector3) {
        self.filter_baking_aabb_offset = offset;
    }

    pub fn get_filter_baking_aabb_offset(&self) -> Vector3 {
        self.filter_baking_aabb_offset
    }

    // -- geometry -----------------------------------------------------------

    pub fn set_vertices(&mut self, vertices: Vec<Vector3>) {
        self.vertices = vertices;
    }

    pub fn get_vertices(&self) -> &[Vector3] {
        &self.vertices
    }

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

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.polygons.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let m = NavigationMeshData::new();
        assert_eq!(m.get_cell_size(), 0.25);
        assert_eq!(m.get_cell_height(), 0.25);
        assert_eq!(m.get_border_size(), 0.0);
        assert_eq!(m.get_agent_height(), 1.5);
        assert_eq!(m.get_agent_radius(), 0.5);
        assert_eq!(m.get_agent_max_climb(), 0.25);
        assert_eq!(m.get_agent_max_slope(), 45.0);
        assert_eq!(m.get_region_min_size(), 2.0);
        assert_eq!(m.get_region_merge_size(), 20.0);
        assert_eq!(m.get_edge_max_length(), 0.0);
        assert_eq!(m.get_edge_max_error(), 1.3);
        assert_eq!(m.get_vertices_per_polygon(), 6.0);
        assert_eq!(m.get_detail_sample_distance(), 6.0);
        assert_eq!(m.get_detail_sample_max_error(), 1.0);
        assert_eq!(m.get_sample_partition_type(), SamplePartitionType::Watershed);
        assert_eq!(m.get_parsed_geometry_type(), ParsedGeometryType::MeshInstances);
        assert_eq!(
            m.get_source_geometry_mode(),
            SourceGeometryMode::RootNodeChildren
        );
        assert_eq!(m.get_source_group_name(), "navigation_mesh_source_group");
        assert_eq!(m.get_collision_mask(), 0xFFFFFFFF);
        assert!(!m.get_filter_low_hanging_obstacles());
        assert!(!m.get_filter_ledge_spans());
        assert!(!m.get_filter_walkable_low_height_spans());
        assert_eq!(m.get_filter_baking_aabb(), Aabb::default());
        assert_eq!(m.get_filter_baking_aabb_offset(), Vector3::ZERO);
        assert!(m.get_vertices().is_empty());
        assert_eq!(m.get_polygon_count(), 0);
    }

    #[test]
    fn rasterization_and_agent_bounds() {
        let mut m = NavigationMeshData::new();
        assert!(!m.set_cell_size(0.0));
        assert!(!m.set_cell_size(-1.0));
        assert!(m.set_cell_size(0.5));
        assert_eq!(m.get_cell_size(), 0.5);

        assert!(!m.set_cell_height(0.0));
        assert!(m.set_cell_height(0.1));
        assert_eq!(m.get_cell_height(), 0.1);

        assert!(!m.set_border_size(-0.1));
        assert!(m.set_border_size(0.0));
        assert!(m.set_border_size(2.0));
        assert_eq!(m.get_border_size(), 2.0);

        assert!(!m.set_agent_height(-1.0));
        assert!(!m.set_agent_radius(-1.0));
        assert!(!m.set_agent_max_climb(-1.0));
        assert!(m.set_agent_height(0.0));
        assert!(m.set_agent_radius(0.0));
        assert!(m.set_agent_max_climb(0.0));

        // Slope is capped at 90 degrees.
        assert!(!m.set_agent_max_slope(-0.01));
        assert!(!m.set_agent_max_slope(90.01));
        assert!(m.set_agent_max_slope(0.0));
        assert!(m.set_agent_max_slope(90.0));
        assert_eq!(m.get_agent_max_slope(), 90.0);
    }

    #[test]
    fn region_edge_and_detail_bounds() {
        let mut m = NavigationMeshData::new();
        assert!(!m.set_region_min_size(-1.0));
        assert!(!m.set_region_merge_size(-1.0));
        assert!(!m.set_edge_max_length(-1.0));
        assert!(!m.set_edge_max_error(-1.0));
        assert!(!m.set_detail_sample_max_error(-1.0));
        assert!(m.set_region_min_size(0.0));
        assert!(m.set_edge_max_length(0.0));

        // Vertices per polygon must be at least 3.
        assert!(!m.set_vertices_per_polygon(2.99));
        assert!(m.set_vertices_per_polygon(3.0));
        assert_eq!(m.get_vertices_per_polygon(), 3.0);

        // Detail sampling needs 0.1 m.
        assert!(!m.set_detail_sample_distance(0.099));
        assert!(m.set_detail_sample_distance(0.1));
        assert_eq!(m.get_detail_sample_distance(), 0.1);
    }

    #[test]
    fn enum_sentinels_are_rejected() {
        let mut m = NavigationMeshData::new();
        assert!(!m.set_sample_partition_type(SamplePartitionType::Max));
        assert!(!m.set_parsed_geometry_type(ParsedGeometryType::Max));
        assert!(!m.set_source_geometry_mode(SourceGeometryMode::Max));
        assert_eq!(m.get_sample_partition_type(), SamplePartitionType::Watershed);

        assert!(m.set_sample_partition_type(SamplePartitionType::Layers));
        assert!(m.set_parsed_geometry_type(ParsedGeometryType::Both));
        assert!(m.set_source_geometry_mode(SourceGeometryMode::GroupsExplicit));
        assert_eq!(m.get_sample_partition_type(), SamplePartitionType::Layers);
        assert_eq!(m.get_parsed_geometry_type(), ParsedGeometryType::Both);
        assert_eq!(
            m.get_source_geometry_mode(),
            SourceGeometryMode::GroupsExplicit
        );

        assert_eq!(SamplePartitionType::Watershed as i32, 0);
        assert_eq!(SamplePartitionType::Monotone as i32, 1);
        assert_eq!(SamplePartitionType::Layers as i32, 2);
        assert_eq!(SamplePartitionType::Max as i32, 3);
        assert_eq!(ParsedGeometryType::MeshInstances as i32, 0);
        assert_eq!(ParsedGeometryType::StaticColliders as i32, 1);
        assert_eq!(ParsedGeometryType::Both as i32, 2);
        assert_eq!(ParsedGeometryType::Max as i32, 3);
        assert_eq!(SourceGeometryMode::RootNodeChildren as i32, 0);
        assert_eq!(SourceGeometryMode::GroupsWithChildren as i32, 1);
        assert_eq!(SourceGeometryMode::GroupsExplicit as i32, 2);
        assert_eq!(SourceGeometryMode::Max as i32, 3);
    }

    #[test]
    fn collision_layers() {
        let mut m = NavigationMeshData::new();
        m.set_collision_mask(0);
        assert!(m.set_collision_mask_value(1, true));
        assert_eq!(m.get_collision_mask(), 1);
        assert!(m.set_collision_mask_value(32, true));
        assert_eq!(m.get_collision_mask(), 1 | (1 << 31));
        assert!(m.get_collision_mask_value(32));
        assert!(!m.get_collision_mask_value(2));
        assert!(m.set_collision_mask_value(32, false));
        assert_eq!(m.get_collision_mask(), 1);

        assert!(!m.set_collision_mask_value(0, true));
        assert!(!m.set_collision_mask_value(33, true));
        assert!(!m.get_collision_mask_value(0));
        assert!(!m.get_collision_mask_value(33));
    }

    #[test]
    fn filters_and_geometry() {
        let mut m = NavigationMeshData::new();
        m.set_filter_low_hanging_obstacles(true);
        m.set_filter_ledge_spans(true);
        m.set_filter_walkable_low_height_spans(true);
        m.set_filter_baking_aabb(Aabb::new(Vector3::ZERO, Vector3::ONE));
        m.set_filter_baking_aabb_offset(Vector3::new(1.0, 2.0, 3.0));
        m.set_source_group_name("nav_sources");
        assert!(m.get_filter_low_hanging_obstacles());
        assert!(m.get_filter_ledge_spans());
        assert!(m.get_filter_walkable_low_height_spans());
        assert_eq!(m.get_filter_baking_aabb().size, Vector3::ONE);
        assert_eq!(
            m.get_filter_baking_aabb_offset(),
            Vector3::new(1.0, 2.0, 3.0)
        );
        assert_eq!(m.get_source_group_name(), "nav_sources");

        m.set_vertices(vec![
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ]);
        assert_eq!(m.get_vertices().len(), 3);
        m.add_polygon(vec![0, 1, 2]);
        assert_eq!(m.get_polygon_count(), 1);
        assert_eq!(m.get_polygon(0), Some(&[0, 1, 2][..]));
        assert!(m.get_polygon(1).is_none());

        m.clear_polygons();
        assert_eq!(m.get_polygon_count(), 0);
        assert_eq!(m.get_vertices().len(), 3);
        m.clear();
        assert!(m.get_vertices().is_empty());
    }
}
