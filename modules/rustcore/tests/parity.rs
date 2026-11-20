//! Differential tests: every value here comes from Godot's own C++ code, dumped
//! by `parity/dump.cpp` and stored in `fixtures/cpp_math_parity.txt`.
//!
//! Regenerate the fixture with `modules\rustcore\parity\run.cmd`.
//! A failure means the Rust port diverges from the engine.

use godot_core_rs::math::funcs;
use godot_core_rs::math::vector2::Vector2;
use godot_core_rs::math::vector3::Vector3;
use std::collections::HashMap;

const FIXTURE: &str = include_str!("fixtures/cpp_math_parity.txt");

fn load() -> HashMap<&'static str, Vec<f64>> {
    let mut map = HashMap::new();
    for line in FIXTURE.lines() {
        let mut it = line.split(',');
        let name = it.next().unwrap().trim();
        if name.is_empty() {
            continue;
        }
        let vals: Vec<f64> = it.map(|v| v.trim().parse().unwrap()).collect();
        map.insert(name, vals);
    }
    map
}

fn check(map: &HashMap<&'static str, Vec<f64>>, name: &str, got: &[f64], tol: f64) {
    let want = map
        .get(name)
        .unwrap_or_else(|| panic!("missing fixture entry {name}"));
    assert_eq!(want.len(), got.len(), "{name}: arity mismatch");
    for (i, (w, g)) in want.iter().zip(got.iter()).enumerate() {
        let err = (w - g).abs();
        let allowed = tol * (1.0 + w.abs());
        assert!(
            err <= allowed,
            "{name}[{i}]: cpp={w} rust={g} err={err} tol={allowed}"
        );
    }
}

fn v2(v: Vector2) -> [f64; 2] {
    [v.x as f64, v.y as f64]
}

fn v3(v: Vector3) -> [f64; 3] {
    [v.x as f64, v.y as f64, v.z as f64]
}

const F32_TOL: f64 = 1e-5; // one f32 ulp is ~6e-8 relative; this allows a few
const F64_TOL: f64 = 1e-12;

#[test]
fn vector2_matches_cpp() {
    let m = load();
    let a = Vector2::new(1.5, -2.25);
    let b = Vector2::new(-0.75, 4.5);

    check(&m, "v2.sum", &v2(a + b), F32_TOL);
    check(&m, "v2.sub", &v2(a - b), F32_TOL);
    check(&m, "v2.mul", &v2(a * b), F32_TOL);
    check(&m, "v2.div", &v2(a / b), F32_TOL);
    check(&m, "v2.length", &[a.length() as f64, a.length_squared() as f64], F32_TOL);
    check(&m, "v2.len_a2", &[a.length() as f64], F32_TOL);
    check(&m, "v2.dot_a2b2", &[a.dot(b) as f64], F32_TOL);
    check(&m, "v2.cross_a2b2", &[a.cross(b) as f64], F32_TOL);
    check(&m, "v2.angle_a2", &[a.angle() as f64], F32_TOL);
    check(&m, "v2.angle_to_a2b2", &[a.angle_to(b) as f64], F32_TOL);
    check(&m, "v2.normalized", &v2(a.normalized()), F32_TOL);
    check(&m, "v2.rotated", &v2(a.rotated(0.7)), F32_TOL);
    check(&m, "v2.slide", &v2(a.slide(Vector2::UP)), F32_TOL);
    check(&m, "v2.bounce", &v2(a.bounce(Vector2::UP)), F32_TOL);
    check(&m, "v2.reflect", &v2(a.reflect(Vector2::UP)), F32_TOL);
    check(&m, "v2.lerp", &v2(a.lerp(b, 0.35)), F32_TOL);
    check(&m, "v2.slerp", &v2(a.slerp(b, 0.35)), F32_TOL);
    check(&m, "v2.move_toward", &v2(a.move_toward(b, 0.75)), F32_TOL);
    check(&m, "v2.posmod", &v2(a.posmod(1.25)), F32_TOL);
    check(&m, "v2.project", &v2(a.project(b)), F32_TOL);
    check(&m, "v2.snapped", &v2(a.snapped(Vector2::new(0.5, 0.25))), F32_TOL);
    check(&m, "v2.limit_length", &v2(a.limit_length(1.0)), F32_TOL);
    check(&m, "v2.direction_to", &v2(a.direction_to(b)), F32_TOL);
    check(&m, "v2.hash", &[a.hash() as f64], 0.0);
}

#[test]
fn convex_hull_and_bresenham_match_cpp() {
    let m = load();

    let pts = [
        Vector2::new(0.0, 0.0),
        Vector2::new(2.0, 0.0),
        Vector2::new(2.0, 2.0),
        Vector2::new(1.0, 0.5),
        Vector2::new(0.0, 2.0),
        Vector2::new(1.0, 1.0),
    ];
    let hull = g2d::convex_hull(&pts);
    check(&m, "g2d.hull_n", &[hull.len() as f64], 0.0);
    for (i, v) in hull.iter().enumerate() {
        check(&m, &format!("g2d.hull{i}"), &v2(*v), F32_TOL);
    }

    let line = g2d::bresenham_line(Vector2i::new(0, 0), Vector2i::new(5, 2));
    check(&m, "g2d.line_n", &[line.len() as f64], 0.0);
    for (i, v) in line.iter().enumerate() {
        check(&m, &format!("g2d.line{i}"), &[v.x as f64, v.y as f64], 0.0);
    }
}

#[test]
fn vector3_matches_cpp() {
    let m = load();
    let a = Vector3::new(1.5, -2.25, 0.75);
    let b = Vector3::new(-0.75, 4.5, 2.125);
    let axis = Vector3::new(0.36, 0.48, 0.8); // exactly unit length
    let axis2 = Vector3::new(1.0, 1.0, 0.0).normalized();

    check(&m, "v3.sum", &v3(a + b), F32_TOL);
    check(&m, "v3.sub", &v3(a - b), F32_TOL);
    check(&m, "v3.cross", &v3(a.cross(b)), F32_TOL);
    check(&m, "v3.dot", &[a.dot(b) as f64], F32_TOL);
    check(&m, "v3.length", &[a.length() as f64], F32_TOL);
    check(&m, "v3.angle_to", &[a.angle_to(b) as f64], F32_TOL);
    check(&m, "v3.signed_angle_to", &[a.signed_angle_to(b, Vector3::UP) as f64], F32_TOL);
    check(&m, "v3.normalized", &v3(a.normalized()), F32_TOL);
    check(&m, "v3.rotated_axis", &v3(a.rotated(axis, 0.7)), F32_TOL);
    check(&m, "v3.rotated_axis2", &v3(a.rotated(axis2, 1.234)), F32_TOL);
    check(&m, "v3.slide", &v3(a.slide(Vector3::UP)), F32_TOL);
    check(&m, "v3.bounce", &v3(a.bounce(Vector3::UP)), F32_TOL);
    check(&m, "v3.reflect", &v3(a.reflect(Vector3::UP)), F32_TOL);
    check(&m, "v3.lerp", &v3(a.lerp(b, 0.35)), F32_TOL);
    check(&m, "v3.slerp", &v3(a.slerp(b, 0.35)), F32_TOL);
    check(&m, "v3.move_toward", &v3(a.move_toward(b, 0.75)), F32_TOL);
    check(&m, "v3.posmod", &v3(a.posmod(1.25)), F32_TOL);
    check(&m, "v3.project", &v3(a.project(b)), F32_TOL);
    check(&m, "v3.snapped", &v3(a.snapped(Vector3::new(0.5, 0.25, 0.125))), F32_TOL);
    check(&m, "v3.octahedron_encode", &v2(a.normalized().octahedron_encode()), F32_TOL);
    check(
        &m,
        "v3.octahedron_decode",
        &v3(Vector3::octahedron_decode(Vector2::new(0.31, 0.82))),
        F32_TOL,
    );
    check(&m, "v3.any_perp", &v3(a.get_any_perpendicular()), F32_TOL);
    check(&m, "v3.hash", &[a.hash() as f64], 0.0);
    check(&m, "v3.min_axis", &[a.min_axis_index() as f64], 0.0);
    check(&m, "v3.max_axis", &[a.max_axis_index() as f64], 0.0);
}

#[test]
fn scalar_math_matches_cpp() {
    let m = load();

    check(&m, "m.angle_difference", &[funcs::angle_difference(0.3f64, -2.1)], F64_TOL);
    check(&m, "m.lerp_angle", &[funcs::lerp_angle(0.3f64, -2.1, 0.4)], F64_TOL);
    check(&m, "m.wrapf", &[funcs::wrapf(-2.75f64, 0.5, 3.5)], F64_TOL);
    check(&m, "m.snapped", &[funcs::snapped(1.26, 0.5)], F64_TOL);
    check(&m, "m.ease_pos", &[funcs::ease(0.37, 0.6)], F64_TOL);
    check(&m, "m.ease_neg", &[funcs::ease(0.62, -0.4)], F64_TOL);
    check(&m, "m.smoothstep", &[funcs::smoothstep(0.0f64, 1.0, 0.37)], F64_TOL);
    check(&m, "m.bezier", &[funcs::bezier_interpolate(0.0f64, 0.2, 0.8, 1.0, 0.4)], F64_TOL);
    check(&m, "m.bezier_deriv", &[funcs::bezier_derivative(0.0f64, 0.2, 0.8, 1.0, 0.4)], F64_TOL);
    check(&m, "m.cubic", &[funcs::cubic_interpolate(0.1f64, 0.9, -0.2, 1.4, 0.33)], F64_TOL);
    check(
        &m,
        "m.cubic_time",
        &[funcs::cubic_interpolate_in_time(0.1f64, 0.9, -0.2, 1.4, 0.33, 1.0, -1.0, 2.0)],
        F64_TOL,
    );
    check(&m, "m.db_to_linear", &[funcs::db_to_linear(-6.0f64)], F64_TOL);
    check(&m, "m.linear_to_db", &[funcs::linear_to_db(0.5f64)], F64_TOL);
    check(&m, "m.remap", &[funcs::remap(0.3f64, 0.0, 1.0, -5.0, 5.0)], F64_TOL);
    check(&m, "m.pingpong", &[funcs::pingpong(3.4f64, 2.0)], F64_TOL);
    check(&m, "m.larger_prime", &[funcs::larger_prime(200) as f64], 0.0);
    check(&m, "m.step_decimals", &[funcs::step_decimals(0.01) as f64], 0.0);
}

