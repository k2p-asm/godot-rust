//! `scene/2d/physics/collision_polygon_2d.*` and
//! `scene/3d/physics/collision_polygon_3d.*` (state only).
//!
//! `CollisionPolygon2D`: build mode (`SOLIDS`/`SEGMENTS`, rejecting values past
//! the two), the editor rect `(-10,-10,20,20)`, disabled and one-way settings
//! (margin 1, direction `(0,1)`). `set_polygon` reproduces the engine's AABB
//! recompute: bounds of the points, then either the default 20×20 rect when the
//! polygon is empty or a 30% expansion on each side.
//!
//! `CollisionPolygon3D`: depth 1, the `(-1,-1,-1)+(2,2,2)` AABB, the polygon, the
//! debug colour/fill flags and `disabled`. Its AABB is recomputed from the points
//! extruded by `depth / 2` along Z (`_build_polygon`'s shape is not ported, so
//! this is the node-side extent only).
//!
//! Shape-owner registration and the physics server are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BuildMode {
    Solids,
    Segments,
}

#[derive(Clone, Debug)]
pub struct CollisionPolygon2DData {
    aabb: Rect2,
    build_mode: BuildMode,
    polygon: Vec<Vector2>,
    disabled: bool,
    one_way_collision: bool,
    one_way_collision_margin: f32,
    one_way_collision_direction: Vector2,
}

impl Default for CollisionPolygon2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionPolygon2DData {
    pub fn new() -> Self {
        Self {
            aabb: Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)),
            build_mode: BuildMode::Solids,
            polygon: Vec::new(),
            disabled: false,
            one_way_collision: false,
            one_way_collision_margin: 1.0,
            one_way_collision_direction: Vector2::new(0.0, 1.0),
        }
    }

    /// `ERR_FAIL_INDEX((int)p_mode, 2)`.
    pub fn set_build_mode(&mut self, mode: BuildMode) -> bool {
        if mode == BuildMode::Segments {
            self.build_mode = mode;
            return true;
        }
        if mode == BuildMode::Solids {
            self.build_mode = mode;
            return true;
        }
        false
    }

    pub fn get_build_mode(&self) -> BuildMode {
        self.build_mode
    }

    /// Stores the outline and recomputes the AABB like the engine.
    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
        let mut aabb = Rect2::default();
        for (index, point) in self.polygon.iter().enumerate() {
            if index == 0 {
                aabb = Rect2::new(*point, Vector2::ZERO);
            } else {
                aabb.expand_to(*point);
            }
        }
        self.aabb = if aabb == Rect2::default() {
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        } else {
            let mut expanded = aabb;
            expanded.position -= expanded.size * 0.3;
            expanded.size += expanded.size * 0.6;
            expanded
        };
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }

    pub fn get_aabb(&self) -> Rect2 {
        self.aabb
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_one_way_collision(&mut self, enabled: bool) {
        self.one_way_collision = enabled;
    }

    pub fn is_one_way_collision_enabled(&self) -> bool {
        self.one_way_collision
    }

    pub fn set_one_way_collision_margin(&mut self, margin: f32) {
        self.one_way_collision_margin = margin;
    }

    pub fn get_one_way_collision_margin(&self) -> f32 {
        self.one_way_collision_margin
    }

    pub fn set_one_way_collision_direction(&mut self, direction: Vector2) {
        self.one_way_collision_direction = direction;
    }

    pub fn get_one_way_collision_direction(&self) -> Vector2 {
        self.one_way_collision_direction
    }
}

#[derive(Clone, Debug)]
pub struct CollisionPolygon3DData {
    depth: f32,
    aabb: crate::math::aabb::Aabb,
    polygon: Vec<Vector2>,
    debug_color: Color,
    debug_fill: bool,
    disabled: bool,
}

impl Default for CollisionPolygon3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionPolygon3DData {
    pub fn new() -> Self {
        Self {
            depth: 1.0,
            aabb: crate::math::aabb::Aabb::new(
                crate::math::vector3::Vector3::new(-1.0, -1.0, -1.0),
                crate::math::vector3::Vector3::new(2.0, 2.0, 2.0),
            ),
            polygon: Vec::new(),
            debug_color: Color::new(0.0, 0.0, 0.0, 0.0),
            debug_fill: true,
            disabled: false,
        }
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth;
        self.recompute_aabb();
    }

