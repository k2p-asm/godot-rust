//! Port of the triangle intersection tests from `core/math/geometry_3d.h`.
//!
//! Ported: `ray_intersects_triangle`, `segment_intersects_triangle`.
//! Pending: sphere/cylinder/convex tests, `segment_intersects_circle` and the
//! rest of `Geometry3D` (that file is mostly mesh/triangulation helpers).

use crate::math::funcs;
use crate::math::vector3::Vector3;

/// Möller-Trumbore ray/triangle test, exactly as the engine implements it
/// (including the `t > 0.00001` floor, which differs from the segment variant).
pub fn ray_intersects_triangle(
    from: Vector3,
    dir: Vector3,
    v0: Vector3,
    v1: Vector3,
    v2: Vector3,
) -> Option<Vector3> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let h = dir.cross(e2);
    let a = e1.dot(h);
    if funcs::is_zero_approx(a) {
        return None; // Parallel.
    }

    let f = 1.0 / a;

    let s = from - v0;
    let u = f * s.dot(h);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(e1);
    let v = f * dir.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * e2.dot(q);
    if t > 0.00001 {
        Some(from + dir * t)
    } else {
        // Line intersection behind the ray origin.
        None
    }
}

pub fn segment_intersects_triangle(
    from: Vector3,
    to: Vector3,
    v0: Vector3,
    v1: Vector3,
    v2: Vector3,
) -> Option<Vector3> {
    let rel = to - from;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let h = rel.cross(e2);
    let a = e1.dot(h);
    if funcs::is_zero_approx(a) {
        return None;
    }

    let f = 1.0 / a;

    let s = from - v0;
    let u = f * s.dot(h);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(e1);
    let v = f * rel.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * e2.dot(q);
    if t > funcs::CMP_EPSILON as f32 && t <= 1.0 {
        Some(from + rel * t)
    } else {
        None
    }
}

/// Closest point on a segment to `p_point` (`get_closest_point_to_segment`).
pub fn get_closest_point_to_segment(p_point: Vector3, a: Vector3, b: Vector3) -> Vector3 {
    let p = p_point - a;
    let n = b - a;
    let l2 = n.length_squared();
    if l2 < 1e-20 {
        return a; // Degenerate segment.
    }

    let d = n.dot(p) / l2;
    if d <= 0.0 {
        a
    } else if d >= 1.0 {
        b
    } else {
        a + n * d
    }
}

/// Same but without clamping to the segment.
pub fn get_closest_point_to_segment_uncapped(p_point: Vector3, a: Vector3, b: Vector3) -> Vector3 {
    let p = p_point - a;
    let n = b - a;
    let l2 = n.length_squared();
    if l2 < 1e-20 {
        return a;
    }
    let d = n.dot(p) / l2;
    a + n * d
}