/// The PCG generator must produce the same stream as `thirdparty/misc/pcg.cpp`.
#[test]
fn pcg_sequence_matches_cpp() {
    let m = load();
    let want = m.get("pcg").expect("pcg row missing from fixture");
    let mut rng =
        godot_core_rs::math::pcg::RandomPcg::with_seed(123456789, godot_core_rs::math::pcg::PCG_DEFAULT_INC_64);
    let got: Vec<f64> = (0..8).map(|_| rng.rand() as f64).collect();
    check(&m, "pcg", &got, 0.0);
    assert_eq!(want.len(), 8);
}
// ---------------------------------------------------------------------------
// Wave 2: culling and 2D geometry primitives.
// ---------------------------------------------------------------------------

use godot_core_rs::math::aabb::Aabb;
use godot_core_rs::math::plane::Plane;
use godot_core_rs::math::rect2::Rect2;

#[test]
fn aabb_matches_cpp() {
    let m = load();
    let b = Aabb::new(Vector3::new(0.5, -1.0, 2.0), Vector3::new(3.0, 2.0, 1.5));
    let o = Aabb::new(Vector3::new(2.0, 0.0, 1.0), Vector3::new(2.0, 2.0, 2.0));

    check(&m, "aabb.center", &v3(b.get_center()), F32_TOL);
    check(&m, "aabb.volume", &[b.get_volume() as f64], F32_TOL);
    check(&m, "aabb.intersects", &[b.intersects(&o) as i32 as f64], 0.0);
    check(
        &m,
        "aabb.intersects_no",
        &[b.intersects(&Aabb::new(Vector3::new(10.0, 10.0, 10.0), Vector3::ONE)) as i32 as f64],
        0.0,
    );
    check(&m, "aabb.encloses", &[b.encloses(&o) as i32 as f64], 0.0);

    let merged = b.merge(&o);
    check(&m, "aabb.merge", &v3(merged.position), F32_TOL);
    check(&m, "aabb.merge_size", &v3(merged.size), F32_TOL);

    let inter = b.intersection(&o);
    check(&m, "aabb.intersection", &v3(inter.position), F32_TOL);
    check(&m, "aabb.intersection_size", &v3(inter.size), F32_TOL);

    check(
        &m,
        "aabb.smits",
        &[b.smits_intersect_ray(Vector3::ZERO, Vector3::new(0.5, 0.2, 0.8), 0.0, 100.0) as i32 as f64],
        0.0,
    );
    let hit = b
        .find_intersects_ray(Vector3::ZERO, Vector3::new(0.5, 0.2, 0.8))
        .unwrap();
    check(&m, "aabb.ray_hit", &v3(hit.point), F32_TOL);

    check(&m, "aabb.longest_axis", &[b.get_longest_axis_index() as f64], 0.0);
    check(&m, "aabb.shortest_axis", &[b.get_shortest_axis_index() as f64], 0.0);

    let (rmin, rmax) = b.project_range_in_plane(&Plane::new(0.0, 1.0, 0.0, 0.5));
    check(&m, "aabb.proj_min", &[rmin as f64], F32_TOL);
    check(&m, "aabb.proj_max", &[rmax as f64], F32_TOL);

    let grown = b.grow(0.25);
    check(&m, "aabb.grown", &v3(grown.position), F32_TOL);
    check(&m, "aabb.grown_size", &v3(grown.size), F32_TOL);

    let quant = b.quantized(0.5);
    check(&m, "aabb.quant", &v3(quant.position), F32_TOL);
    check(&m, "aabb.quant_size", &v3(quant.size), F32_TOL);

    check(&m, "aabb.hash", &[b.hash() as f64], 0.0);
}

#[test]
fn plane_matches_cpp() {
    let m = load();
    let plane = Plane::new(0.0, 1.0, 0.0, 2.0);

    check(&m, "plane.distance", &[plane.distance_to(Vector3::new(1.0, 5.5, -2.0)) as f64], F32_TOL);
    check(&m, "plane.project", &v3(plane.project(Vector3::new(1.0, 5.5, -2.0))), F32_TOL);
    check(&m, "plane.center", &v3(plane.get_center()), F32_TOL);
    check(&m, "plane.perp", &v3(plane.get_any_perpendicular_normal()), F32_TOL);

    let p1 = Plane::new(1.0, 0.0, 0.0, 1.0);
    let p2 = Plane::new(0.0, 0.0, 1.0, 3.0);
    check(&m, "plane.intersect3", &v3(plane.intersect_3(&p1, &p2).unwrap()), F32_TOL);
    // Parallel planes have no unique intersection.
    assert!(plane.intersect_3(&Plane::PLANE_XZ, &Plane::new(0.0, 1.0, 0.0, 5.0)).is_none());

    let ray = plane
        .intersects_ray(Vector3::new(0.0, 8.0, 0.0), Vector3::new(0.0, -2.0, 0.0))
        .unwrap();
    check(&m, "plane.ray", &v3(ray), F32_TOL);

    let seg = plane
        .intersects_segment(Vector3::new(0.0, 5.0, 0.0), Vector3::new(0.0, 0.0, 0.0))
        .unwrap();
    check(&m, "plane.segment", &v3(seg), F32_TOL);

    let scaled = Plane::new(0.0, 3.0, 0.0, 6.0).normalized();
    check(&m, "plane.norm_d", &[scaled.d as f64], F32_TOL);
    check(&m, "plane.norm_n", &v3(scaled.normal), F32_TOL);
}

#[test]
fn rect2_matches_cpp() {
    let m = load();
    let r = Rect2::new_xywh(1.0, 2.0, 4.0, 3.0);
    let o = Rect2::new_xywh(-1.0, 0.5, 3.0, 3.0);

    check(&m, "rect.center", &v2(r.get_center()), F32_TOL);
    check(&m, "rect.area", &[r.get_area() as f64], F32_TOL);
    check(&m, "rect.intersects", &[r.intersects(&o, false) as i32 as f64], 0.0);

    let i = r.intersection(&o);
    check(&m, "rect.intersection", &v2(i.position), F32_TOL);
    check(&m, "rect.intersection_size", &v2(i.size), F32_TOL);

    let mrg = r.merge(&o);
    check(&m, "rect.merge", &v2(mrg.position), F32_TOL);
    check(&m, "rect.merge_size", &v2(mrg.size), F32_TOL);

    check(&m, "rect.distance_out", &[r.distance_to(Vector2::new(7.0, 9.0)) as f64], F32_TOL);
    check(&m, "rect.distance_in", &[r.distance_to(Vector2::new(2.0, 3.0)) as f64], F32_TOL);

    let g = r.grow(0.5);
    check(&m, "rect.grow", &v2(g.position), F32_TOL);
    check(&m, "rect.grow_size", &v2(g.size), F32_TOL);

    let e = r.expand(Vector2::new(-2.0, 7.0));
    check(&m, "rect.expand", &v2(e.position), F32_TOL);
    check(&m, "rect.expand_size", &v2(e.size), F32_TOL);

    check(&m, "rect.support", &v2(r.get_support(Vector2::new(1.0, -1.0))), F32_TOL);

    let xf2 = Transform2D::from_rotation_origin(0.6, Vector2::new(0.5, 0.5));
    check(
        &m,
        "rect.xf_hit",
        &[r.intersects_transformed(&xf2, &Rect2::new_xywh(0.0, 0.0, 2.0, 2.0)) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "rect.xf_miss",
        &[r.intersects_transformed(&xf2, &Rect2::new_xywh(50.0, 50.0, 1.0, 1.0)) as i32 as f64],
        0.0,
    );
    let xf3 = Transform2D::from_rotation_origin(0.0, Vector2::new(4.0, 0.0));
    check(
        &m,
        "rect.xf_touch",
        &[r.intersects_transformed(&xf3, &Rect2::new_xywh(0.0, 0.0, 2.0, 2.0)) as i32 as f64],
        0.0,
    );
}

// ---------------------------------------------------------------------------
// Wave 3: transforms (Basis / Quaternion).
// ---------------------------------------------------------------------------

use godot_core_rs::math::basis::{Basis, EulerOrder};
use godot_core_rs::math::quaternion::Quaternion;

#[test]
fn basis_matches_cpp() {
    let m = load();
    let v = Vector3::new(0.4, -1.2, 2.5);
    let b = Basis::from_euler(Vector3::new(0.3, -0.7, 1.1), EulerOrder::YXZ);

    check(&m, "basis.xform", &v3(b.xform(v)), F32_TOL);
    check(&m, "basis.xform_inv", &v3(b.xform_inv(v)), F32_TOL);
    check(&m, "basis.det", &[b.determinant() as f64], F32_TOL);
    check(&m, "basis.inverse_xform", &v3(b.inverse().xform(v)), F32_TOL);
    check(&m, "basis.transposed_col0", &v3(b.transposed().get_column(0)), F32_TOL);
    check(&m, "basis.scale", &v3(b.get_scale()), F32_TOL);
    check(&m, "basis.scale_abs", &v3(b.get_scale_abs()), F32_TOL);
    check(&m, "basis.uniform", &[b.get_uniform_scale() as f64], F32_TOL);

    let mtx = Basis::from_columns(
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(4.0, 5.0, 6.0),
        Vector3::new(7.0, 8.0, 10.0),
    );
    check(&m, "basis.det_m", &[mtx.determinant() as f64], F32_TOL);
    check(&m, "basis.orthonorm_col0", &v3(mtx.orthonormalized().get_column(0)), F32_TOL);
    check(&m, "basis.orthonorm_col2", &v3(mtx.orthonormalized().get_column(2)), F32_TOL);

    let r = Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.77);
    check(&m, "basis.axis_angle", &v3(r.xform(Vector3::new(1.0, 2.0, 3.0))), F32_TOL);
    check(&m, "basis.mul", &v3((r * b).xform(v)), F32_TOL);
    check(
        &m,
        "basis.scaled_local",
        &v3(b.scaled_local(Vector3::new(3.0, 2.0, 0.5)).get_column(1)),
        F32_TOL,
    );
    check(
        &m,
        "basis.slerp",
        &v3(b.slerp(&b.scaled_local(Vector3::splat(2.0)), 0.5).get_column(0)),
        F32_TOL,
    );
}

