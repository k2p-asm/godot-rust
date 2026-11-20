//! `scene/2d/navigation/navigation_agent_2d.*`: pathfinding/avoidance state.
//!
//! Defaults differ from the 3D agent in the interesting places: avoidance is
//! **off**, the path/target desired distances are 20/10, the radius 10, the
//! neighbour distance 500, the max speed 100 and the path max distance 100; the
//! rest matches the 3D defaults (A*, corridor funnel, all metadata, ten
//! neighbours, horizons 1/0, simplification off, 4096 polygons). The debug block
//! adds a line width of -1.
//!
//! Ported: every setting, target submission/reached/finished bookkeeping, the
//! path index, safe/velocity/forced velocity latching with `consume_velocities`
//! and the debug options. The async query, avoidance simulation and map lookups
//! are not ported.

use crate::math::color::Color;
use crate::math::vector2::Vector2;

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

pub const PATH_METADATA_INCLUDE_ALL: u32 = 7;

#[derive(Clone, Debug)]
pub struct NavigationAgent2DData {
    avoidance_enabled: bool,
    avoidance_layers: u32,
    avoidance_mask: u32,
    avoidance_priority: f32,
    navigation_layers: u32,
    pathfinding_algorithm: PathfindingAlgorithm,
    path_postprocessing: PathPostProcessing,
    path_metadata_flags: u32,
    path_desired_distance: f32,
    target_desired_distance: f32,
    radius: f32,
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
    target_position: Vector2,
    navigation_path_index: i32,
    safe_velocity: Vector2,
    velocity: Vector2,
    velocity_submitted: bool,
    velocity_forced: Vector2,
    velocity_forced_submitted: bool,
    target_position_submitted: bool,
    target_reached: bool,
    navigation_finished: bool,
    last_waypoint_reached: bool,
    debug_enabled: bool,
    debug_path_custom_point_size: f32,
    debug_path_custom_line_width: f32,
    debug_use_custom: bool,
    debug_path_custom_color: Color,
}

