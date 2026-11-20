//! `scene/3d/navigation/navigation_agent_3d.*`: pathfinding/avoidance state.
//!
//! The constructor defaults come from the header and the navigation constants:
//! avoidance enabled on mask/layers 1 with priority 1, A* pathfinding, corridor
//! funnel post-processing, all metadata, path and target desired distances 1,
//! agent height 1 and radius 0.5, no path height offset, neighbour distance 50,
//! ten neighbours, agent/obstacle horizons 1/0, max speed 10, path max distance
//! 5, simplification off, unbounded returned path length/radius and
//! `path_search_max_polygons` 4096, plus the target/velocity bookkeeping and the
//! debug draw options.
//!
//! Ported: every setting and the result bookkeeping (`target_position`,
//! `target_reached`, `navigation_finished`, `last_waypoint_reached`,
//! `navigation_path_index`, safe/velocity/forced velocity + submission flags and
//! the kept Y velocity). The path query, avoidance simulation and map lookups are
//! not ported; `set_path_result` stands in for the async result.

use crate::math::color::Color;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PathfindingAlgorithm {
    Astar,
    Dijkstra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PathPostProcessing {
    CorridorFunnel,
    LeftCorridorFunnel,
    RightCorridorFunnel,
    CenteredFunnel,
    None,
}

/// `PathMetadataFlags` bits used by the query.
pub const PATH_METADATA_INCLUDE_NONE: u32 = 0;
pub const PATH_METADATA_INCLUDE_TYPES: u32 = 1;
pub const PATH_METADATA_INCLUDE_RIDS: u32 = 2;
pub const PATH_METADATA_INCLUDE_OWNERS: u32 = 4;
pub const PATH_METADATA_INCLUDE_ALL: u32 = 7;

#[derive(Clone, Debug)]
pub struct NavigationAgent3DData {
    avoidance_enabled: bool,
    avoidance_mask: u32,
    avoidance_priority: f32,
    navigation_layers: u32,
    pathfinding_algorithm: PathfindingAlgorithm,
    path_postprocessing: PathPostProcessing,
    path_metadata_flags: u32,
    path_desired_distance: f32,
    target_desired_distance: f32,
    height: f32,
    radius: f32,
    path_height_offset: f32,
    neighbor_distance: f32,
    max_neighbors: i32,
    time_horizon_agents: f32,
    time_horizon_obstacles: f32,
    max_speed: f32,
    path_max_distance: f32,
    simplify_path: bool,
    simplify_epsilon: f32,
    path_return_max_length: f32,
    path_return_max_radius: f32,
    path_search_max_polygons: i32,
    path_search_max_distance: f32,
    // Results / bookkeeping.
    target_position: Vector3,
    navigation_path_index: i32,
    safe_velocity: Vector3,
    velocity: Vector3,
    velocity_submitted: bool,
    velocity_forced: Vector3,
    velocity_forced_submitted: bool,
    keep_y_velocity: bool,
    stored_y_velocity: f32,
    target_position_submitted: bool,
    target_reached: bool,
    navigation_finished: bool,
    last_waypoint_reached: bool,
    // Debug.
    debug_enabled: bool,
    debug_path_custom_point_size: f32,
    debug_use_custom: bool,
    debug_path_custom_color: Color,
}

impl Default for NavigationAgent3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationAgent3DData {
    pub fn new() -> Self {
        Self {
            avoidance_enabled: true,
            avoidance_mask: 1,
            avoidance_priority: 1.0,
            navigation_layers: 1,
            pathfinding_algorithm: PathfindingAlgorithm::Astar,
            path_postprocessing: PathPostProcessing::CorridorFunnel,
            path_metadata_flags: PATH_METADATA_INCLUDE_ALL,
            path_desired_distance: 1.0,
            target_desired_distance: 1.0,
            height: 1.0,
            radius: 0.5,
            path_height_offset: 0.0,
            neighbor_distance: 50.0,
            max_neighbors: 10,
            time_horizon_agents: 1.0,
            time_horizon_obstacles: 0.0,
            max_speed: 10.0,
            path_max_distance: 5.0,
            simplify_path: false,
            simplify_epsilon: 0.0,
            path_return_max_length: 0.0,
            path_return_max_radius: 0.0,
            path_search_max_polygons: 4096,
            path_search_max_distance: 0.0,
            target_position: Vector3::ZERO,
            navigation_path_index: 0,
            safe_velocity: Vector3::ZERO,
            velocity: Vector3::ZERO,
            velocity_submitted: false,
            velocity_forced: Vector3::ZERO,
            velocity_forced_submitted: false,
            keep_y_velocity: true,
            stored_y_velocity: 0.0,
            target_position_submitted: false,
            target_reached: false,
            navigation_finished: true,
            last_waypoint_reached: false,
            debug_enabled: false,
            debug_path_custom_point_size: 4.0,
            debug_use_custom: false,
            debug_path_custom_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// Setters mirror the engine's plain assignments.

    pub fn set_avoidance_enabled(&mut self, enabled: bool) {
        self.avoidance_enabled = enabled;
    }

    pub fn get_avoidance_enabled(&self) -> bool {
        self.avoidance_enabled
    }

    pub fn set_avoidance_mask(&mut self, mask: u32) {
        self.avoidance_mask = mask;
    }

    pub fn get_avoidance_mask(&self) -> u32 {
        self.avoidance_mask
    }

    pub fn set_avoidance_priority(&mut self, priority: f32) {
        self.avoidance_priority = priority;
    }

    pub fn get_avoidance_priority(&self) -> f32 {
        self.avoidance_priority
    }

    pub fn set_navigation_layers(&mut self, layers: u32) {
        self.navigation_layers = layers;
    }

    pub fn get_navigation_layers(&self) -> u32 {
        self.navigation_layers
    }

    pub fn set_pathfinding_algorithm(&mut self, algorithm: PathfindingAlgorithm) {
        self.pathfinding_algorithm = algorithm;
    }

    pub fn get_pathfinding_algorithm(&self) -> PathfindingAlgorithm {
        self.pathfinding_algorithm
    }

    pub fn set_path_postprocessing(&mut self, postprocessing: PathPostProcessing) {
        self.path_postprocessing = postprocessing;
    }

    pub fn get_path_postprocessing(&self) -> PathPostProcessing {
        self.path_postprocessing
    }

    pub fn set_path_metadata_flags(&mut self, flags: u32) {
        self.path_metadata_flags = flags;
    }

    pub fn get_path_metadata_flags(&self) -> u32 {
        self.path_metadata_flags
    }

    pub fn set_path_desired_distance(&mut self, distance: f32) {
        self.path_desired_distance = distance;
    }

    pub fn get_path_desired_distance(&self) -> f32 {
        self.path_desired_distance
    }

    pub fn set_target_desired_distance(&mut self, distance: f32) {
        self.target_desired_distance = distance;
    }

    pub fn get_target_desired_distance(&self) -> f32 {
        self.target_desired_distance
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_path_height_offset(&mut self, offset: f32) {
        self.path_height_offset = offset;
    }

    pub fn get_path_height_offset(&self) -> f32 {
        self.path_height_offset
    }

    pub fn set_neighbor_distance(&mut self, distance: f32) {
        self.neighbor_distance = distance;
    }

    pub fn get_neighbor_distance(&self) -> f32 {
        self.neighbor_distance
    }

    pub fn set_max_neighbors(&mut self, neighbors: i32) {
        self.max_neighbors = neighbors;
    }

    pub fn get_max_neighbors(&self) -> i32 {
        self.max_neighbors
    }

    pub fn set_time_horizon_agents(&mut self, horizon: f32) {
        self.time_horizon_agents = horizon;
    }

    pub fn get_time_horizon_agents(&self) -> f32 {
        self.time_horizon_agents
    }

    pub fn set_time_horizon_obstacles(&mut self, horizon: f32) {
        self.time_horizon_obstacles = horizon;
    }

    pub fn get_time_horizon_obstacles(&self) -> f32 {
        self.time_horizon_obstacles
    }

    pub fn set_max_speed(&mut self, speed: f32) {
        self.max_speed = speed;
    }

    pub fn get_max_speed(&self) -> f32 {
        self.max_speed
    }

    pub fn set_path_max_distance(&mut self, distance: f32) {
        self.path_max_distance = distance;
    }

    pub fn get_path_max_distance(&self) -> f32 {
        self.path_max_distance
    }

    pub fn set_simplify_path(&mut self, enabled: bool) {
        self.simplify_path = enabled;
    }

    pub fn get_simplify_path(&self) -> bool {
        self.simplify_path
    }

    pub fn set_simplify_epsilon(&mut self, epsilon: f32) {
        self.simplify_epsilon = epsilon;
    }

    pub fn get_simplify_epsilon(&self) -> f32 {
        self.simplify_epsilon
    }

    pub fn set_path_return_max_length(&mut self, length: f32) {
        self.path_return_max_length = length;
    }

    pub fn get_path_return_max_length(&self) -> f32 {
        self.path_return_max_length
    }

    pub fn set_path_return_max_radius(&mut self, radius: f32) {
        self.path_return_max_radius = radius;
    }

    pub fn get_path_return_max_radius(&self) -> f32 {
        self.path_return_max_radius
    }

    pub fn set_path_search_max_polygons(&mut self, count: i32) {
        self.path_search_max_polygons = count;
    }

    pub fn get_path_search_max_polygons(&self) -> i32 {
        self.path_search_max_polygons
    }

    pub fn set_path_search_max_distance(&mut self, distance: f32) {
        self.path_search_max_distance = distance;
    }

    pub fn get_path_search_max_distance(&self) -> f32 {
        self.path_search_max_distance
    }

    // -- targets and results -------------------------------------------------

    /// `set_target_position`: latches the submitted flag and clears the reached
    /// flags until the next query.
    pub fn set_target_position(&mut self, position: Vector3) {
        self.target_position = position;
        self.target_position_submitted = true;
        self.target_reached = false;
        self.navigation_finished = false;
        self.last_waypoint_reached = false;
    }

    pub fn get_target_position(&self) -> Vector3 {
        self.target_position
    }

    pub fn is_target_position_submitted(&self) -> bool {
        self.target_position_submitted
    }

    pub fn is_target_reached(&self) -> bool {
        self.target_reached
    }

    pub fn is_navigation_finished(&self) -> bool {
        self.navigation_finished
    }

    pub fn is_last_waypoint_reached(&self) -> bool {
        self.last_waypoint_reached
    }

    /// Marks the query result (`_update_navigation`'s outcome).
    pub fn set_path_result(&mut self, finished: bool, reached: bool) {
        self.navigation_finished = finished;
        self.target_reached = reached;
        self.target_position_submitted = false;
        self.navigation_path_index = 0;
    }

    pub fn get_path_index(&self) -> i32 {
        self.navigation_path_index
    }

    pub fn set_path_index(&mut self, index: i32) {
        self.navigation_path_index = index;
    }

    pub fn set_safe_velocity(&mut self, velocity: Vector3) {
        self.safe_velocity = velocity;
    }

    pub fn get_safe_velocity(&self) -> Vector3 {
        self.safe_velocity
    }

    /// Submitting a velocity latches `velocity_submitted`.
    pub fn set_velocity(&mut self, velocity: Vector3) {
        self.velocity = velocity;
        self.velocity_submitted = true;
    }

    pub fn set_velocity_forced(&mut self, velocity: Vector3) {
        self.velocity_forced = velocity;
        self.velocity_forced_submitted = true;
    }

    pub fn is_velocity_submitted(&self) -> bool {
        self.velocity_submitted
    }

    pub fn is_velocity_forced_submitted(&self) -> bool {
        self.velocity_forced_submitted
    }

    /// Consumes the submitted velocities (the engine clears the flags in its
    /// avoidance step, keeping the forced velocity).
    pub fn consume_velocities(&mut self) -> (Vector3, Vector3) {
        self.velocity_submitted = false;
        self.velocity_forced_submitted = false;
        (self.velocity, self.velocity_forced)
    }

    pub fn set_keep_y_velocity(&mut self, keep: bool) {
        self.keep_y_velocity = keep;
    }

    pub fn get_keep_y_velocity(&self) -> bool {
        self.keep_y_velocity
    }

    pub fn set_stored_y_velocity(&mut self, velocity: f32) {
        self.stored_y_velocity = velocity;
    }

    pub fn get_stored_y_velocity(&self) -> f32 {
        self.stored_y_velocity
    }

    // -- debug ---------------------------------------------------------------

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    pub fn get_debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    pub fn set_debug_path_custom_point_size(&mut self, size: f32) {
        self.debug_path_custom_point_size = size;
    }

    pub fn get_debug_path_custom_point_size(&self) -> f32 {
        self.debug_path_custom_point_size
    }

    pub fn set_debug_use_custom(&mut self, use_custom: bool) {
        self.debug_use_custom = use_custom;
    }

    pub fn get_debug_use_custom(&self) -> bool {
        self.debug_use_custom
    }

    pub fn set_debug_path_custom_color(&mut self, color: Color) {
        self.debug_path_custom_color = color;
    }

    pub fn get_debug_path_custom_color(&self) -> Color {
        self.debug_path_custom_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let a = NavigationAgent3DData::new();
        assert!(a.get_avoidance_enabled());
        assert_eq!(a.get_avoidance_mask(), 1);
        assert_eq!(a.get_avoidance_priority(), 1.0);
        assert_eq!(a.get_navigation_layers(), 1);
        assert_eq!(a.get_pathfinding_algorithm(), PathfindingAlgorithm::Astar);
        assert_eq!(
            a.get_path_postprocessing(),
            PathPostProcessing::CorridorFunnel
        );
        assert_eq!(a.get_path_metadata_flags(), PATH_METADATA_INCLUDE_ALL);
        assert_eq!(a.get_path_desired_distance(), 1.0);
        assert_eq!(a.get_target_desired_distance(), 1.0);
        assert_eq!(a.get_height(), 1.0);
        assert_eq!(a.get_radius(), 0.5);
        assert_eq!(a.get_path_height_offset(), 0.0);
        assert_eq!(a.get_neighbor_distance(), 50.0);
        assert_eq!(a.get_max_neighbors(), 10);
        assert_eq!(a.get_time_horizon_agents(), 1.0);
        assert_eq!(a.get_time_horizon_obstacles(), 0.0);
        assert_eq!(a.get_max_speed(), 10.0);
        assert_eq!(a.get_path_max_distance(), 5.0);
        assert!(!a.get_simplify_path());
        assert_eq!(a.get_simplify_epsilon(), 0.0);
        assert_eq!(a.get_path_return_max_length(), 0.0);
        assert_eq!(a.get_path_return_max_radius(), 0.0);
        assert_eq!(a.get_path_search_max_polygons(), 4096);
        assert_eq!(a.get_path_search_max_distance(), 0.0);
        assert_eq!(a.get_target_position(), Vector3::ZERO);
        assert!(!a.is_target_position_submitted());
        assert!(!a.is_target_reached());
        assert!(a.is_navigation_finished());
        assert!(!a.is_last_waypoint_reached());
        assert!(!a.is_velocity_submitted());
        assert!(!a.is_velocity_forced_submitted());
        assert!(a.get_keep_y_velocity());
        assert_eq!(a.get_stored_y_velocity(), 0.0);
        assert!(!a.get_debug_enabled());
        assert_eq!(a.get_debug_path_custom_point_size(), 4.0);
        assert!(!a.get_debug_use_custom());
        assert_eq!(a.get_debug_path_custom_color(), Color::new(1.0, 1.0, 1.0, 1.0));

        assert_eq!(PathfindingAlgorithm::Astar as i32, 0);
        assert_eq!(PathfindingAlgorithm::Dijkstra as i32, 1);
        assert_eq!(PathPostProcessing::CorridorFunnel as i32, 0);
        assert_eq!(PathPostProcessing::None as i32, 4);
        assert_eq!(PATH_METADATA_INCLUDE_ALL, 7);
    }

    #[test]
    fn settings_round_trip() {
        let mut a = NavigationAgent3DData::new();
        a.set_avoidance_enabled(false);
        a.set_avoidance_mask(4);
        a.set_avoidance_priority(2.0);
        a.set_navigation_layers(3);
        a.set_pathfinding_algorithm(PathfindingAlgorithm::Dijkstra);
        a.set_path_postprocessing(PathPostProcessing::CenteredFunnel);
        a.set_path_metadata_flags(1);
        a.set_path_desired_distance(0.5);
        a.set_target_desired_distance(0.25);
        a.set_height(2.0);
        a.set_radius(1.0);
        a.set_path_height_offset(0.5);
        a.set_neighbor_distance(20.0);
        a.set_max_neighbors(4);
        a.set_time_horizon_agents(2.0);
        a.set_time_horizon_obstacles(1.0);
        a.set_max_speed(5.0);
        a.set_path_max_distance(9.0);
        a.set_simplify_path(true);
        a.set_simplify_epsilon(0.1);
        a.set_path_return_max_length(100.0);
        a.set_path_return_max_radius(10.0);
        a.set_path_search_max_polygons(64);
        a.set_path_search_max_distance(30.0);

        assert!(!a.get_avoidance_enabled());
        assert_eq!(a.get_avoidance_mask(), 4);
        assert_eq!(a.get_avoidance_priority(), 2.0);
        assert_eq!(a.get_navigation_layers(), 3);
        assert_eq!(a.get_pathfinding_algorithm(), PathfindingAlgorithm::Dijkstra);
        assert_eq!(a.get_path_postprocessing(), PathPostProcessing::CenteredFunnel);
        assert_eq!(a.get_path_metadata_flags(), 1);
        assert_eq!(a.get_path_desired_distance(), 0.5);
        assert_eq!(a.get_target_desired_distance(), 0.25);
        assert_eq!(a.get_height(), 2.0);
        assert_eq!(a.get_radius(), 1.0);
        assert_eq!(a.get_path_height_offset(), 0.5);
        assert_eq!(a.get_neighbor_distance(), 20.0);
        assert_eq!(a.get_max_neighbors(), 4);
        assert_eq!(a.get_time_horizon_agents(), 2.0);
        assert_eq!(a.get_time_horizon_obstacles(), 1.0);
        assert_eq!(a.get_max_speed(), 5.0);
        assert_eq!(a.get_path_max_distance(), 9.0);
        assert!(a.get_simplify_path());
        assert_eq!(a.get_simplify_epsilon(), 0.1);
        assert_eq!(a.get_path_return_max_length(), 100.0);
        assert_eq!(a.get_path_return_max_radius(), 10.0);
        assert_eq!(a.get_path_search_max_polygons(), 64);
        assert_eq!(a.get_path_search_max_distance(), 30.0);
    }

    #[test]
    fn targets_and_velocities() {
        let mut a = NavigationAgent3DData::new();
        a.set_target_position(Vector3::new(1.0, 0.0, 2.0));
        assert!(a.is_target_position_submitted());
        assert!(!a.is_target_reached());
        assert!(!a.is_navigation_finished());
        assert_eq!(a.get_target_position(), Vector3::new(1.0, 0.0, 2.0));

        a.set_path_index(3);
        assert_eq!(a.get_path_index(), 3);
        a.set_path_result(true, true);
        assert!(!a.is_target_position_submitted());
        assert!(a.is_navigation_finished());
        assert!(a.is_target_reached());
        assert_eq!(a.get_path_index(), 0, "a fresh result restarts at index 0");

        a.set_velocity(Vector3::new(1.0, 0.0, 0.0));
        a.set_velocity_forced(Vector3::new(0.0, 0.0, 1.0));
        assert!(a.is_velocity_submitted());
        assert!(a.is_velocity_forced_submitted());
        let (velocity, forced) = a.consume_velocities();
        assert_eq!(velocity, Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(forced, Vector3::new(0.0, 0.0, 1.0));
        assert!(!a.is_velocity_submitted());
        assert!(!a.is_velocity_forced_submitted());

        a.set_keep_y_velocity(false);
        a.set_stored_y_velocity(2.5);
        assert!(!a.get_keep_y_velocity());
        assert_eq!(a.get_stored_y_velocity(), 2.5);

        a.set_safe_velocity(Vector3::new(0.5, 0.0, 0.5));
        assert_eq!(a.get_safe_velocity(), Vector3::new(0.5, 0.0, 0.5));
    }

    #[test]
    fn debug_options() {
        let mut a = NavigationAgent3DData::new();
        a.set_debug_enabled(true);
        a.set_debug_path_custom_point_size(8.0);
        a.set_debug_use_custom(true);
        a.set_debug_path_custom_color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert!(a.get_debug_enabled());
        assert_eq!(a.get_debug_path_custom_point_size(), 8.0);
        assert!(a.get_debug_use_custom());
        assert_eq!(a.get_debug_path_custom_color(), Color::new(1.0, 0.0, 0.0, 1.0));
    }
}