#[test]
fn quaternion_matches_cpp() {
    let m = load();
    let v = Vector3::new(0.4, -1.2, 2.5);
    let b = Basis::from_euler(Vector3::new(0.3, -0.7, 1.1), EulerOrder::YXZ);
    let q = b.get_quaternion();

    check(&m, "quat.x", &[q.x as f64], F32_TOL);
    check(&m, "quat.y", &[q.y as f64], F32_TOL);
    check(&m, "quat.z", &[q.z as f64], F32_TOL);
    check(&m, "quat.w", &[q.w as f64], F32_TOL);
    check(&m, "quat.xform", &v3(q.xform(v)), F32_TOL);

    let q2 = Quaternion::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.77);
    check(&m, "quat.axis_angle", &v3(q2.xform(v)), F32_TOL);
    check(&m, "quat.slerp", &v3(q.slerp(&q2, 0.35).xform(v)), F32_TOL);
    check(&m, "quat.angle_to", &[q.angle_to(&q2) as f64], F32_TOL);
    check(
        &m,
        "quat.euler",
        &v3(Quaternion::from_euler(Vector3::new(0.3, -0.7, 1.1)).xform(v)),
        F32_TOL,
    );
}

// ---------------------------------------------------------------------------
// Wave 4: Transform3D.
// ---------------------------------------------------------------------------

use godot_core_rs::math::transform_3d::Transform3D;

#[test]
fn transform3d_matches_cpp() {
    let m = load();
    let v = Vector3::new(0.4, -1.2, 2.5);
    let t = Transform3D::new(
        Basis::from_euler(Vector3::new(0.2, 0.6, -0.4), EulerOrder::YXZ),
        Vector3::new(1.0, -2.0, 3.0),
    );
    let t2 = Transform3D::new(
        Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.5),
        Vector3::new(-1.0, 0.5, 2.0),
    );

    check(&m, "t3.xform", &v3(t.xform(v)), F32_TOL);
    check(&m, "t3.xform_inv", &v3(t.xform_inv(t.xform(v))), F32_TOL);
    check(&m, "t3.affine_xform_inv", &v3(t.affine_inverse().xform(t.xform(v))), F32_TOL);
    check(&m, "t3.mul", &v3((t * t2).xform(v)), F32_TOL);
    check(&m, "t3.rotated", &v3(t.rotated(Vector3::UP, 0.3).xform(v)), F32_TOL);
    check(&m, "t3.translated", &v3(t.translated(Vector3::new(1.0, 2.0, 3.0)).xform(v)), F32_TOL);

    let b = Aabb::new(Vector3::new(-1.0, -2.0, -3.0), Vector3::new(2.0, 4.0, 6.0));
    let tb = t.xform_aabb(&b);
    check(&m, "t3.aabb_pos", &v3(tb.position), F32_TOL);
    check(&m, "t3.aabb_size", &v3(tb.size), F32_TOL);

    let pl = Plane::new(0.0, 1.0, 0.0, 0.5);
    let tp = t.xform_plane(&pl);
    check(&m, "t3.plane_n", &v3(tp.normal), F32_TOL);
    check(&m, "t3.plane_d", &[tp.d as f64], F32_TOL);

    check(&m, "t3.interp", &v3(t.interpolate_with(&t2, 0.35).xform(v)), 1e-4);
    check(&m, "t3.scale_origin", &v3(t.scaled(Vector3::new(2.0, 0.5, 1.5)).origin), F32_TOL);

    let batch = [
        Vector3::new(0.5, 1.5, -0.5),
        Vector3::new(-1.0, 2.0, 0.25),
    ];
    let out = t.xform_array(&batch);
    let inv = t.xform_array_inv(&batch);
    for i in 0..batch.len() {
        check(&m, &format!("t3.batch{i}"), &v3(out[i]), F32_TOL);
        check(&m, &format!("t3.batch_inv{i}"), &v3(inv[i]), F32_TOL);
    }
}

// ---------------------------------------------------------------------------
// Wave 5: Euler decomposition for all six orders.
// ---------------------------------------------------------------------------

#[test]
fn euler_decomposition_matches_cpp() {
    let m = load();
    let e = Vector3::new(0.3, -0.7, 1.1);
    let orders = [
        (EulerOrder::XYZ, "xyz"),
        (EulerOrder::XZY, "xzy"),
        (EulerOrder::YXZ, "yxz"),
        (EulerOrder::YZX, "yzx"),
        (EulerOrder::ZXY, "zxy"),
        (EulerOrder::ZYX, "zyx"),
    ];

    for (order, name) in orders {
        check(
            &m,
            &format!("euler.{name}"),
            &v3(Basis::from_euler(e, order).get_euler(order)),
            F32_TOL,
        );
        let mixed = Basis::from_euler(Vector3::ZERO, order)
            .rotated(Vector3::new(0.3, 0.8, -0.5).normalized(), 1.1);
        check(
            &m,
            &format!("euler_round.{name}"),
            &v3(mixed.get_euler(order)),
            F32_TOL,
        );
    }

    let arbitrary = Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.77)
        * Basis::from_axis_angle(Vector3::UP, 0.4);
    for (order, name) in orders {
        check(
            &m,
            &format!("euler_arb.{name}"),
            &v3(arbitrary.get_euler(order)),
            F32_TOL,
        );
    }
}

// ---------------------------------------------------------------------------
// Wave 6: integer vectors.
// ---------------------------------------------------------------------------

use godot_core_rs::math::vector2i::Vector2i;
use godot_core_rs::math::vector3i::Vector3i;

#[test]
fn integer_vectors_match_cpp() {
    let m = load();
    let a = Vector3i::new(3, -4, 12);
    let b = Vector3i::new(-1, 5, 2);

    check(&m, "v3i.sum", &[((a + b).x) as f64, ((a + b).y) as f64, ((a + b).z) as f64], 0.0);
    check(&m, "v3i.min", &[a.min(b).x as f64, a.min(b).y as f64, a.min(b).z as f64], 0.0);
    check(&m, "v3i.max", &[a.max(b).x as f64, a.max(b).y as f64, a.max(b).z as f64], 0.0);
    check(&m, "v3i.abs", &[a.abs().x as f64, a.abs().y as f64, a.abs().z as f64], 0.0);
    check(&m, "v3i.sign", &[a.sign().x as f64, a.sign().y as f64, a.sign().z as f64], 0.0);
    let c = Vector3i::new(9, -9, 4).clamp(Vector3i::new(-5, -5, -5), Vector3i::new(5, 5, 5));
    check(&m, "v3i.clamp", &[c.x as f64, c.y as f64, c.z as f64], 0.0);
    check(&m, "v3i.len_sq", &[a.length_squared() as f64], 0.0);
    check(&m, "v3i.len", &[a.length()], 1e-12);
    check(&m, "v3i.hash", &[a.hash() as f64], 0.0);
    check(&m, "v3i.min_axis", &[a.min_axis_index() as f64], 0.0);
    let sn3 = Vector3i::new(-5, 3, 17).snappedi(4);
    check(
        &m,
        "v3i.snap",
        &[sn3.x as f64, sn3.y as f64, sn3.z as f64],
        0.0,
    );

    let d = Vector2i::new(7, -9);
    let mul = d * Vector2i::new(2, 3);
    check(&m, "v2i.mul", &[mul.x as f64, mul.y as f64], 0.0);
    check(&m, "v2i.len_sq", &[d.length_squared() as f64], 0.0);
    check(&m, "v2i.hash", &[d.hash() as f64], 0.0);
    let sn2 = Vector2i::new(-5, 3).snappedi(4);
    check(&m, "v2i.snap", &[sn2.x as f64, sn2.y as f64], 0.0);
}

// ---------------------------------------------------------------------------
// Wave 7: axis-angle extraction, filled polygons, Vector4.
// ---------------------------------------------------------------------------

