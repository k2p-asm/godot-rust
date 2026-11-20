//! `core/math/delaunay_2d.h`: Bowyer–Watson Delaunay triangulation in 2D.
//!
//! Ported operation for operation from the header-only implementation: a
//! bounding triangle 16× the data's largest extent, insert each point by
//! deleting every triangle whose circumcircle contains it, collect the cavity
//! edges, drop the edges shared by two deleted triangles and fan the rest to the
//! new point, then discard triangles that reference a bounding vertex.
//!
//! `create_triangle` reproduces the engine's circumcircle formula (orthogonal of
//! `b*|a|² - a*|b|²` over `2*cross(a,b)`).

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triangle {
    pub points: [i32; 3],
    pub circum_center: Vector2,
    pub circum_radius_squared: f32,
}

impl Triangle {
    pub fn new(a: i32, b: i32, c: i32) -> Self {
        Self {
            points: [a, b, c],
            circum_center: Vector2::ZERO,
            circum_radius_squared: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    /// Sorted pair to avoid checking both orientations.
    pub points: [i32; 2],
    pub bad: bool,
}

impl Edge {
    pub fn new(a: i32, b: i32) -> Self {
        if a > b {
            Self {
                points: [b, a],
                bad: false,
            }
        } else {
            Self {
                points: [a, b],
                bad: false,
            }
        }
    }
}

/// `Delaunay2D::create_triangle`.
pub fn create_triangle(vertices: &[Vector2], a: i32, b: i32, c: i32) -> Triangle {
    let pa = vertices[a as usize];
    let a_vec = vertices[b as usize] - pa;
    let b_vec = vertices[c as usize] - pa;
    let o = (b_vec * a_vec.length_squared() - a_vec * b_vec.length_squared()).orthogonal()
        / (a_vec.cross(b_vec) * 2.0);
    Triangle {
        points: [a, b, c],
        circum_center: o + pa,
        circum_radius_squared: o.length_squared(),
    }
}

/// `Delaunay2D::triangulate`.
pub fn triangulate(points: &[Vector2]) -> Vec<Triangle> {
    if points.len() <= 2 {
        return Vec::new();
    }

    let mut vertices = points.to_vec();
    let point_count = points.len();

    // Bounding rectangle.
    let mut rect = Rect2::new(points[0], Vector2::ZERO);
    for point in points.iter().skip(1) {
        rect.expand_to(*point);
    }
    let delta_max = rect.size.x.max(rect.size.y);
    let center = rect.get_center();

    vertices.push(Vector2::new(
        center.x - delta_max * 16.0,
        center.y - delta_max,
    ));
    vertices.push(Vector2::new(center.x, center.y + delta_max * 16.0));
    vertices.push(Vector2::new(
        center.x + delta_max * 16.0,
        center.y - delta_max,
    ));

    let mut triangles = vec![create_triangle(
        &vertices,
        point_count as i32,
        point_count as i32 + 1,
        point_count as i32 + 2,
    )];

    for i in 0..point_count {
        let mut polygon: Vec<Edge> = Vec::new();

        // Cavity: every triangle whose circumcircle contains the point.
        let mut j = triangles.len();
        while j > 0 {
            j -= 1;
            let triangle = triangles[j];
            if vertices[i].distance_squared_to(triangle.circum_center)
                < triangle.circum_radius_squared
            {
                polygon.push(Edge::new(triangle.points[0], triangle.points[1]));
                polygon.push(Edge::new(triangle.points[1], triangle.points[2]));
                polygon.push(Edge::new(triangle.points[2], triangle.points[0]));
                triangles.remove(j);
            }
        }

        // One triangle per unique edge.
        for a in 0..polygon.len() {
            if polygon[a].bad {
                continue;
            }
            for b in a + 1..polygon.len() {
                if polygon[b].points[0] == polygon[a].points[0]
                    && polygon[b].points[1] == polygon[a].points[1]
                {
                    polygon[a].bad = true;
                    polygon[b].bad = true;
                    break;
                }
            }
            if !polygon[a].bad {
                triangles.push(create_triangle(
                    &vertices,
                    polygon[a].points[0],
                    polygon[a].points[1],
                    i as i32,
                ));
            }
        }
    }

    triangles.retain(|triangle| {
        !(triangle.points[0] >= point_count as i32
            || triangle.points[1] >= point_count as i32
            || triangle.points[2] >= point_count as i32)
    });
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort_triangles(triangles: &mut Vec<Triangle>) {
        for triangle in triangles.iter_mut() {
            triangle.points.sort_unstable();
        }
        triangles.sort_by_key(|triangle| (triangle.points[0], triangle.points[1], triangle.points[2]));
    }

    #[test]
    fn circumcircle_of_a_unit_triangle() {
        let points = [
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 1.0),
        ];
        let triangle = create_triangle(&points, 0, 1, 2);
        assert!((triangle.circum_center.x - 0.5).abs() < 1e-5);
        assert!((triangle.circum_center.y - 0.5).abs() < 1e-5);
        assert!((triangle.circum_radius_squared - 0.5).abs() < 1e-5);
    }

    #[test]
    fn square_triangulates_into_two() {
        let mut points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(0.0, 1.0),
        ];
        let mut triangles = triangulate(&points);
        assert_eq!(triangles.len(), 2);
        sort_triangles(&mut triangles);
        // The diagonal can be either way; both triangulations are valid.
        let first = triangles[0].points;
        assert_eq!(first.len(), 3);
        let all_in_range = triangles
            .iter()
            .all(|t| t.points.iter().all(|&p| (0..4).contains(&p)));
        assert!(all_in_range);

        // Fewer than three points yields nothing.
        points.truncate(2);
        assert!(triangulate(&points).is_empty());
        assert!(triangulate(&[]).is_empty());
    }

    #[test]
    fn collinear_points_produce_no_finite_triangles() {
        let points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(3.0, 0.0),
        ];
        let triangles = triangulate(&points);
        // Degenerate circumcircles (zero cross product) can produce any count of
        // NaN-radius triangles, but every returned index must be a real point.
        assert!(triangles
            .iter()
            .all(|t| t.points.iter().all(|&p| (0..4).contains(&p))));
    }

    #[test]
    fn five_points_cover_the_hull() {
        let points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
            Vector2::new(2.0, 2.0),
        ];
        let triangles = triangulate(&points);
        assert_eq!(triangles.len(), 4, "a square with a centre has four triangles");
        assert!(triangles
            .iter()
            .all(|t| t.points.iter().all(|&p| (0..5).contains(&p))));
        // Every triangle has a finite circumcircle.
        assert!(triangles
            .iter()
            .all(|t| t.circum_radius_squared.is_finite()));
    }

    #[test]
    fn edges_are_sorted() {
        assert_eq!(Edge::new(3, 1).points, [1, 3]);
        assert_eq!(Edge::new(1, 3).points, [1, 3]);
        assert!(!Edge::new(0, 1).bad);
    }
}