/// Barycentric coordinates of `p_pos` in the triangle; zero vector when the
/// triangle is degenerate.
pub fn triangle_get_barycentric_coords(
    a: Vector3,
    b: Vector3,
    c: Vector3,
    p_pos: Vector3,
) -> Vector3 {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p_pos - a;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;
    if denom == 0.0 {
        return Vector3::ZERO;
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    Vector3::new(u, v, w)
}
/// Segment vs sphere. Returns `(point, normal)` where `normal` points away from
/// the sphere center.
pub fn segment_intersects_sphere(
    from: Vector3,
    to: Vector3,
    sphere_pos: Vector3,
    sphere_radius: f32,
) -> Option<(Vector3, Vector3)> {
    let rel_pos = sphere_pos - from;
    let rel = to - from;
    let rel_l = rel.length();
    if rel_l < funcs::CMP_EPSILON as f32 {
        return None; // Degenerate segment.
    }
    let normal = rel / rel_l;

    let sphere_d = normal.dot(rel_pos);
    let ray_distance = rel_pos.distance_to(normal * sphere_d);

    if ray_distance >= sphere_radius {
        return None;
    }

    let inters_d2 = sphere_radius * sphere_radius - ray_distance * ray_distance;
    let mut inters_d = sphere_d;

    if inters_d2 >= funcs::CMP_EPSILON as f32 {
        inters_d -= inters_d2.sqrt();
    }

    // Check that the hit is inside the segment.
    if inters_d < 0.0 || inters_d > rel_l {
        return None;
    }

    let result = from + normal * inters_d;
    Some((result, (result - sphere_pos).normalized()))
}

/// Segment vs capped cylinder aligned to `cylinder_axis` (0 = x, 1 = y, 2 = z),
/// centered at the origin. Returns `(point, normal)`.
pub fn segment_intersects_cylinder(
    from: Vector3,
    to: Vector3,
    height: f32,
    radius: f32,
    cylinder_axis: usize,
) -> Option<(Vector3, Vector3)> {
    let rel = to - from;
    let rel_l = rel.length();
    if rel_l < funcs::CMP_EPSILON as f32 {
        return None;
    }
    if cylinder_axis > 2 {
        return None;
    }

    let mut axis_vec = Vector3::ZERO;
    axis_vec[cylinder_axis] = 1.0;

    // Parallel check.
    let normal = rel / rel_l;
    let crs = normal.cross(axis_vec);
    let crs_l = crs.length();

    let axis_dir = if crs_l < funcs::CMP_EPSILON as f32 {
        let mut side_axis = Vector3::ZERO;
        side_axis[(cylinder_axis + 1) % 3] = 1.0;
        side_axis
    } else {
        crs / crs_l
    };

    let dist = axis_dir.dot(from);
    if dist >= radius {
        return None; // Too far away.
    }

    // Convert to 2D.
    let w2 = radius * radius - dist * dist;
    if w2 < funcs::CMP_EPSILON as f32 {
        return None; // Avoid numerical error.
    }
    let size = [w2.sqrt(), height * 0.5];

    let side_dir = axis_dir.cross(axis_vec).normalized();

    let from2d = [side_dir.dot(from), from[cylinder_axis]];
    let to2d = [side_dir.dot(to), to[cylinder_axis]];

    let mut min = 0.0f32;
    let mut max = 1.0f32;
    let mut axis = usize::MAX;

    for i in 0..2 {
        let seg_from = from2d[i];
        let seg_to = to2d[i];
        let box_begin = -size[i];
        let box_end = size[i];
        let (cmin, cmax);

        if seg_from < seg_to {
            if seg_from > box_end || seg_to < box_begin {
                return None;
            }
            let length = seg_to - seg_from;
            cmin = if seg_from < box_begin {
                (box_begin - seg_from) / length
            } else {
                0.0
            };
            cmax = if seg_to > box_end {
                (box_end - seg_from) / length
            } else {
                1.0
            };
        } else {
            if seg_to > box_end || seg_from < box_begin {
                return None;
            }
            let length = seg_to - seg_from;
            cmin = if seg_from > box_end {
                (box_end - seg_from) / length
            } else {
                0.0
            };
            cmax = if seg_to < box_begin {
                (box_begin - seg_from) / length
            } else {
                1.0
            };
        }

        if cmin > min {
            min = cmin;
            axis = i;
        }
        if cmax < max {
            max = cmax;
        }
        if max < min {
            return None;
        }
    }

    // Back to 3D.
    let result = from + rel * min;
    let mut res_normal = result;

    if axis == 0 {
        res_normal[cylinder_axis] = 0.0;
    } else {
        let mut axis_side = (cylinder_axis + 1) % 3;
        res_normal[axis_side] = 0.0;
        axis_side = (axis_side + 1) % 3;
        res_normal[axis_side] = 0.0;
    }

    res_normal.normalize();
    Some((result, res_normal))
}
/// Segment vs convex volume defined by its inward planes
/// (`segment_intersects_convex`). Returns `(point, normal of the entry plane)`.
pub fn segment_intersects_convex(
    from: Vector3,
    to: Vector3,
    planes: &[crate::math::plane::Plane],
) -> Option<(Vector3, Vector3)> {
    let mut min = -1e20f32;
    let mut max = 1e20f32;

    let rel = to - from;
    let rel_l = rel.length();
    if rel_l < funcs::CMP_EPSILON as f32 {
        return None;
    }

    let dir = rel / rel_l;
    let mut min_index: Option<usize> = None;

    for (i, p) in planes.iter().enumerate() {
        let den = p.normal.dot(dir);

        if den.abs() <= funcs::CMP_EPSILON as f32 {
            if p.is_point_over(from) {
                return None; // Separating plane.
            }
            continue; // Ignore parallel plane.
        }

        let dist = -p.distance_to(from) / den;

        if den > 0.0 {
            // Backwards facing plane.
            if dist < max {
                max = dist;
            }
        } else {
            // Front facing plane.
            if dist > min {
                min = dist;
                min_index = Some(i);
            }
        }
    }

    if max <= min || min < 0.0 || min > rel_l || min_index.is_none() {
        return None;
    }

    let idx = min_index.unwrap();
    Some((from + dir * min, planes[idx].normal))
}
/// `point_in_projected_triangle`: three half-space tests against the triangle's
/// face normal (points behind are rejected).
pub fn point_in_projected_triangle(
    p_point: Vector3,
    v1: Vector3,
    v2: Vector3,
    v3: Vector3,
) -> bool {
    let face_n = (v1 - v3).cross(v1 - v2);

    let n1 = (p_point - v3).cross(p_point - v2);
    if face_n.dot(n1) < 0.0 {
        return false;
    }

    let n2 = (v1 - v3).cross(v1 - p_point);
    if face_n.dot(n2) < 0.0 {
        return false;
    }

    let n3 = (v1 - p_point).cross(v1 - v2);
    if face_n.dot(n3) < 0.0 {
        return false;
    }

    true
}
/// Triangle vs sphere contact test, returning `(triangle_contact, sphere_contact)`.
/// Mirrors the engine: plane rejection, inside-triangle case, then the edge
/// capsule/endpoint cases (with the same early `break`).
pub fn triangle_sphere_intersection_test(
    a: Vector3,
    b: Vector3,
    c: Vector3,
    normal: Vector3,
    sphere_pos: Vector3,
    sphere_radius: f32,
) -> Option<(Vector3, Vector3)> {
    // 1st: plane distance.
    let d = normal.dot(sphere_pos) - normal.dot(a);
    if d > sphere_radius || d < -sphere_radius {
        return None;
    }

    let contact = sphere_pos - normal * d;

    // 2nd: contact point inside the triangle.
    if point_in_projected_triangle(contact, a, b, c) {
        return Some((contact, sphere_pos - normal * sphere_radius));
    }

    // 3rd: inside the edge cylinders.
    let verts = [a, b, c, a];

    for i in 0..3 {
        let n1 = verts[i] - verts[i + 1];
        let n2 = sphere_pos - verts[i + 1];

        let mut axis = n1.cross(n2).cross(n1);
        axis.normalize();

        let ad = axis.dot(n2);
        if ad.abs() > sphere_radius {
            // Too far from this edge.
            continue;
        }

        // 4th: inside the edge points.
        let sphere_at = n1.dot(n2);
        if sphere_at >= 0.0 && sphere_at < n1.dot(n1) {
            let tri = sphere_pos - axis * axis.dot(n2);
            let sph = sphere_pos - axis * sphere_radius;
            return Some((tri, sph));
        }

        let r2 = sphere_radius * sphere_radius;

        if n2.length_squared() < r2 {
            let n = (sphere_pos - verts[i + 1]).normalized();
            return Some((verts[i + 1], sphere_pos - n * sphere_radius));
        }

        if n2.distance_squared_to(n1) < r2 {
            let n = (sphere_pos - verts[i]).normalized();
            return Some((verts[i], sphere_pos - n * sphere_radius));
        }

        break; // Pointless to continue, as in the original.
    }

    None
}
const LOC_INSIDE: i8 = 1;
const LOC_BOUNDARY: i8 = 0;
const LOC_OUTSIDE: i8 = -1;

/// Sutherland-Hodgman clip of a polygon against one plane
/// (`Geometry3D::clip_polygon`). Vertices within
/// `CMP_POINT_IN_PLANE_EPSILON` of the plane count as inside.
pub fn clip_polygon(polygon: &[Vector3], plane: &crate::math::plane::Plane) -> Vec<Vector3> {
    if polygon.is_empty() {
        return Vec::new();
    }

    let eps = funcs::CMP_POINT_IN_PLANE_EPSILON as f32;
    let mut location = Vec::with_capacity(polygon.len());
    let mut inside_count = 0;
    let mut outside_count = 0;

    for p in polygon {
        let dist = plane.distance_to(*p);
        if dist < -eps {
            location.push(LOC_INSIDE);
            inside_count += 1;
        } else if dist > eps {
            location.push(LOC_OUTSIDE);
            outside_count += 1;
        } else {
            location.push(LOC_BOUNDARY);
        }
    }

    if outside_count == 0 {
        return polygon.to_vec(); // No changes.
    } else if inside_count == 0 {
        return Vec::new(); // Fully outside.
    }

    let mut previous = polygon.len() - 1;
    let mut clipped = Vec::new();

    for index in 0..polygon.len() {
        let loc = location[index];
        if loc == LOC_OUTSIDE {
            if location[previous] == LOC_INSIDE {
                let v1 = polygon[previous];
                let v2 = polygon[index];
                let segment = v1 - v2;
                let den = plane.normal.dot(segment);
                let mut dist = plane.distance_to(v1) / den;
                dist = -dist;
                clipped.push(v1 + segment * dist);
            }
        } else {
            let v1 = polygon[index];
            if loc == LOC_INSIDE && location[previous] == LOC_OUTSIDE {
                let v2 = polygon[previous];
                let segment = v1 - v2;
                let den = plane.normal.dot(segment);
                let mut dist = plane.distance_to(v1) / den;
                dist = -dist;
                clipped.push(v1 + segment * dist);
            }
            clipped.push(v1);
        }

        previous = index;
    }

    clipped
}
/// Closest points between two segments (Eberly's algorithm), as
/// `Geometry3D::get_closest_points_between_segments`.
pub fn get_closest_points_between_segments(
    p0: Vector3,
    p1: Vector3,
    q0: Vector3,
    q1: Vector3,
) -> (Vector3, Vector3) {
    let p = p1 - p0;
    let q = q1 - q0;
    let r = p0 - q0;

    let a = p.dot(p);
    let b = p.dot(q);
    let c = q.dot(q);
    let d = p.dot(r);
    let e = q.dot(r);

    let mut s = 0.0f32;
    let mut t = 0.0f32;

    let det = a * c - b * b;
    if det > funcs::CMP_EPSILON as f32 {
        // Non-parallel segments.
        let bte = b * e;
        let ctd = c * d;

        if bte <= ctd {
            // s <= 0.
            if e <= 0.0 {
                // t <= 0.
                s = if -d >= a {
                    1.0
                } else if -d > 0.0 {
                    -d / a
                } else {
                    0.0
                };
                t = 0.0;
            } else if e < c {
                // 0 < t < 1.
                s = 0.0;
                t = e / c;
            } else {
                // t >= 1.
                s = if b - d >= a {
                    1.0
                } else if b - d > 0.0 {
                    (b - d) / a
                } else {
                    0.0
                };
                t = 1.0;
            }
        } else {
            // s > 0.
            s = bte - ctd;
            if s >= det {
                // s >= 1.
                if b + e <= 0.0 {
                    // t <= 0.
                    s = if -d <= 0.0 {
                        0.0
                    } else if -d < a {
                        -d / a
                    } else {
                        1.0
                    };
                    t = 0.0;
                } else if b + e < c {
                    // 0 < t < 1.
                    s = 1.0;
                    t = (b + e) / c;
                } else {
                    // t >= 1.
                    s = if b - d <= 0.0 {
                        0.0
                    } else if b - d < a {
                        (b - d) / a
                    } else {
                        1.0
                    };
                    t = 1.0;
                }
            } else {
                // 0 < s < 1.
                let ate = a * e;
                let btd = b * d;

                if ate <= btd {
                    // t <= 0.
                    s = if -d <= 0.0 {
                        0.0
                    } else if -d >= a {
                        1.0
                    } else {
                        -d / a
                    };
                    t = 0.0;
                } else {
                    // t > 0.
                    t = ate - btd;
                    if t >= det {
                        // t >= 1.
                        s = if b - d <= 0.0 {
                            0.0
                        } else if b - d >= a {
                            1.0
                        } else {
                            (b - d) / a
                        };
                        t = 1.0;
                    } else {
                        // 0 < t < 1.
                        s /= det;
                        t /= det;
                    }
                }
            }
        }
    } else {
        // Parallel segments.
        if e <= 0.0 {
            s = if -d <= 0.0 {
                0.0
            } else if -d >= a {
                1.0
            } else {
                -d / a
            };
            t = 0.0;
        } else if e >= c {
            s = if b - d <= 0.0 {
                0.0
            } else if b - d >= a {
                1.0
            } else {
                (b - d) / a
            };
            t = 1.0;
        } else {
            s = 0.0;
            t = e / c;
        }
    }

    ((1.0 - s) * p0 + s * p1, (1.0 - t) * q0 + t * q1)
}

/// Distance between the closest points of two segments.
pub fn get_closest_distance_between_segments(
    p0: Vector3,
    p1: Vector3,
    q0: Vector3,
    q1: Vector3,
) -> f32 {
    let (ps, qt) = get_closest_points_between_segments(p0, p1, q0, q1);
    ps.distance_to(qt)
}
/// The six planes of an axis-aligned box (`build_box_planes`), normals pointing
/// outwards and `d` equal to the half extent on that axis.
pub fn build_box_planes(extents: Vector3) -> [crate::math::plane::Plane; 6] {
    use crate::math::plane::Plane;
    [
        Plane::new(1.0, 0.0, 0.0, extents.x),
        Plane::new(-1.0, 0.0, 0.0, extents.x),
        Plane::new(0.0, 1.0, 0.0, extents.y),
        Plane::new(0.0, -1.0, 0.0, extents.y),
        Plane::new(0.0, 0.0, 1.0, extents.z),
        Plane::new(0.0, 0.0, -1.0, extents.z),
    ]
}

/// Planes of a cylinder around `axis`: one per side facet plus the two caps
/// (`build_cylinder_planes`). `height` is the *full* height; caps sit at `h/2`.
pub fn build_cylinder_planes(
    radius: f32,
    height: f32,
    sides: i32,
    axis: usize,
) -> Vec<crate::math::plane::Plane> {
    use crate::math::plane::Plane;
    if axis > 2 || sides <= 0 {
        return Vec::new();
    }

    let step = funcs::TAU / sides as f64;
    let mut planes = Vec::with_capacity(sides as usize);

    for i in 0..sides {
        let mut normal = Vector3::ZERO;
        normal[(axis + 1) % 3] = (i as f64 * step).cos() as f32;
        normal[(axis + 2) % 3] = (i as f64 * step).sin() as f32;
        planes.push(Plane::new(normal.x, normal.y, normal.z, radius));
    }

    // Caps: the engine appends +axis and -axis at half the height.
    let mut axis_vec = Vector3::ZERO;
    axis_vec[axis] = 1.0;
    planes.push(Plane::new(axis_vec.x, axis_vec.y, axis_vec.z, height * 0.5));
    planes.push(Plane::new(-axis_vec.x, -axis_vec.y, -axis_vec.z, height * 0.5));

    planes
}
/// Planes of a sphere (`build_sphere_planes`): `lons` around the axis and, for
/// each, `2 * lats` tilted pairs. Count = `lons * (1 + 2 * lats)`.
pub fn build_sphere_planes(
    radius: f32,
    lats: i32,
    lons: i32,
    axis: usize,
) -> Vec<crate::math::plane::Plane> {
    use crate::math::plane::Plane;
    if axis > 2 || lons <= 0 {
        return Vec::new();
    }

    let mut planes = Vec::new();

    let mut axis_vec = Vector3::ZERO;
    axis_vec[axis] = 1.0;

    let mut axis_neg = Vector3::ZERO;
    axis_neg[(axis + 1) % 3] = 1.0;
    axis_neg[(axis + 2) % 3] = 1.0;
    axis_neg[axis] = -1.0;

    let lon_step = funcs::TAU / lons as f64;
    for i in 0..lons {
        let mut normal = Vector3::ZERO;
        normal[(axis + 1) % 3] = (i as f64 * lon_step).cos() as f32;
        normal[(axis + 2) % 3] = (i as f64 * lon_step).sin() as f32;

        planes.push(Plane::new(normal.x, normal.y, normal.z, radius));

        for j in 1..=lats {
            let t = j as f32 / lats as f32;
            let plane_normal = {
                let mut l = normal.lerp(axis_vec, t);
                l.normalize();
                l
            };
            planes.push(Plane::new(plane_normal.x, plane_normal.y, plane_normal.z, radius));
            let neg = plane_normal * axis_neg;
            planes.push(Plane::new(neg.x, neg.y, neg.z, radius));
        }
    }

    planes
}
/// Planes of a capsule (`build_capsule_planes`): sides plus `2 * lats` tilted
/// planes at each cap, so the count is `sides * (1 + 2 * lats)`.
pub fn build_capsule_planes(
    radius: f32,
    height: f32,
    sides: i32,
    lats: i32,
    axis: usize,
) -> Vec<crate::math::plane::Plane> {
    use crate::math::plane::Plane;
    if axis > 2 || sides <= 0 {
        return Vec::new();
    }

    let mut planes = Vec::new();

    let mut axis_vec = Vector3::ZERO;
    axis_vec[axis] = 1.0;

    let mut axis_neg = Vector3::ZERO;
    axis_neg[(axis + 1) % 3] = 1.0;
    axis_neg[(axis + 2) % 3] = 1.0;
    axis_neg[axis] = -1.0;

    let sides_step = funcs::TAU / sides as f64;
    for i in 0..sides {
        let mut normal = Vector3::ZERO;
        normal[(axis + 1) % 3] = (i as f64 * sides_step).cos() as f32;
        normal[(axis + 2) % 3] = (i as f64 * sides_step).sin() as f32;

        planes.push(Plane::new(normal.x, normal.y, normal.z, radius));

        for j in 1..=lats {
            let t = j as f32 / lats as f32;
            let plane_normal = {
                let mut l = normal.lerp(axis_vec, t);
                l.normalize();
                l
            };
            let position = axis_vec * (height * 0.5) + plane_normal * radius;
            planes.push(Plane::from_normal_point(plane_normal, position));
            planes.push(Plane::from_normal_point(plane_normal * axis_neg, position * axis_neg));
        }
    }

    planes
}

/// Points of the convex shape defined by `planes`
/// (`compute_convex_mesh_points`): every triple that intersects, kept only when
/// no other plane excludes it.
pub fn compute_convex_mesh_points(planes: &[crate::math::plane::Plane]) -> Vec<Vector3> {
    let n = planes.len();
    let mut points = Vec::new();

    let mut i = n as i64 - 1;
    while i >= 0 {
        let mut j = i - 1;
        while j >= 0 {
            let mut k = j - 1;
            while k >= 0 {
                if let Some(p) = planes[i as usize].intersect_3(&planes[j as usize], &planes[k as usize]) {
                    let mut excluded = false;
                    for (idx, plane) in planes.iter().enumerate() {
                        if idx != i as usize && idx != j as usize && idx != k as usize {
                            let dp = plane.normal.dot(p);
                            if dp - plane.d > funcs::CMP_EPSILON as f32 {
                                excluded = true;
                                break;
                            }
                        }
                    }
                    if !excluded {
                        points.push(p);
                    }
                }
                k -= 1;
            }
            j -= 1;
        }
        i -= 1;
    }

    points
}
/// `Geometry3D::tetrahedralize_delaunay`: tetrahedralizes `points` and returns
/// the flattened index list (4 per tetrahedron).
pub fn tetrahedralize_delaunay(points: &[Vector3]) -> Vec<i32> {
    let tets = crate::math::delaunay_3d::tetrahedralize(points);
    crate::math::delaunay_3d::tetrahedralize_indices(&tets)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri() -> (Vector3, Vector3, Vector3) {
        (
            Vector3::new(-1.0, -1.0, 0.0),
            Vector3::new(1.0, -1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
    }

    #[test]
    fn ray_hits_triangle() {
        let (v0, v1, v2) = tri();
        let hit = ray_intersects_triangle(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 1.0),
            v0,
            v1,
            v2,
        )
        .unwrap();
        assert!(hit.is_equal_approx(Vector3::new(0.0, 0.0, 0.0)), "{hit:?}");

        // Ray pointing away never hits.
        assert!(ray_intersects_triangle(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, -1.0),
            v0,
            v1,
            v2
        )
        .is_none());

        // Outside the triangle's footprint.
        assert!(ray_intersects_triangle(
            Vector3::new(5.0, 5.0, -5.0),
            Vector3::new(0.0, 0.0, 1.0),
            v0,
            v1,
            v2
        )
        .is_none());
    }

    #[test]
    fn segment_respects_bounds() {
        let (v0, v1, v2) = tri();
        // Segment crossing the triangle.
        let hit = segment_intersects_triangle(
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0),
            v0,
            v1,
            v2,
        )
        .unwrap();
        assert!(hit.is_zero_approx(), "{hit:?}");

        // Segment entirely in front of the triangle: no hit.
        assert!(segment_intersects_triangle(
            Vector3::new(0.0, 0.0, -2.0),
            Vector3::new(0.0, 0.0, -1.0),
            v0,
            v1,
            v2
        )
        .is_none());
    }
}