use godot_core_rs::math::vector4::Vector4;

fn v4(v: Vector4) -> [f64; 4] {
    [v.x as f64, v.y as f64, v.z as f64, v.w as f64]
}

#[test]
fn axis_angle_matches_cpp() {
    let m = load();

    let (axis, angle) = Basis::IDENTITY.get_axis_angle();
    check(&m, "axis.id_axis", &v3(axis), F32_TOL);
    check(&m, "axis.id_angle", &[angle as f64], F32_TOL);

    let (axis, angle) = Basis::from_axis_angle(Vector3::UP, std::f32::consts::PI).get_axis_angle();
    check(&m, "axis.180_axis", &v3(axis), F32_TOL);
    check(&m, "axis.180_angle", &[angle as f64], 1e-4);

    let (axis, angle) = Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.77).get_axis_angle();
    check(&m, "axis.gen_axis", &v3(axis), 1e-4);
    check(&m, "axis.gen_angle", &[angle as f64], 1e-4);

    let (axis, angle) = Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 2.9).get_axis_angle();
    check(&m, "axis.gen2_axis", &v3(axis), 1e-4);
    check(&m, "axis.gen2_angle", &[angle as f64], 1e-4);
}

#[test]
fn filled_polygon_matches_cpp() {
    let m = load();
    let rect = Rect2::new_xywh(0.0, 0.0, 10.0, 10.0);

    let far = [
        Vector2::new(20.0, 20.0),
        Vector2::new(30.0, 20.0),
        Vector2::new(20.0, 30.0),
    ];
    check(&m, "poly.far", &[rect.intersects_filled_polygon(&far) as i32 as f64], 0.0);

    let overlap = [
        Vector2::new(5.0, 5.0),
        Vector2::new(15.0, 5.0),
        Vector2::new(5.0, 15.0),
    ];
    check(&m, "poly.overlap", &[rect.intersects_filled_polygon(&overlap) as i32 as f64], 0.0);

    let inside = [
        Vector2::new(2.0, 2.0),
        Vector2::new(8.0, 2.0),
        Vector2::new(2.0, 8.0),
    ];
    check(&m, "poly.inside", &[rect.intersects_filled_polygon(&inside) as i32 as f64], 0.0);
}

#[test]
fn vector4_matches_cpp() {
    let m = load();
    let a = Vector4::new(1.5, -2.25, 0.75, 4.0);
    let b = Vector4::new(-0.5, 3.0, 2.25, -1.5);

    check(&m, "v4.sum4", &v4(a + b), F32_TOL);
    check(&m, "v4.mul4", &v4(a * b), F32_TOL);
    check(&m, "v4.dot", &[a.dot(b) as f64], F32_TOL);
    check(&m, "v4.len", &[a.length() as f64], F32_TOL);
    check(&m, "v4.norm4", &v4(a.normalized()), F32_TOL);
    check(
        &m,
        "v4.snap4",
        &v4(a.snapped(Vector4::new(0.5, 0.25, 0.125, 1.0))),
        F32_TOL,
    );
    check(&m, "v4.posmod4", &v4(a.posmod(1.25)), F32_TOL);
    check(&m, "v4.hash", &[a.hash() as f64], 0.0);
    check(&m, "v4.min_axis", &[a.min_axis_index() as f64], 0.0);
    check(&m, "v4.max_axis", &[a.max_axis_index() as f64], 0.0);
}

// ---------------------------------------------------------------------------
// Wave 8: Transform2D.
// ---------------------------------------------------------------------------

use godot_core_rs::math::transform_2d::Transform2D;

#[test]
fn transform2d_matches_cpp() {
    let m = load();
    let v = Vector2::new(1.5, -0.25);
    let t = Transform2D::from_rotation_origin(0.7, Vector2::new(2.0, 3.0));
    let s = Transform2D::from_rotation_scale_skew_origin(
        0.7,
        Vector2::new(2.0, 0.5),
        0.1,
        Vector2::new(1.0, -2.0),
    );

    check(&m, "t2.xform", &v2(t.xform(v)), F32_TOL);
    check(&m, "t2.det", &[t.determinant() as f64], F32_TOL);
    check(&m, "t2.affine_inv", &v2(s.affine_inverse().xform(s.xform(v))), F32_TOL);
    check(&m, "t2.rot", &[s.get_rotation() as f64], F32_TOL);
    check(&m, "t2.scale", &v2(s.get_scale()), F32_TOL);
    check(&m, "t2.skew", &[s.get_skew() as f64], F32_TOL);
    check(&m, "t2.mul", &v2((t * s).xform(Vector2::new(0.3, 0.9))), F32_TOL);
    check(&m, "t2.origin", &v2(s.get_origin()), F32_TOL);
    check(
        &m,
        "t2.interp",
        &v2(s.interpolate_with(&t, 0.35).xform(Vector2::new(0.3, 0.9))),
        F32_TOL,
    );
}

// ---------------------------------------------------------------------------
// Wave 9: Rect2i.
// ---------------------------------------------------------------------------

use godot_core_rs::math::rect2i::Rect2i;

#[test]
fn rect2i_matches_cpp() {
    let m = load();
    let ri = Rect2i::new_xywh(1, 2, 4, 3);
    let ro = Rect2i::new_xywh(-1, 0, 3, 4);

    let c = ri.get_center();
    check(&m, "ri.center", &[c.x as f64, c.y as f64], 0.0);

    let i = ri.intersection(&ro);
    check(&m, "ri.inter_pos", &[i.position.x as f64, i.position.y as f64], 0.0);
    check(&m, "ri.inter_size", &[i.size.x as f64, i.size.y as f64], 0.0);

    let mrg = ri.merge(&ro);
    check(&m, "ri.merge_pos", &[mrg.position.x as f64, mrg.position.y as f64], 0.0);
    check(&m, "ri.merge_size", &[mrg.size.x as f64, mrg.size.y as f64], 0.0);

    check(&m, "ri.area", &[ri.get_area() as f64], 0.0);
    check(&m, "ri.hash", &[ri.hash() as f64], 0.0);
    check(&m, "ri.intersects", &[ri.intersects(&ro) as i32 as f64], 0.0);

    let g = ri.grow(2);
    check(&m, "ri.grow_pos", &[g.position.x as f64, g.position.y as f64], 0.0);
    check(&m, "ri.grow_size", &[g.size.x as f64, g.size.y as f64], 0.0);
}

// ---------------------------------------------------------------------------
// Wave 10: Geometry2D hit testing.
// ---------------------------------------------------------------------------

use godot_core_rs::math::geometry_2d as g2d;