    pub fn get_depth(&self) -> f32 {
        self.depth
    }

    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
        self.recompute_aabb();
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }

    /// The node-side extent: the 2D bounds extruded by `depth / 2` along Z.
    fn recompute_aabb(&mut self) {
        if self.polygon.is_empty() {
            self.aabb = crate::math::aabb::Aabb::default();
            return;
        }
        let mut min = self.polygon[0];
        let mut max = self.polygon[0];
        for point in &self.polygon {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }
        let half = self.depth * 0.5;
        self.aabb = crate::math::aabb::Aabb::new(
            crate::math::vector3::Vector3::new(min.x, min.y, -half),
            crate::math::vector3::Vector3::new(max.x - min.x, max.y - min.y, self.depth),
        );
    }

    pub fn get_aabb(&self) -> crate::math::aabb::Aabb {
        self.aabb
    }

    pub fn set_debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }

    pub fn get_debug_color(&self) -> Color {
        self.debug_color
    }

    pub fn set_debug_fill_enabled(&mut self, enabled: bool) {
        self.debug_fill = enabled;
    }

    pub fn is_debug_fill_enabled(&self) -> bool {
        self.debug_fill
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn polygon_2d_defaults() {
        let mut p = CollisionPolygon2DData::new();
        assert_eq!(p.get_build_mode(), BuildMode::Solids);
        assert!(p.get_polygon().is_empty());
        assert_eq!(
            p.get_aabb(),
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        );
        assert!(!p.is_disabled());
        assert!(!p.is_one_way_collision_enabled());
        assert_eq!(p.get_one_way_collision_margin(), 1.0);
        assert_eq!(p.get_one_way_collision_direction(), Vector2::new(0.0, 1.0));

        assert!(p.set_build_mode(BuildMode::Segments));
        assert_eq!(p.get_build_mode(), BuildMode::Segments);
        assert!(p.set_build_mode(BuildMode::Solids));
        assert_eq!(BuildMode::Solids as i32, 0);
        assert_eq!(BuildMode::Segments as i32, 1);

        p.set_disabled(true);
        p.set_one_way_collision(true);
        p.set_one_way_collision_margin(2.0);
        p.set_one_way_collision_direction(Vector2::UP);
        assert!(p.is_disabled());
        assert!(p.is_one_way_collision_enabled());
        assert_eq!(p.get_one_way_collision_margin(), 2.0);
        assert_eq!(p.get_one_way_collision_direction(), Vector2::UP);
    }

    #[test]
    fn polygon_2d_aabb_rules() {
        let mut p = CollisionPolygon2DData::new();
        // A square from (0,0) to (4,4), then 30% expansion on each side.
        p.set_polygon(vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
        ]);
        let aabb = p.get_aabb();
        assert!((aabb.position.x + 1.2).abs() < 1e-5);
        assert!((aabb.position.y + 1.2).abs() < 1e-5);
        assert!((aabb.size.x - 6.4).abs() < 1e-5);
        assert!((aabb.size.y - 6.4).abs() < 1e-5);

        // An empty polygon restores the default rect.
        p.set_polygon(Vec::new());
        assert_eq!(
            p.get_aabb(),
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        );
    }

    #[test]
    fn polygon_3d_defaults_and_extent() {
        let mut p = CollisionPolygon3DData::new();
        assert_eq!(p.get_depth(), 1.0);
        assert_eq!(
            p.get_aabb(),
            crate::math::aabb::Aabb::new(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(2.0, 2.0, 2.0)
            )
        );
        assert!(p.get_polygon().is_empty());
        assert_eq!(p.get_debug_color(), Color::new(0.0, 0.0, 0.0, 0.0));
        assert!(p.is_debug_fill_enabled());
        assert!(!p.is_disabled());

        p.set_depth(2.0);
        p.set_polygon(vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(2.0, 3.0),
        ]);
        let aabb = p.get_aabb();
        assert_eq!(aabb.position, Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(aabb.size, Vector3::new(2.0, 3.0, 2.0));

        p.set_debug_color(Color::new(1.0, 0.0, 0.0, 1.0));
        p.set_debug_fill_enabled(false);
        p.set_disabled(true);
        assert_eq!(p.get_debug_color(), Color::new(1.0, 0.0, 0.0, 1.0));
        assert!(!p.is_debug_fill_enabled());
        assert!(p.is_disabled());

        // Clearing the polygon zeroes the extent.
        p.set_polygon(Vec::new());
        assert_eq!(p.get_aabb(), crate::math::aabb::Aabb::default());
    }
}