impl Default for NavigationAgent2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationAgent2DData {
    pub fn new() -> Self {
        Self {
            avoidance_enabled: false,
            avoidance_layers: 1,
            avoidance_mask: 1,
            avoidance_priority: 1.0,
            navigation_layers: 1,
            pathfinding_algorithm: PathfindingAlgorithm::Astar,
            path_postprocessing: PathPostProcessing::CorridorFunnel,
            path_metadata_flags: PATH_METADATA_INCLUDE_ALL,
            path_desired_distance: 20.0,
            target_desired_distance: 10.0,
            radius: 10.0,
            neighbor_distance: 500.0,
            max_neighbors: 10,
            time_horizon_agents: 1.0,
            time_horizon_obstacles: 0.0,
            max_speed: 100.0,
            path_max_distance: 100.0,
            simplify_path: false,
            simplify_epsilon: 0.0,
            path_return_max_length: 0.0,
            path_return_max_radius: 0.0,
            path_search_max_polygons: 4096,
            path_search_max_distance: 0.0,
            target_position: Vector2::ZERO,
            navigation_path_index: 0,
            safe_velocity: Vector2::ZERO,
            velocity: Vector2::ZERO,
            velocity_submitted: false,
            velocity_forced: Vector2::ZERO,
            velocity_forced_submitted: false,
            target_position_submitted: false,
            target_reached: false,
            navigation_finished: true,
            last_waypoint_reached: false,
            debug_enabled: false,
            debug_path_custom_point_size: 4.0,
            debug_path_custom_line_width: -1.0,
            debug_use_custom: false,
            debug_path_custom_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn set_avoidance_enabled(&mut self, enabled: bool) {
        self.avoidance_enabled = enabled;
    }

    pub fn get_avoidance_enabled(&self) -> bool {
        self.avoidance_enabled
    }

    pub fn set_avoidance_layers(&mut self, layers: u32) {
        self.avoidance_layers = layers;
    }

    pub fn get_avoidance_layers(&self) -> u32 {
        self.avoidance_layers
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

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
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

    pub fn set_target_position(&mut self, position: Vector2) {
        self.target_position = position;
        self.target_position_submitted = true;
        self.target_reached = false;
        self.navigation_finished = false;
        self.last_waypoint_reached = false;
    }

    pub fn get_target_position(&self) -> Vector2 {
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

    pub fn set_safe_velocity(&mut self, velocity: Vector2) {
        self.safe_velocity = velocity;
    }

    pub fn get_safe_velocity(&self) -> Vector2 {
        self.safe_velocity
    }

    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
        self.velocity_submitted = true;
    }

    pub fn set_velocity_forced(&mut self, velocity: Vector2) {
        self.velocity_forced = velocity;
        self.velocity_forced_submitted = true;
    }

    pub fn is_velocity_submitted(&self) -> bool {
        self.velocity_submitted
    }

    pub fn is_velocity_forced_submitted(&self) -> bool {
        self.velocity_forced_submitted
    }

    pub fn consume_velocities(&mut self) -> (Vector2, Vector2) {
        self.velocity_submitted = false;
        self.velocity_forced_submitted = false;
        (self.velocity, self.velocity_forced)
    }

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

    pub fn set_debug_path_custom_line_width(&mut self, width: f32) {
        self.debug_path_custom_line_width = width;
    }

    pub fn get_debug_path_custom_line_width(&self) -> f32 {
        self.debug_path_custom_line_width
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
        let a = NavigationAgent2DData::new();
        assert!(!a.get_avoidance_enabled(), "2D avoidance starts off");
        assert_eq!(a.get_avoidance_layers(), 1);
        assert_eq!(a.get_avoidance_mask(), 1);
        assert_eq!(a.get_avoidance_priority(), 1.0);
        assert_eq!(a.get_navigation_layers(), 1);
        assert_eq!(a.get_pathfinding_algorithm(), PathfindingAlgorithm::Astar);
        assert_eq!(a.get_path_postprocessing(), PathPostProcessing::CorridorFunnel);
        assert_eq!(a.get_path_metadata_flags(), PATH_METADATA_INCLUDE_ALL);
        assert_eq!(a.get_path_desired_distance(), 20.0);
        assert_eq!(a.get_target_desired_distance(), 10.0);
        assert_eq!(a.get_radius(), 10.0);
        assert_eq!(a.get_neighbor_distance(), 500.0);
        assert_eq!(a.get_max_neighbors(), 10);
        assert_eq!(a.get_time_horizon_agents(), 1.0);
        assert_eq!(a.get_time_horizon_obstacles(), 0.0);
        assert_eq!(a.get_max_speed(), 100.0);
        assert_eq!(a.get_path_max_distance(), 100.0);
        assert!(!a.get_simplify_path());
        assert_eq!(a.get_simplify_epsilon(), 0.0);
        assert_eq!(a.get_path_return_max_length(), 0.0);
        assert_eq!(a.get_path_return_max_radius(), 0.0);
        assert_eq!(a.get_path_search_max_polygons(), 4096);
        assert_eq!(a.get_path_search_max_distance(), 0.0);
        assert_eq!(a.get_target_position(), Vector2::ZERO);
        assert!(a.is_navigation_finished());
        assert_eq!(a.get_debug_path_custom_point_size(), 4.0);
        assert_eq!(a.get_debug_path_custom_line_width(), -1.0);
        assert!(!a.get_debug_enabled());
        assert!(!a.get_debug_use_custom());
    }

    #[test]
    fn settings_and_results() {
        let mut a = NavigationAgent2DData::new();
        a.set_avoidance_enabled(true);
        a.set_avoidance_layers(2);
        a.set_avoidance_mask(4);
        a.set_path_desired_distance(1.0);
        a.set_target_desired_distance(2.0);
        a.set_radius(3.0);
        a.set_max_speed(50.0);
        a.set_path_max_distance(7.0);
        a.set_pathfinding_algorithm(PathfindingAlgorithm::Dijkstra);
        a.set_path_postprocessing(PathPostProcessing::None);
        a.set_path_metadata_flags(1);
        a.set_simplify_path(true);
        a.set_simplify_epsilon(0.5);
        a.set_path_return_max_length(12.0);
        a.set_path_return_max_radius(3.0);
        a.set_path_search_max_polygons(8);
        a.set_path_search_max_distance(9.0);

        assert!(a.get_avoidance_enabled());
        assert_eq!(a.get_avoidance_layers(), 2);
        assert_eq!(a.get_avoidance_mask(), 4);
        assert_eq!(a.get_path_desired_distance(), 1.0);
        assert_eq!(a.get_target_desired_distance(), 2.0);
        assert_eq!(a.get_radius(), 3.0);
        assert_eq!(a.get_max_speed(), 50.0);
        assert_eq!(a.get_path_max_distance(), 7.0);
        assert_eq!(a.get_pathfinding_algorithm(), PathfindingAlgorithm::Dijkstra);
        assert_eq!(a.get_path_postprocessing(), PathPostProcessing::None);
        assert_eq!(a.get_path_metadata_flags(), 1);
        assert!(a.get_simplify_path());
        assert_eq!(a.get_simplify_epsilon(), 0.5);
        assert_eq!(a.get_path_return_max_length(), 12.0);
        assert_eq!(a.get_path_return_max_radius(), 3.0);
        assert_eq!(a.get_path_search_max_polygons(), 8);
        assert_eq!(a.get_path_search_max_distance(), 9.0);

        a.set_target_position(Vector2::new(3.0, 4.0));
        assert!(a.is_target_position_submitted());
        assert!(!a.is_navigation_finished());
        assert_eq!(a.get_target_position(), Vector2::new(3.0, 4.0));
        a.set_path_index(2);
        a.set_path_result(true, true);
        assert!(a.is_navigation_finished());
        assert!(a.is_target_reached());
        assert_eq!(a.get_path_index(), 0);
        assert!(!a.is_target_position_submitted());

        a.set_velocity(Vector2::new(1.0, 0.0));
        a.set_velocity_forced(Vector2::new(0.0, 1.0));
        assert!(a.is_velocity_submitted() && a.is_velocity_forced_submitted());
        let (velocity, forced) = a.consume_velocities();
        assert_eq!(velocity, Vector2::new(1.0, 0.0));
        assert_eq!(forced, Vector2::new(0.0, 1.0));
        assert!(!a.is_velocity_submitted() && !a.is_velocity_forced_submitted());

        a.set_debug_enabled(true);
        a.set_debug_path_custom_line_width(2.0);
        assert!(a.get_debug_enabled());
        assert_eq!(a.get_debug_path_custom_line_width(), 2.0);
        assert_eq!(PathPostProcessing::None as i32, 4);
    }
}