#[test]
fn geometry2d_matches_cpp() {
    let m = load();

    let hit = g2d::segment_intersects_segment(
        Vector2::new(-1.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, -1.0),
        Vector2::new(0.0, 1.0),
    );
    check(&m, "g2d.seg_hit", &[hit.is_some() as i32 as f64], 0.0);
    check(&m, "g2d.seg_pt", &v2(hit.unwrap()), F32_TOL);

    check(
        &m,
        "g2d.par",
        &[g2d::segment_intersects_segment(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
        )
        .is_some() as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g2d.outside",
        &[g2d::segment_intersects_segment(
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(2.0, -1.0),
            Vector2::new(2.0, 1.0),
        )
        .is_some() as i32 as f64],
        0.0,
    );

    check(
        &m,
        "g2d.circle_in",
        &[g2d::is_point_in_circle(Vector2::new(1.0, 0.0), Vector2::ZERO, 1.0) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g2d.circle_out",
        &[g2d::is_point_in_circle(Vector2::new(1.5, 0.0), Vector2::ZERO, 1.0) as i32 as f64],
        0.0,
    );

    let square = [
        Vector2::new(0.0, 0.0),
        Vector2::new(4.0, 0.0),
        Vector2::new(4.0, 4.0),
        Vector2::new(0.0, 4.0),
    ];
    check(
        &m,
        "g2d.poly_in",
        &[g2d::is_point_in_polygon(Vector2::new(2.0, 2.0), &square) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g2d.poly_out",
        &[g2d::is_point_in_polygon(Vector2::new(5.0, 2.0), &square) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g2d.seg_poly",
        &[g2d::is_segment_intersecting_polygon(
            Vector2::new(-1.0, 2.0),
            Vector2::new(5.0, 2.0),
            &square,
        ) as i32 as f64],
        0.0,
    );
}

// ---------------------------------------------------------------------------
// Wave 11: Projection.
// ---------------------------------------------------------------------------

use godot_core_rs::math::projection::Projection;

#[test]
fn projection_matches_cpp() {
    let m = load();
    let persp = Projection::create_perspective(75.0, 16.0 / 9.0, 0.05, 1000.0, false);

    let v4out = persp.xform(Vector4::new(1.0, 2.0, -10.0, 1.0));
    check(&m, "proj.xform4", &v4(v4out), F32_TOL);
    check(&m, "proj.xform3", &v3(persp.xform_vec3(Vector3::new(1.0, 2.0, -10.0))), F32_TOL);
    check(&m, "proj.det", &[persp.determinant() as f64], F32_TOL);
    check(&m, "proj.znear", &[persp.get_z_near() as f64], F32_TOL);
    check(&m, "proj.zfar", &[persp.get_z_far() as f64], F32_TOL);
    check(&m, "proj.fov", &[persp.get_fov() as f64], F32_TOL);
    check(&m, "proj.aspect", &[persp.get_aspect() as f64], F32_TOL);
    check(&m, "proj.ortho", &[persp.is_orthogonal() as i32 as f64], 0.0);

    let ortho = Projection::create_orthogonal(-2.0, 2.0, -1.0, 1.0, 0.1, 100.0);
    check(&m, "ortho.xform3", &v3(ortho.xform_vec3(Vector3::new(1.0, 0.5, -10.0))), F32_TOL);
    check(&m, "ortho.is_ortho", &[ortho.is_orthogonal() as i32 as f64], 0.0);

    let fr = Projection::create_frustum(-1.0, 1.0, -0.5, 0.5, 0.1, 100.0);
    check(&m, "frustum.xform3", &v3(fr.xform_vec3(Vector3::new(0.5, 0.2, -5.0))), F32_TOL);

    let pinv = persp.inverse();
    check(&m, "proj.inv4", &v4(pinv.xform(Vector4::new(0.3, -0.7, -5.0, 1.0))), F32_TOL);

    check(&m, "proj.vp_half", &v2(persp.get_viewport_half_extents()), F32_TOL);
    check(&m, "proj.far_half", &v2(persp.get_far_plane_half_extents()), 1e-4);
    check(&m, "proj.px_m", &[persp.get_pixels_per_meter(1920) as f64], 0.0);
    check(&m, "proj.lod_mul", &[persp.get_lod_multiplier() as f64], F32_TOL);

    let mut scaled_p = Projection::IDENTITY;
    scaled_p.make_scale(Vector3::new(2.0, 0.5, 3.0));
    check(&m, "proj.scale_col0", &[scaled_p.columns[0].x as f64, scaled_p.columns[0].y as f64, scaled_p.columns[0].z as f64], F32_TOL);
    check(&m, "proj.scale_origin", &[scaled_p.columns[3].x as f64, scaled_p.columns[3].y as f64, scaled_p.columns[3].z as f64], F32_TOL);

    let mut adj = persp;
    adj.adjust_perspective_znear(0.25);
    check(&m, "adj.znear", &[adj.get_z_near() as f64], F32_TOL);
    check(&m, "adj.zfar", &[adj.get_z_far() as f64], F32_TOL);
    check(&m, "adj.xform", &v3(adj.xform_vec3(Vector3::new(0.3, 0.2, -3.0))), F32_TOL);

    let hmd_l = Projection::create_for_hmd(1, 1.7777778, 0.064, 0.14, 0.045, 1.2, 0.05, 1000.0);
    let hmd_r = Projection::create_for_hmd(2, 1.7777778, 0.064, 0.14, 0.045, 1.2, 0.05, 1000.0);
    let hmd_m = Projection::create_for_hmd(0, 1.7777778, 0.064, 0.14, 0.045, 1.2, 0.05, 1000.0);
    check(&m, "hmd.left_pt", &v3(hmd_l.xform_vec3(Vector3::new(0.1, 0.05, -2.0))), F32_TOL);
    check(&m, "hmd.right_pt", &v3(hmd_r.xform_vec3(Vector3::new(0.1, 0.05, -2.0))), F32_TOL);
    check(&m, "hmd.mono_identity", &[(hmd_m == Projection::IDENTITY) as i32 as f64], 0.0);

    // create_fit_aabb and perspective_znear_adjusted.
    let fit2 = Projection::create_fit_aabb(&Aabb::new(
        Vector3::new(-1.0, -2.0, 0.5),
        Vector3::new(2.0, 4.0, 1.0),
    ));
    check(&m, "proj.fit2_min", &v3(fit2.xform_vec3(Vector3::new(-1.0, -2.0, 0.5))), F32_TOL);
    check(&m, "proj.fit2_max", &v3(fit2.xform_vec3(Vector3::new(1.0, 2.0, 1.5))), F32_TOL);

    let zn = persp.perspective_znear_adjusted(0.75);
    check(&m, "proj.zn_near", &[zn.get_z_near() as f64], F32_TOL);
    check(&m, "proj.zn_pt", &v3(zn.xform_vec3(Vector3::new(0.4, -0.3, -6.0))), F32_TOL);

    let mut fit = Projection::IDENTITY;
    fit.scale_translate_to_fit(&Aabb::new(
        Vector3::new(-2.0, -1.0, 3.0),
        Vector3::new(4.0, 2.0, 6.0),
    ));
    check(&m, "proj.fit_min", &v3(fit.xform_vec3(Vector3::new(-2.0, -1.0, 3.0))), F32_TOL);
    check(&m, "proj.fit_max", &v3(fit.xform_vec3(Vector3::new(2.0, 1.0, 9.0))), F32_TOL);
    check(&m, "proj.fit_mid", &v3(fit.xform_vec3(Vector3::new(0.0, 0.0, 6.0))), F32_TOL);

    let view = Transform3D::new(
        Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.4),
        Vector3::new(3.0, -2.0, 7.0),
    );
    let wplanes = persp.get_projection_planes(&view);
    for (i, pl) in wplanes.iter().enumerate() {
        check(&m, &format!("proj.wplane{i}"), &v3(pl.normal), 1e-4);
        check(&m, &format!("proj.wplane_d{i}"), &[pl.d as f64], 1e-3);
    }

    // Asymmetric frustum: exercises the `columns[2][0] != 0` branch of get_fov.
    let asym = Projection::create_frustum(-2.0, 1.0, -1.0, 1.5, 0.1, 100.0);
    check(&m, "proj.asym_fov", &[asym.get_fov() as f64], 1e-4);
    check(&m, "proj.asym_aspect", &[asym.get_aspect() as f64], F32_TOL);
    check(&m, "proj.asym_pt", &v3(asym.xform_vec3(Vector3::new(0.0, 0.0, -5.0))), F32_TOL);

    let fy = persp.flipped_y();
    check(&m, "proj.flip_col1", &v4(fy.columns[1]), F32_TOL);
}

// ---------------------------------------------------------------------------
// Wave 12: Basis::diagonalize (Jacobi).
// ---------------------------------------------------------------------------

#[test]
fn diagonalize_matches_cpp() {
    let m = load();
    let mut sym = Basis::from_columns(
        Vector3::new(2.0, 0.5, 0.25),
        Vector3::new(0.5, 3.0, 0.75),
        Vector3::new(0.25, 0.75, 5.0),
    );
    let acc = sym.diagonalize();

    check(&m, "diag.col0", &v3(sym.get_column(0)), 1e-4);
    check(&m, "diag.col1", &v3(sym.get_column(1)), 1e-4);
    check(&m, "diag.col2", &v3(sym.get_column(2)), 1e-4);
    check(&m, "diag.rot", &v3(acc.get_column(0)), 1e-4);
    check(&m, "diag.rot1", &v3(acc.get_column(1)), 1e-4);

    let off = sym.rows[0][1].abs() + sym.rows[0][2].abs() + sym.rows[1][2].abs();
    check(&m, "diag.off", &[off as f64], 1e-6);

    // The diagonal must reproduce the eigenvalues of the original matrix.
    assert!((sym.rows[0][0] - 1.79175889).abs() < 1e-4, "{}", sym.rows[0][0]);
    assert!((sym.rows[1][1] - 2.90997028).abs() < 1e-4, "{}", sym.rows[1][1]);
    assert!((sym.rows[2][2] - 5.29827023).abs() < 1e-4, "{}", sym.rows[2][2]);
}

// ---------------------------------------------------------------------------
// Wave 13: spherical-harmonic rotation.
// ---------------------------------------------------------------------------

#[test]
fn rotate_sh_matches_cpp() {
    let m = load();
    let b = Basis::from_euler(Vector3::new(0.3, -0.7, 1.1), EulerOrder::YXZ);
    let mut sh = [0.5f32, -0.25, 0.75, 1.0, -0.5, 0.125, 0.3, -0.8, 0.6];
    b.rotate_sh(&mut sh);

    let got: Vec<f64> = sh.iter().map(|v| *v as f64).collect();
    check(&m, "sh", &got, F32_TOL);

    // Rotating by identity must be a no-op.
    let mut id = [0.5f32, -0.25, 0.75, 1.0, -0.5, 0.125, 0.3, -0.8, 0.6];
    Basis::IDENTITY.rotate_sh(&mut id);
    assert!((id[0] - 0.5).abs() < 1e-6);
    assert!((id[1] + 0.25).abs() < 1e-6);
}

// ---------------------------------------------------------------------------
// Wave 14: Vector4i.
// ---------------------------------------------------------------------------

use godot_core_rs::math::vector4i::Vector4i;

#[test]
fn vector4i_matches_cpp() {
    let m = load();
    let a = Vector4i::new(1, -2, 3, -4);
    let b = Vector4i::new(5, 6, -7, 8);

    let s = a + b;
    check(&m, "v4i.sum", &[s.x as f64, s.y as f64, s.z as f64, s.w as f64], 0.0);
    let mn = a.min(b);
    check(&m, "v4i.min", &[mn.x as f64, mn.y as f64, mn.z as f64, mn.w as f64], 0.0);
    let ab = a.abs();
    check(&m, "v4i.abs", &[ab.x as f64, ab.y as f64, ab.z as f64, ab.w as f64], 0.0);
    check(&m, "v4i.len_sq", &[a.length_squared() as f64], 0.0);
    check(&m, "v4i.sum64", &[a.dot(b) as f64], 0.0);
    check(&m, "v4i.hash", &[a.hash() as f64], 0.0);

    let sn = Vector4i::new(-5, 3, 17, -1).snappedi(4);
    check(&m, "v4i.snap", &[sn.x as f64, sn.y as f64, sn.z as f64, sn.w as f64], 0.0);
    check(&m, "v4i.min_axis", &[a.min_axis_index() as f64], 0.0);
    check(&m, "v4i.max_axis", &[a.max_axis_index() as f64], 0.0);
}

// ---------------------------------------------------------------------------
// Wave 15: Face3.
// ---------------------------------------------------------------------------

use godot_core_rs::math::color::Color;
use godot_core_rs::math::face3::Face3;
use godot_core_rs::math::geometry_3d as g3_3d;

#[test]
fn face3_matches_cpp() {
    let m = load();
    let f = Face3::new(
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(2.0, -1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    check(&m, "f3.area", &[f.get_area() as f64], F32_TOL);
    check(&m, "f3.degenerate", &[f.is_degenerate() as i32 as f64], 0.0);
    check(&m, "f3.plane_n", &v3(f.get_plane().normal), F32_TOL);
    check(&m, "f3.plane_d", &[f.get_plane().d as f64], F32_TOL);
    check(&m, "f3.aabb_pos", &v3(f.get_aabb().position), F32_TOL);
    check(&m, "f3.aabb_size", &v3(f.get_aabb().size), F32_TOL);

    check(&m, "f3.closest", &v3(f.get_closest_point_to(Vector3::new(0.6, -0.2, 1.5))), F32_TOL);
    check(&m, "f3.closest_in", &v3(f.get_closest_point_to(Vector3::new(0.3, -0.5, 0.0))), F32_TOL);

    let hit = Aabb::new(Vector3::new(-0.5, -0.5, -0.5), Vector3::ONE);
    check(&m, "f3.aabb_hit", &[f.intersects_aabb(&hit) as i32 as f64], 0.0);
    let miss = Aabb::new(Vector3::splat(5.0), Vector3::ONE);
    check(&m, "f3.aabb_miss", &[f.intersects_aabb(&miss) as i32 as f64], 0.0);

    let ray = f.intersects_ray(Vector3::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
    check(&m, "f3.ray_hit", &[ray.is_some() as i32 as f64], 0.0);
    check(&m, "f3.ray_pt", &v3(ray.unwrap()), F32_TOL);
    let seg = f.intersects_segment(Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0));
    check(&m, "f3.seg_hit", &[seg.is_some() as i32 as f64], 0.0);
    check(&m, "f3.seg_pt", &v3(seg.unwrap()), F32_TOL);
    check(
        &m,
        "f3.ray_miss",
        &[f.intersects_ray(Vector3::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, -1.0)).is_some() as i32 as f64],
        0.0,
    );

    // Support points: face, edge or vertex depending on alignment.
    let identity_t = Transform3D::IDENTITY;
    let sup_face = f.get_support(Vector3::new(0.0, 0.0, 1.0), &identity_t, 3);
    check(&m, "f3.sup_face_n", &[sup_face.len() as f64], 0.0);
    check(&m, "f3.sup_face0", &v3(sup_face[0]), F32_TOL);

    let sup_vertex = f.get_support(Vector3::new(0.0, 1.0, 0.0), &identity_t, 3);
    check(&m, "f3.sup_vertex_n", &[sup_vertex.len() as f64], 0.0);
    check(&m, "f3.sup_vertex0", &v3(sup_vertex[0]), F32_TOL);

    let xf_t = Transform3D::new(
        Basis::from_axis_angle(Vector3::UP, 0.5),
        Vector3::new(1.0, 2.0, 3.0),
    );
    let (pr_min, pr_max) = f.project_range_xf(Vector3::UP, &xf_t);
    check(&m, "f3.pr_min", &[pr_min as f64], F32_TOL);
    check(&m, "f3.pr_max", &[pr_max as f64], F32_TOL);

    let sup_xf = f.get_support(Vector3::new(0.0, 0.0, 1.0), &xf_t, 3);
    check(&m, "f3.sup_xf_n", &[sup_xf.len() as f64], 0.0);
    check(&m, "f3.sup_xf0", &v3(sup_xf[0]), F32_TOL);

    let parts = f.split_by_plane(&Plane::new(0.0, 1.0, 0.0, 0.0));
    check(&m, "f3.split_count", &[parts.len() as f64], 0.0);
    check(&m, "f3.split_area0", &[parts[0].0.get_area() as f64], F32_TOL);
    check(&m, "f3.split_area1", &[parts[1].0.get_area() as f64], F32_TOL);
}

// ---------------------------------------------------------------------------
// Wave 16: Okhsl / Okhsv (thirdparty/misc/ok_color.h).
// ---------------------------------------------------------------------------

use godot_core_rs::math::ok_color as ok;

const OK_FIXTURE: &str = include_str!("fixtures/cpp_okcolor_parity.txt");

fn ok_load() -> std::collections::HashMap<&'static str, Vec<f64>> {
    let mut map = std::collections::HashMap::new();
    for line in OK_FIXTURE.lines() {
        let mut it = line.split(',');
        let name = it.next().unwrap().trim();
        if name.is_empty() {
            continue;
        }
        map.insert(name, it.map(|v| v.trim().parse().unwrap()).collect());
    }
    map
}

#[test]
fn ok_color_matches_cpp() {
    let m = ok_load();

    let cases = [
        ("ok.hsl_red", ok::Hsl { h: 0.0, s: 1.0, l: 0.5 }),
        ("ok.hsl_green", ok::Hsl { h: 0.333, s: 0.8, l: 0.6 }),
        ("ok.hsl_blue", ok::Hsl { h: 0.666, s: 0.55, l: 0.35 }),
        ("ok.hsl_mid", ok::Hsl { h: 0.12, s: 0.4, l: 0.72 }),
    ];
    for (name, hsl) in cases {
        let c = ok::okhsl_to_srgb(hsl);
        check(&m, name, &[c.r as f64, c.g as f64, c.b as f64], F32_TOL);
    }

    let hsl_cases = [
        ("ok.to_hsl_1", ok::Rgb { r: 0.2, g: 0.6, b: 0.9 }),
        ("ok.to_hsl_2", ok::Rgb { r: 0.9, g: 0.1, b: 0.05 }),
    ];
    for (name, rgb) in hsl_cases {
        let c = ok::srgb_to_okhsl(rgb);
        check(&m, name, &[c.h as f64, c.s as f64, c.l as f64], F32_TOL);
    }

    let hsv_cases = [
        ("ok.hsv_1", ok::Hsv { h: 0.25, s: 0.7, v: 0.8 }),
        ("ok.hsv_2", ok::Hsv { h: 0.8, s: 0.35, v: 0.5 }),
    ];
    for (name, hsv) in hsv_cases {
        let c = ok::okhsv_to_srgb(hsv);
        check(&m, name, &[c.r as f64, c.g as f64, c.b as f64], F32_TOL);
    }

    let back = [
        ("ok.to_hsv_1", ok::Rgb { r: 0.2, g: 0.6, b: 0.9 }),
        ("ok.to_hsv_2", ok::Rgb { r: 0.9, g: 0.1, b: 0.05 }),
    ];
    for (name, rgb) in back {
        let c = ok::srgb_to_okhsv(rgb);
        check(&m, name, &[c.h as f64, c.s as f64, c.v as f64], F32_TOL);
    }
}

// ---------------------------------------------------------------------------
// Wave 17: frustum planes and rotation/scale decomposition.
// ---------------------------------------------------------------------------

#[test]
fn projection_planes_match_cpp() {
    let m = load();
    let persp = Projection::create_perspective(75.0, 16.0 / 9.0, 0.05, 1000.0, false);

    for i in 0..6 {
        let plane = persp.get_projection_plane(i);
        check(&m, &format!("proj.plane{i}"), &v3(plane.normal), F32_TOL);
        check(&m, &format!("proj.plane_d{i}"), &[plane.d as f64], F32_TOL);
    }
}

#[test]
fn rotref_decomposition_matches_cpp() {
    let m = load();
    let b = Basis::from_axis_angle(Vector3::new(0.36, 0.48, 0.8), 0.77)
        .scaled(Vector3::new(2.0, 0.5, 1.5));
    let (rotref, scale) = b.rotref_posscale_decomposition();

    check(&m, "rr.scale", &v3(scale), F32_TOL);
    check(&m, "rr.rot0", &v3(rotref.get_column(0)), F32_TOL);
    check(&m, "rr.rot2", &v3(rotref.get_column(2)), F32_TOL);

    // Recomposition must return the original basis.
    let rebuilt = rotref.scaled_local(Vector3::new(scale.x, scale.y, scale.z));
    assert!(rebuilt.is_equal_approx(&b), "{rebuilt:?} vs {b:?}");
}

// ---------------------------------------------------------------------------
// Wave 18: core/string character classification (Phase 2 start).
// ---------------------------------------------------------------------------

use godot_core_rs::string::char_utils as cu;

const CU_FIXTURE: &str = include_str!("fixtures/cpp_charutils_parity.txt");

fn fnv(pred: fn(u32) -> bool) -> (u64, u64) {
    let mut count = 0u64;
    let mut hash: u64 = 1469598103934665603;
    for c in 0u32..=0x10FFFF {
        if pred(c) {
            count += 1;
            hash = (hash ^ c as u64).wrapping_mul(1099511628211);
        }
    }
    (count, hash)
}

#[test]
fn char_utils_match_cpp() {
    let mut map = std::collections::HashMap::new();
    for line in CU_FIXTURE.lines() {
        let mut it = line.split(',');
        let name = it.next().unwrap().trim();
        if name.is_empty() {
            continue;
        }
        let count: u64 = it.next().unwrap().trim().parse().unwrap();
        let hash: u64 = it.next().unwrap().trim().parse().unwrap();
        map.insert(name.to_string(), (count, hash));
    }

    let cases: [(&str, fn(u32) -> bool); 20] = [
        ("cu.is_ascii_upper_case", cu::is_ascii_upper_case),
        ("cu.is_ascii_lower_case", cu::is_ascii_lower_case),
        ("cu.is_digit", cu::is_digit),
        ("cu.is_hex_digit", cu::is_hex_digit),
        ("cu.is_binary_digit", cu::is_binary_digit),
        ("cu.is_ascii_alphabet_char", cu::is_ascii_alphabet_char),
        ("cu.is_ascii_alphanumeric_char", cu::is_ascii_alphanumeric_char),
        ("cu.is_ascii_identifier_char", cu::is_ascii_identifier_char),
        ("cu.is_symbol", cu::is_symbol),
        ("cu.is_control", cu::is_control),
        ("cu.is_whitespace", cu::is_whitespace),
        ("cu.is_linebreak", cu::is_linebreak),
        ("cu.is_punct", cu::is_punct),
        ("cu.is_underscore", cu::is_underscore),
        ("cu.is_hyphen", cu::is_hyphen),
        ("cu.is_unicode_identifier_start", cu::is_unicode_identifier_start),
        ("cu.is_unicode_identifier_continue", cu::is_unicode_identifier_continue),
        ("cu.is_unicode_upper_case", cu::is_unicode_upper_case),
        ("cu.is_unicode_lower_case", cu::is_unicode_lower_case),
        ("cu.is_unicode_letter", cu::is_unicode_letter),
    ];

    for (name, pred) in cases {
        let want = map.get(name).unwrap_or_else(|| panic!("missing {name}"));
        let got = fnv(pred);
        assert_eq!(got, *want, "{name}: count/hash differ from the engine");
    }
}

// ---------------------------------------------------------------------------
// Wave 19: Geometry3D helpers.
// ---------------------------------------------------------------------------

#[test]
fn geometry3d_helpers_match_cpp() {
    let m = load();
    let a = Vector3::ZERO;
    let b = Vector3::new(2.0, 2.0, 2.0);

    check(
        &m,
        "g3.closest_seg",
        &v3(g3_3d::get_closest_point_to_segment(Vector3::new(3.0, 0.0, 1.0), a, b)),
        F32_TOL,
    );
    check(
        &m,
        "g3.closest_seg_before",
        &v3(g3_3d::get_closest_point_to_segment(Vector3::splat(-3.0), a, b)),
        F32_TOL,
    );
    check(
        &m,
        "g3.closest_seg_unc",
        &v3(g3_3d::get_closest_point_to_segment_uncapped(Vector3::new(3.0, 0.0, 1.0), a, b)),
        F32_TOL,
    );
    check(
        &m,
        "g3.bary",
        &v3(g3_3d::triangle_get_barycentric_coords(
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.25, 0.5, 0.0),
        )),
        F32_TOL,
    );
    let sph = g3_3d::segment_intersects_sphere(
        Vector3::new(-5.0, 0.0, 0.0),
        Vector3::new(5.0, 0.0, 0.0),
        Vector3::ZERO,
        2.0,
    );
    check(&m, "g3.sphere_hit", &[sph.is_some() as i32 as f64], 0.0);
    let (spt, snrm) = sph.unwrap();
    check(&m, "g3.sphere_pt", &v3(spt), F32_TOL);
    check(&m, "g3.sphere_n", &v3(snrm), F32_TOL);
    check(
        &m,
        "g3.sphere_miss",
        &[g3_3d::segment_intersects_sphere(
            Vector3::new(-5.0, 5.0, 0.0),
            Vector3::new(5.0, 5.0, 0.0),
            Vector3::ZERO,
            2.0,
        )
        .is_some() as i32 as f64],
        0.0,
    );

    check(
        &m,
        "g3.pit_in",
        &[g3_3d::point_in_projected_triangle(
            Vector3::new(0.2, 0.2, 0.0),
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g3.pit_out",
        &[g3_3d::point_in_projected_triangle(
            Vector3::new(1.2, 0.2, 0.0),
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g3.pit_behind",
        &[g3_3d::point_in_projected_triangle(
            Vector3::new(0.2, 0.2, -1.0),
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ) as i32 as f64],
        0.0,
    );

    // Closest points between segments.
    let (ps, qt) = g3_3d::get_closest_points_between_segments(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(4.0, 0.0, 0.0),
        Vector3::new(1.0, 2.0, 0.0),
        Vector3::new(3.0, 2.0, 0.0),
    );
    check(&m, "g3.seg_ps", &v3(ps), F32_TOL);
    check(&m, "g3.seg_qt", &v3(qt), F32_TOL);
    check(
        &m,
        "g3.seg_dist",
        &[g3_3d::get_closest_distance_between_segments(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(4.0, 0.0, 0.0),
            Vector3::new(1.0, 2.0, 0.0),
            Vector3::new(3.0, 2.0, 0.0),
        ) as f64],
        F32_TOL,
    );

    let (pps, pqt) = g3_3d::get_closest_points_between_segments(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(3.0, 0.0, 0.0),
        Vector3::new(4.0, 0.0, 0.0),
    );
    check(&m, "g3.seg_par_ps", &v3(pps), F32_TOL);
    check(&m, "g3.seg_par_qt", &v3(pqt), F32_TOL);

    let (cps, cqt) = g3_3d::get_closest_points_between_segments(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.5, 1.0, 0.0),
        Vector3::new(0.5, 2.0, 0.0),
    );
    check(&m, "g3.seg_cross_ps", &v3(cps), F32_TOL);
    check(&m, "g3.seg_cross_qt", &v3(cqt), F32_TOL);

    // Polygon clipped by the plane x = 0.
    let poly = vec![
        Vector3::new(-2.0, 0.0, -1.0),
        Vector3::new(2.0, 0.0, -1.0),
        Vector3::new(2.0, 0.0, 1.0),
        Vector3::new(-2.0, 0.0, 1.0),
    ];
    let clipped = g3_3d::clip_polygon(&poly, &Plane::new(1.0, 0.0, 0.0, 0.0));
    check(&m, "g3.clip_n", &[clipped.len() as f64], 0.0);
    for (i, v) in clipped.iter().enumerate() {
        check(&m, &format!("g3.clip{i}"), &v3(*v), F32_TOL);
    }
    check(
        &m,
        "g3.clip_all_in",
        &[(g3_3d::clip_polygon(&poly, &Plane::new(0.0, 1.0, 0.0, 1.0)).len() == poly.len()) as i32 as f64],
        0.0,
    );
    check(
        &m,
        "g3.clip_all_out",
        &[(g3_3d::clip_polygon(&poly, &Plane::new(1.0, 0.0, 0.0, 5.0)).len() == 0) as i32 as f64],
        0.0,
    );

    // Triangle in the XY plane, sphere above its interior.
    let tn = (Vector3::new(1.0, 0.0, 0.0) - Vector3::ZERO)
        .cross(Vector3::new(1.0, 0.0, 0.0) - Vector3::new(0.0, 1.0, 0.0))
        .normalized();
    let tri = g3_3d::triangle_sphere_intersection_test(
        Vector3::ZERO,
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        tn,
        Vector3::new(0.3, 0.3, 0.5),
        1.0,
    );
    check(&m, "g3.tri_hit", &[tri.is_some() as i32 as f64], 0.0);
    let (tc, sc) = tri.unwrap();
    check(&m, "g3.tri_tc", &v3(tc), F32_TOL);
    check(&m, "g3.tri_sc", &v3(sc), F32_TOL);
    check(
        &m,
        "g3.tri_miss",
        &[g3_3d::triangle_sphere_intersection_test(
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            tn,
            Vector3::new(5.0, 5.0, 5.0),
            1.0,
        )
        .is_some() as i32 as f64],
        0.0,
    );

    let box_planes = [
        Plane::new(1.0, 0.0, 0.0, 1.0),
        Plane::new(-1.0, 0.0, 0.0, 1.0),
        Plane::new(0.0, 1.0, 0.0, 1.0),
        Plane::new(0.0, -1.0, 0.0, 1.0),
        Plane::new(0.0, 0.0, 1.0, 1.0),
        Plane::new(0.0, 0.0, -1.0, 1.0),
    ];
    let cvx = g3_3d::segment_intersects_convex(
        Vector3::new(-5.0, 0.0, 0.0),
        Vector3::new(5.0, 0.0, 0.0),
        &box_planes,
    );
    check(&m, "g3.convex_hit", &[cvx.is_some() as i32 as f64], 0.0);
    let (vpt, vnrm) = cvx.unwrap();
    check(&m, "g3.convex_pt", &v3(vpt), F32_TOL);
    check(&m, "g3.convex_n", &v3(vnrm), F32_TOL);
    check(
        &m,
        "g3.convex_miss",
        &[g3_3d::segment_intersects_convex(
            Vector3::new(-5.0, 9.0, 0.0),
            Vector3::new(5.0, 9.0, 0.0),
            &box_planes,
        )
        .is_some() as i32 as f64],
        0.0,
    );

    let box_planes_geo = g3_3d::build_box_planes(Vector3::new(1.0, 2.0, 3.0));
    check(&m, "g3.box_n", &[box_planes_geo.len() as f64], 0.0);
    for (i, pl) in box_planes_geo.iter().enumerate() {
        check(&m, &format!("g3.box{i}"), &v3(pl.normal), F32_TOL);
        check(&m, &format!("g3.box_d{i}"), &[pl.d as f64], F32_TOL);
    }

    let cyl_planes = g3_3d::build_cylinder_planes(2.0, 3.0, 4, 1);
    check(&m, "g3.cylp_n", &[cyl_planes.len() as f64], 0.0);
    check(&m, "g3.cylp0", &v3(cyl_planes[0].normal), F32_TOL);
    check(&m, "g3.cylp1", &v3(cyl_planes[1].normal), F32_TOL);
    check(&m, "g3.cylp_d0", &[cyl_planes[0].d as f64], F32_TOL);

    let cap = g3_3d::build_capsule_planes(2.0, 4.0, 4, 1, 1);
    check(&m, "g3.cap_n", &[cap.len() as f64], 0.0);
    check(&m, "g3.cap0", &v3(cap[0].normal), F32_TOL);
    check(&m, "g3.cap1", &v3(cap[1].normal), F32_TOL);
    check(&m, "g3.cap_d1", &[cap[1].d as f64], F32_TOL);

    let boxp = g3_3d::build_box_planes(Vector3::ONE);
    let mesh = g3_3d::compute_convex_mesh_points(&boxp);
    check(&m, "g3.mesh_n", &[mesh.len() as f64], 0.0);
    check(&m, "g3.mesh0", &v3(mesh[0]), F32_TOL);

    let sph_planes = g3_3d::build_sphere_planes(2.0, 2, 4, 1);
    check(&m, "g3.sph_n", &[sph_planes.len() as f64], 0.0);
    check(&m, "g3.sph0", &v3(sph_planes[0].normal), F32_TOL);
    check(&m, "g3.sph1", &v3(sph_planes[1].normal), F32_TOL);
    check(&m, "g3.sph2", &v3(sph_planes[2].normal), F32_TOL);
    check(&m, "g3.sph_d0", &[sph_planes[0].d as f64], F32_TOL);

    let cyl = g3_3d::segment_intersects_cylinder(
        Vector3::new(-5.0, 0.0, 0.0),
        Vector3::new(5.0, 0.0, 0.0),
        4.0,
        2.0,
        1,
    );
    check(&m, "g3.cyl_hit", &[cyl.is_some() as i32 as f64], 0.0);
    let (cpt, cnrm) = cyl.unwrap();
    check(&m, "g3.cyl_pt", &v3(cpt), F32_TOL);
    check(&m, "g3.cyl_n", &v3(cnrm), F32_TOL);
    check(
        &m,
        "g3.cyl_miss",
        &[g3_3d::segment_intersects_cylinder(
            Vector3::new(-5.0, 9.0, 0.0),
            Vector3::new(5.0, 9.0, 0.0),
            4.0,
            2.0,
            1,
        )
        .is_some() as i32 as f64],
        0.0,
    );

    check(
        &m,
        "g3.bary_degenerate",
        &v3(g3_3d::triangle_get_barycentric_coords(
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.25, 0.5, 0.0),
        )),
        F32_TOL,
    );
}

// ---------------------------------------------------------------------------
// Wave 20: named colors (table generated from color_names.inc).
// ---------------------------------------------------------------------------

const CN_FIXTURE: &str = include_str!("fixtures/cpp_colornames.txt");

#[test]
fn named_colors_match_inc() {
    // The C++ side reads `color_names.inc` with an independent parser, so this
    // checks the generator, not just the data being present.
    let mut lines = CN_FIXTURE.lines();
    let count_line = lines.next().unwrap();
    let want_count: usize = count_line.split(',').nth(1).unwrap().trim().parse().unwrap();
    assert_eq!(want_count, Color::named_count());

    let mut seen = 0;
    for line in lines {
        let mut it = line.split(',');
        let name = it.next().unwrap().trim();
        if name.is_empty() {
            continue;
        }
        let hexv = u32::from_str_radix(it.next().unwrap().trim().trim_start_matches("0x"), 16).unwrap();

        let idx = Color::find_named_color(name)
            .unwrap_or_else(|| panic!("missing named color {name}"));
        let (got_name, got_color) = Color::named_at(idx).unwrap();
        assert_eq!(got_name, name);
        assert_eq!(got_color.to_rgba32(), Color::hex(hexv).to_rgba32(), "{name}");
        seen += 1;
    }
    assert_eq!(seen, want_count);
}

#[test]
fn named_color_lookup_normalizes() {
    // Spaces, hyphens, apostrophes and dots are stripped; case is ignored.
    assert_eq!(Color::named("alice blue"), Color::named("ALICE_BLUE"));
    assert_eq!(Color::named("AliceBlue"), Color::named("ALICE_BLUE"));
    assert_eq!(Color::named("red"), Some(Color::new(1.0, 0.0, 0.0, 1.0)));
    assert_eq!(Color::named("not_a_color"), None);
    assert_eq!(
        Color::named_or("not_a_color", Color::new(0.1, 0.2, 0.3, 1.0)),
        Color::new(0.1, 0.2, 0.3, 1.0)
    );
}

// ---------------------------------------------------------------------------
// Wave 21: Variant type tags (Phase 2 scaffolding).
// ---------------------------------------------------------------------------

use godot_core_rs::variant::{Variant, VariantType};

const VT_FIXTURE: &str = include_str!("fixtures/cpp_variant_types.txt");

#[test]
fn variant_type_tags_match_engine() {
    let mut got = std::collections::HashMap::new();
    for line in VT_FIXTURE.lines() {
        let mut it = line.split(',');
        let name = it.next().unwrap().trim();
        if name.is_empty() {
            continue;
        }
        got.insert(name.to_string(), it.next().unwrap().trim().parse::<u32>().unwrap());
    }

    let cases: [(&str, VariantType); 39] = [
        ("vt.NIL", VariantType::Nil),
        ("vt.BOOL", VariantType::Bool),
        ("vt.INT", VariantType::Int),
        ("vt.FLOAT", VariantType::Float),
        ("vt.STRING", VariantType::String),
        ("vt.VECTOR2", VariantType::Vector2),
        ("vt.VECTOR2I", VariantType::Vector2i),
        ("vt.RECT2", VariantType::Rect2),
        ("vt.RECT2I", VariantType::Rect2i),
        ("vt.VECTOR3", VariantType::Vector3),
        ("vt.VECTOR3I", VariantType::Vector3i),
        ("vt.TRANSFORM2D", VariantType::Transform2D),
        ("vt.VECTOR4", VariantType::Vector4),
        ("vt.VECTOR4I", VariantType::Vector4i),
        ("vt.PLANE", VariantType::Plane),
        ("vt.QUATERNION", VariantType::Quaternion),
        ("vt.AABB", VariantType::Aabb),
        ("vt.BASIS", VariantType::Basis),
        ("vt.TRANSFORM3D", VariantType::Transform3D),
        ("vt.PROJECTION", VariantType::Projection),
        ("vt.COLOR", VariantType::Color),
        ("vt.STRING_NAME", VariantType::StringName),
        ("vt.NODE_PATH", VariantType::NodePath),
        ("vt.RID", VariantType::Rid),
        ("vt.OBJECT", VariantType::Object),
        ("vt.CALLABLE", VariantType::Callable),
        ("vt.SIGNAL", VariantType::Signal),
        ("vt.DICTIONARY", VariantType::Dictionary),
        ("vt.ARRAY", VariantType::Array),
        ("vt.PACKED_BYTE_ARRAY", VariantType::PackedByteArray),
        ("vt.PACKED_INT32_ARRAY", VariantType::PackedInt32Array),
        ("vt.PACKED_INT64_ARRAY", VariantType::PackedInt64Array),
        ("vt.PACKED_FLOAT32_ARRAY", VariantType::PackedFloat32Array),
        ("vt.PACKED_FLOAT64_ARRAY", VariantType::PackedFloat64Array),
        ("vt.PACKED_STRING_ARRAY", VariantType::PackedStringArray),
        ("vt.PACKED_VECTOR2_ARRAY", VariantType::PackedVector2Array),
        ("vt.PACKED_VECTOR3_ARRAY", VariantType::PackedVector3Array),
        ("vt.PACKED_COLOR_ARRAY", VariantType::PackedColorArray),
        ("vt.PACKED_VECTOR4_ARRAY", VariantType::PackedVector4Array),
    ];

    for (name, tag) in cases {
        let want = got.get(name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(tag as u32, *want, "{name}: tag mismatch");
    }

    // TYPE_MAX (39) follows the last tag: all 0..=38 are now modelled.
    assert_eq!(got["vt.TYPE_MAX"], VariantType::PackedVector4Array as u32 + 1);
}

#[test]
fn variant_payloads_roundtrip() {
    let values = [
        Variant::Nil,
        Variant::Bool(true),
        Variant::Int(-5),
        Variant::Float(1.5),
        Variant::Vector2(Vector2::new(1.0, 2.0)),
        Variant::Vector3(Vector3::new(1.0, 2.0, 3.0)),
        Variant::Color(Color::new(1.0, 0.0, 0.0, 1.0)),
        Variant::Transform3D(Transform3D::IDENTITY),
        Variant::Basis(Basis::IDENTITY),
    ];

    for v in values {
        // The tag must agree with the payload, and cloning must preserve both.
        let c = v.clone();
        assert_eq!(v.vtype(), c.vtype());
        assert_eq!(v, c);
    }
}
