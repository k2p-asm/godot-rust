// Differential test harness: compiles Godot's own math sources and prints the
// values of a fixed set of expressions. The Rust port is checked against this
// output by `cargo test --test parity`.
//
// Build (from the repository root):
//   call "<VS>\Common7\Tools\VsDevCmd.bat" -arch=x64
//   cl /nologo /std:c++20 /Zc:__cplusplus /EHsc /O2 /I . /I platform\windows ^
//      modules\rustcore\parity\dump.cpp core\math\vector2.cpp core\math\vector3.cpp ^
//      core\math\basis.cpp core\math\quaternion.cpp core\math\math_funcs.cpp ^
//      core\math\random_pcg.cpp modules\rustcore\parity\os_stub.cpp

#include "core/math/vector2.h"
#include "core/math/vector2i.h"
#include "core/math/vector3i.h"
#include "core/math/vector4.h"
#include "core/math/vector4i.h"
#include "core/math/face3.h"
#include "core/math/geometry_3d.h"
#include "core/math/geometry_2d.h"
#include "core/math/vector3.h"
#include "core/math/aabb.h"
#include "core/math/color.h"
#include "core/math/basis.h"
#include "core/math/rect2.h"
#include "core/math/rect2i.h"
#include "core/math/projection.h"
#include "core/math/transform_2d.h"
#include "core/math/transform_3d.h"
#include "core/math/math_funcs.h"
#include "core/math/random_pcg.h"
#include "thirdparty/misc/pcg.h"

#include <cstdio>

static void out(const char *name, const Vector2 &v) {
	printf("%s,%.9g,%.9g\n", name, (double)v.x, (double)v.y);
}

static void out(const char *name, const Vector3i &v) {
	printf("%s,%d,%d,%d\n", name, v.x, v.y, v.z);
}

static void out(const char *name, const Vector2i &v) {
	printf("%s,%d,%d\n", name, v.x, v.y);
}

static void outi(const char *name, long long v) {
	printf("%s,%lld\n", name, v);
}

static void out(const char *name, const Color &v) {
	printf("%s,%.9g,%.9g,%.9g,%.9g\n", name, (double)v.r, (double)v.g, (double)v.b, (double)v.a);
}

static void out(const char *name, const Vector3 &v) {
	printf("%s,%.9g,%.9g,%.9g\n", name, (double)v.x, (double)v.y, (double)v.z);
}

static void outf(const char *name, double v) {
	printf("%s,%.17g\n", name, v);
}

static void outu(const char *name, unsigned v) {
	printf("%s,%u\n", name, v);
}

int main() {
	const Vector2 a2(1.5f, -2.25f);
	const Vector2 b2(-0.75f, 4.5f);
	const Vector3 a3(1.5f, -2.25f, 0.75f);
	const Vector3 b3(-0.75f, 4.5f, 2.125f);
	const Vector3 axis(0.36f, 0.48f, 0.8f); // normalized
	const Vector3 axis2 = Vector3(1.0f, 1.0f, 0.0f).normalized();

	// --- Vector2
	out("v2.sum", a2 + b2);
	out("v2.sub", a2 - b2);
	out("v2.mul", a2 * b2);
	out("v2.div", a2 / b2);
	out("v2.length", Vector2(a2.length(), a2.length_squared()));
	outf("v2.len_a2", a2.length());
	outf("v2.dot_a2b2", a2.dot(b2));
	outf("v2.cross_a2b2", a2.cross(b2));
	outf("v2.angle_a2", a2.angle());
	outf("v2.angle_to_a2b2", a2.angle_to(b2));
	out("v2.normalized", a2.normalized());
	out("v2.rotated", a2.rotated(0.7f));
	out("v2.slide", a2.slide(Vector2(0.0f, 1.0f)));
	out("v2.bounce", a2.bounce(Vector2(0.0f, 1.0f)));
	out("v2.reflect", a2.reflect(Vector2(0.0f, 1.0f)));
	out("v2.lerp", a2.lerp(b2, 0.35f));
	out("v2.slerp", a2.slerp(b2, 0.35f));
	out("v2.move_toward", a2.move_toward(b2, 0.75f));
	out("v2.posmod", a2.posmod(1.25f));
	out("v2.project", a2.project(b2));
	out("v2.snapped", a2.snapped(Vector2(0.5f, 0.25f)));
	out("v2.limit_length", a2.limit_length(1.0f));
	out("v2.direction_to", a2.direction_to(b2));
	outu("v2.hash", a2.hash());

	// --- Vector3
	out("v3.sum", a3 + b3);
	out("v3.sub", a3 - b3);
	out("v3.cross", a3.cross(b3));
	outf("v3.dot", a3.dot(b3));
	outf("v3.length", a3.length());
	outf("v3.angle_to", a3.angle_to(b3));
	outf("v3.signed_angle_to", a3.signed_angle_to(b3, Vector3(0, 1, 0)));
	out("v3.normalized", a3.normalized());
	out("v3.rotated_axis", a3.rotated(axis, 0.7f));
	out("v3.rotated_axis2", a3.rotated(axis2, 1.234f));
	out("v3.slide", a3.slide(Vector3(0, 1, 0)));
	out("v3.bounce", a3.bounce(Vector3(0, 1, 0)));
	out("v3.reflect", a3.reflect(Vector3(0, 1, 0)));
	out("v3.lerp", a3.lerp(b3, 0.35f));
	out("v3.slerp", a3.slerp(b3, 0.35f));
	out("v3.move_toward", a3.move_toward(b3, 0.75f));
	out("v3.posmod", a3.posmod(1.25f));
	out("v3.project", a3.project(b3));
	out("v3.snapped", a3.snapped(Vector3(0.5f, 0.25f, 0.125f)));
	out("v3.octahedron_encode", a3.normalized().octahedron_encode());
	out("v3.octahedron_decode", Vector3::octahedron_decode(Vector2(0.31f, 0.82f)));
	out("v3.any_perp", a3.get_any_perpendicular());
	outu("v3.hash", a3.hash());
	outu("v3.min_axis", (unsigned)a3.min_axis_index());
	outu("v3.max_axis", (unsigned)a3.max_axis_index());

	// --- scalar math
	outf("m.angle_difference", Math::angle_difference(0.3, -2.1));
	outf("m.lerp_angle", Math::lerp_angle(0.3, -2.1, 0.4));
	outf("m.wrapf", Math::wrapf(-2.75, 0.5, 3.5));
	outf("m.snapped", Math::snapped(1.26, 0.5));
	outf("m.ease_pos", Math::ease(0.37, 0.6));
	outf("m.ease_neg", Math::ease(0.62, -0.4));
	outf("m.smoothstep", Math::smoothstep(0.0, 1.0, 0.37));
	outf("m.bezier", Math::bezier_interpolate(0.0, 0.2, 0.8, 1.0, 0.4));
	outf("m.bezier_deriv", Math::bezier_derivative(0.0, 0.2, 0.8, 1.0, 0.4));
	outf("m.cubic", Math::cubic_interpolate(0.1, 0.9, -0.2, 1.4, 0.33));
	outf("m.cubic_time", Math::cubic_interpolate_in_time(0.1, 0.9, -0.2, 1.4, 0.33, 1.0, -1.0, 2.0));
	outf("m.db_to_linear", Math::db_to_linear(-6.0));
	outf("m.linear_to_db", Math::linear_to_db(0.5));
	outf("m.remap", Math::remap(0.3, 0.0, 1.0, -5.0, 5.0));
	outf("m.pingpong", Math::pingpong(3.4, 2.0));
	outu("m.larger_prime", Math::larger_prime(200));
	outu("m.step_decimals", (unsigned)Math::step_decimals(0.01));

	// --- Basis / Quaternion
	{
		const Vector3 v(0.4f, -1.2f, 2.5f);
		const Basis b = Basis::from_euler(Vector3(0.3f, -0.7f, 1.1f));
		out("basis.xform", b.xform(v));
		out("basis.xform_inv", b.xform_inv(v));
		outf("basis.det", b.determinant());
		out("basis.inverse_xform", b.inverse().xform(v));
		out("basis.transposed_col0", b.transposed().get_column(0));
		out("basis.scale", b.get_scale());
		out("basis.scale_abs", b.get_scale_abs());
		outf("basis.uniform", b.get_uniform_scale());

		const Basis m(Vector3(1, 2, 3), Vector3(4, 5, 6), Vector3(7, 8, 10));
		outf("basis.det_m", m.determinant());
		out("basis.orthonorm_col0", m.orthonormalized().get_column(0));
		out("basis.orthonorm_col2", m.orthonormalized().get_column(2));

		const Basis r(Vector3(0.36f, 0.48f, 0.8f), 0.77f);
		out("basis.axis_angle", r.xform(Vector3(1.0f, 2.0f, 3.0f)));
		out("basis.mul", (r * b).xform(v));
		out("basis.scaled_local", b.scaled_local(Vector3(3.0f, 2.0f, 0.5f)).get_column(1));

		const Quaternion q = b.get_quaternion();
		outf("quat.x", q.x);
		outf("quat.y", q.y);
		outf("quat.z", q.z);
		outf("quat.w", q.w);
		out("quat.xform", q.xform(v));
		const Quaternion q2(Vector3(0.36f, 0.48f, 0.8f), 0.77f);
		out("quat.axis_angle", q2.xform(v));
		out("quat.slerp", q.slerp(q2, 0.35f).xform(v));
		outf("quat.angle_to", q.angle_to(q2));
		out("quat.euler", Quaternion::from_euler(Vector3(0.3f, -0.7f, 1.1f)).xform(v));
		out("basis.slerp", b.slerp(b.scaled_local(Vector3(2, 2, 2)), 0.5f).get_column(0));
	}

	// --- Transform3D
	{
		const Vector3 v(0.4f, -1.2f, 2.5f);
		const Transform3D t(Basis::from_euler(Vector3(0.2f, 0.6f, -0.4f)), Vector3(1.0f, -2.0f, 3.0f));
		const Transform3D t2(Basis(Vector3(0.36f, 0.48f, 0.8f), 0.5f), Vector3(-1.0f, 0.5f, 2.0f));
		out("t3.xform", t.xform(v));
		out("t3.xform_inv", t.xform_inv(t.xform(v)));
		out("t3.affine_xform_inv", t.affine_inverse().xform(t.xform(v)));
		out("t3.mul", (t * t2).xform(v));
		out("t3.rotated", t.rotated(Vector3(0, 1, 0), 0.3f).xform(v));
		out("t3.translated", t.translated(Vector3(1.0f, 2.0f, 3.0f)).xform(v));
		const AABB box(Vector3(-1.0f, -2.0f, -3.0f), Vector3(2.0f, 4.0f, 6.0f));
		out("t3.aabb_pos", t.xform(box).position);
		out("t3.aabb_size", t.xform(box).size);
		out("t3.plane_n", t.xform(Plane(Vector3(0.0f, 1.0f, 0.0f), 0.5f)).normal);
		outf("t3.plane_d", t.xform(Plane(Vector3(0.0f, 1.0f, 0.0f), 0.5f)).d);
		out("t3.interp", t.interpolate_with(t2, 0.35f).xform(v));
		out("t3.scale_origin", t.scaled(Vector3(2.0f, 0.5f, 1.5f)).origin);
		Vector<Vector3> batch;
		batch.push_back(Vector3(0.5f, 1.5f, -0.5f));
		batch.push_back(Vector3(-1.0f, 2.0f, 0.25f));
		Vector<Vector3> batch_out = t.xform(batch);
		Vector<Vector3> batch_inv = t.xform_inv(batch);
		for (int i = 0; i < batch.size(); i++) {
			char lbl[32]; snprintf(lbl, sizeof(lbl), "t3.batch%d", i);
			out(lbl, batch_out[i]);
			snprintf(lbl, sizeof(lbl), "t3.batch_inv%d", i);
			out(lbl, batch_inv[i]);
		}
	}

	// --- Euler decomposition (both round-trip and arbitrary rotation)
	{
		const Vector3 e(0.3f, -0.7f, 1.1f);
		const Vector3 c[] = { e, Vector3(0.0f, 0.0f, 0.0f), Vector3(1.2f, -0.4f, 2.3f) };
		const EulerOrder orders[] = { EulerOrder::XYZ, EulerOrder::XZY, EulerOrder::YXZ,
			EulerOrder::YZX, EulerOrder::ZXY, EulerOrder::ZYX };
		const char *names[] = { "xyz", "xzy", "yxz", "yzx", "zxy", "zyx" };
		for (int i = 0; i < 6; i++) {
			char label[64];
			snprintf(label, sizeof(label), "euler.%s", names[i]);
			out(label, Basis::from_euler(e, orders[i]).get_euler(orders[i]));
			snprintf(label, sizeof(label), "euler_round.%s", names[i]);
			const Basis mixed = Basis::from_euler(c[1], orders[i]).rotated(Vector3(0.3f, 0.8f, -0.5f).normalized(), 1.1f);
			out(label, mixed.get_euler(orders[i]));
		}
		const Basis arbitrary = Basis(Vector3(0.36f, 0.48f, 0.8f), 0.77f) * Basis(Vector3(0.0f, 1.0f, 0.0f), 0.4f);
		for (int i = 0; i < 6; i++) {
			char label[64];
			snprintf(label, sizeof(label), "euler_arb.%s", names[i]);
			out(label, arbitrary.get_euler(orders[i]));
		}
	}

	// --- Integer vectors
	{
		const Vector3i a(3, -4, 12);
		const Vector3i b(-1, 5, 2);
		out("v3i.sum", a + b);
		out("v3i.min", a.min(b));
		out("v3i.max", a.max(b));
		out("v3i.abs", a.abs());
		out("v3i.sign", a.sign());
		out("v3i.clamp", Vector3i(9, -9, 4).clamp(Vector3i(-5, -5, -5), Vector3i(5, 5, 5)));
		outi("v3i.len_sq", a.length_squared());
		outf("v3i.len", a.length());
		outu("v3i.hash", a.hash());
		outu("v3i.min_axis", (unsigned)a.min_axis_index());
		out("v3i.snap", Vector3i(-5, 3, 17).snappedi(4));
		const Vector2i c(7, -9);
		out("v2i.mul", c * Vector2i(2, 3));
		outi("v2i.len_sq", c.length_squared());
		outu("v2i.hash", c.hash());
		out("v2i.snap", Vector2i(-5, 3).snappedi(4));
	}

	// --- Axis-angle extraction
	{
		Vector3 ax;
		real_t ang;
		Basis().get_axis_angle(ax, ang);
		out("axis.id_axis", ax);
		outf("axis.id_angle", ang);
		Basis(Vector3(0.0f, 1.0f, 0.0f), Math::PI).get_axis_angle(ax, ang);
		out("axis.180_axis", ax);
		outf("axis.180_angle", ang);
		Basis(Vector3(0.36f, 0.48f, 0.8f), 0.77f).get_axis_angle(ax, ang);
		out("axis.gen_axis", ax);
		outf("axis.gen_angle", ang);
		Basis(Vector3(0.36f, 0.48f, 0.8f), 2.9f).get_axis_angle(ax, ang);
		out("axis.gen2_axis", ax);
		outf("axis.gen2_angle", ang);
	}

	// --- Filled polygon vs rect
	{
		const Rect2 rect(0.0f, 0.0f, 10.0f, 10.0f);
		Vector2 tri[3] = { Vector2(20.0f, 20.0f), Vector2(30.0f, 20.0f), Vector2(20.0f, 30.0f) };
		outu("poly.far", rect.intersects_filled_polygon(tri, 3) ? 1 : 0);
		Vector2 tri2[3] = { Vector2(5.0f, 5.0f), Vector2(15.0f, 5.0f), Vector2(5.0f, 15.0f) };
		outu("poly.overlap", rect.intersects_filled_polygon(tri2, 3) ? 1 : 0);
		Vector2 tri3[3] = { Vector2(2.0f, 2.0f), Vector2(8.0f, 2.0f), Vector2(2.0f, 8.0f) };
		outu("poly.inside", rect.intersects_filled_polygon(tri3, 3) ? 1 : 0);
	}

	// --- Vector4
	{
		const Vector4 a(1.5f, -2.25f, 0.75f, 4.0f);
		const Vector4 b(-0.5f, 3.0f, 2.25f, -1.5f);
		out("v4.sum", Vector2(a.x, a.y)); // placeholder to keep arity predictable
		printf("v4.sum4,%.9g,%.9g,%.9g,%.9g\n", (double)(a + b).x, (double)(a + b).y, (double)(a + b).z, (double)(a + b).w);
		printf("v4.mul4,%.9g,%.9g,%.9g,%.9g\n", (double)(a * b).x, (double)(a * b).y, (double)(a * b).z, (double)(a * b).w);
		outf("v4.dot", a.dot(b));
		outf("v4.len", a.length());
		printf("v4.norm4,%.9g,%.9g,%.9g,%.9g\n", (double)a.normalized().x, (double)a.normalized().y, (double)a.normalized().z, (double)a.normalized().w);
		printf("v4.snap4,%.9g,%.9g,%.9g,%.9g\n", (double)a.snapped(Vector4(0.5f, 0.25f, 0.125f, 1.0f)).x, (double)a.snapped(Vector4(0.5f, 0.25f, 0.125f, 1.0f)).y, (double)a.snapped(Vector4(0.5f, 0.25f, 0.125f, 1.0f)).z, (double)a.snapped(Vector4(0.5f, 0.25f, 0.125f, 1.0f)).w);
		printf("v4.posmod4,%.9g,%.9g,%.9g,%.9g\n", (double)a.posmod(1.25f).x, (double)a.posmod(1.25f).y, (double)a.posmod(1.25f).z, (double)a.posmod(1.25f).w);
		outu("v4.hash", a.hash());
		outu("v4.min_axis", (unsigned)a.min_axis_index());
		outu("v4.max_axis", (unsigned)a.max_axis_index());
	}

	// --- Transform2D
	{
		const Vector2 v(1.5f, -0.25f);
		const Transform2D t(0.7f, Vector2(2.0f, 3.0f));
		const Transform2D s(0.7f, Size2(2.0f, 0.5f), 0.1f, Vector2(1.0f, -2.0f));
		out("t2.xform", t.xform(v));
		outf("t2.det", t.determinant());
		out("t2.affine_inv", s.affine_inverse().xform(s.xform(v)));
		outf("t2.rot", s.get_rotation());
		out("t2.scale", s.get_scale());
		outf("t2.skew", s.get_skew());
		out("t2.mul", (t * s).xform(Vector2(0.3f, 0.9f)));
		out("t2.origin", s.get_origin());
		out("t2.interp", s.interpolate_with(t, 0.35f).xform(Vector2(0.3f, 0.9f)));
	}

	// --- Rect2i
	{
		const Rect2i ri(1, 2, 4, 3);
		const Rect2i ro(-1, 0, 3, 4);
		const Vector2i ci = ri.get_center();
		printf("ri.center,%d,%d\n", ci.x, ci.y);
		const Rect2i ii = ri.intersection(ro);
		printf("ri.inter_pos,%d,%d\n", ii.position.x, ii.position.y);
		printf("ri.inter_size,%d,%d\n", ii.size.x, ii.size.y);
		const Rect2i mi = ri.merge(ro);
		printf("ri.merge_pos,%d,%d\n", mi.position.x, mi.position.y);
		printf("ri.merge_size,%d,%d\n", mi.size.x, mi.size.y);
		outu("ri.area", (unsigned)ri.get_area());
		outu("ri.hash", ri.hash());
		outu("ri.intersects", ri.intersects(ro) ? 1 : 0);
		const Rect2i gr = ri.grow(2);
		printf("ri.grow_pos,%d,%d\n", gr.position.x, gr.position.y);
		printf("ri.grow_size,%d,%d\n", gr.size.x, gr.size.y);
	}

	// --- Geometry2D hit testing
	{
		Vector2 res;
		const bool hit = Geometry2D::segment_intersects_segment(Vector2(-1, 0), Vector2(1, 0), Vector2(0, -1), Vector2(0, 1), &res);
		printf("g2d.seg_hit,%d\n", hit ? 1 : 0);
		printf("g2d.seg_pt,%.9g,%.9g\n", (double)res.x, (double)res.y);
		const bool par = Geometry2D::segment_intersects_segment(Vector2(0, 0), Vector2(1, 0), Vector2(0, 1), Vector2(1, 1), &res);
		printf("g2d.par,%d\n", par ? 1 : 0);
		const bool outside = Geometry2D::segment_intersects_segment(Vector2(0, 0), Vector2(1, 0), Vector2(2, -1), Vector2(2, 1), &res);
		printf("g2d.outside,%d\n", outside ? 1 : 0);
		outu("g2d.circle_in", Geometry2D::is_point_in_circle(Vector2(1, 0), Vector2(0, 0), 1.0f) ? 1 : 0);
		outu("g2d.circle_out", Geometry2D::is_point_in_circle(Vector2(1.5f, 0), Vector2(0, 0), 1.0f) ? 1 : 0);
		Vector<Vector2> square;
		square.push_back(Vector2(0, 0));
		square.push_back(Vector2(4, 0));
		square.push_back(Vector2(4, 4));
		square.push_back(Vector2(0, 4));
		outu("g2d.poly_in", Geometry2D::is_point_in_polygon(Vector2(2, 2), square) ? 1 : 0);
		outu("g2d.poly_out", Geometry2D::is_point_in_polygon(Vector2(5, 2), square) ? 1 : 0);
		outu("g2d.seg_poly", Geometry2D::is_segment_intersecting_polygon(Vector2(-1, 2), Vector2(5, 2), square) ? 1 : 0);
		Vector<Vector2> pts;
		pts.push_back(Vector2(0, 0)); pts.push_back(Vector2(2, 0)); pts.push_back(Vector2(2, 2));
		pts.push_back(Vector2(1, 0.5f)); pts.push_back(Vector2(0, 2)); pts.push_back(Vector2(1, 1));
		Vector<Vector2> hull = Geometry2D::convex_hull(pts);
		printf("g2d.hull_n,%d\n", hull.size());
		for (int i = 0; i < hull.size(); i++) {
			char lbl[32]; snprintf(lbl, sizeof(lbl), "g2d.hull%d", i);
			out(lbl, hull[i]);
		}
		Vector<Vector2i> line = Geometry2D::bresenham_line(Vector2i(0, 0), Vector2i(5, 2));
		printf("g2d.line_n,%d\n", line.size());
		for (int i = 0; i < line.size(); i++) {
			char lbl[32]; snprintf(lbl, sizeof(lbl), "g2d.line%d", i);
			printf("%s,%d,%d\n", lbl, line[i].x, line[i].y);
		}
	}

	// --- Projection
	{
		const Projection persp = Projection::create_perspective(75.0f, 16.0f / 9.0f, 0.05f, 1000.0f, false);
		printf("proj.xform4,%.9g,%.9g,%.9g,%.9g\n", (double)persp.xform(Vector4(1, 2, -10, 1)).x, (double)persp.xform(Vector4(1, 2, -10, 1)).y, (double)persp.xform(Vector4(1, 2, -10, 1)).z, (double)persp.xform(Vector4(1, 2, -10, 1)).w);
		out("proj.xform3", persp.xform(Vector3(1, 2, -10)));
		outf("proj.det", persp.determinant());
		outf("proj.znear", persp.get_z_near());
		outf("proj.zfar", persp.get_z_far());
		outf("proj.fov", persp.get_fov());
		outf("proj.aspect", persp.get_aspect());
		outu("proj.ortho", persp.is_orthogonal() ? 1 : 0);
		const Projection ortho = Projection::create_orthogonal(-2.0f, 2.0f, -1.0f, 1.0f, 0.1f, 100.0f);
		out("ortho.xform3", ortho.xform(Vector3(1.0f, 0.5f, -10.0f)));
		outu("ortho.is_ortho", ortho.is_orthogonal() ? 1 : 0);
		const Projection fr = Projection::create_frustum(-1.0f, 1.0f, -0.5f, 0.5f, 0.1f, 100.0f);
		out("frustum.xform3", fr.xform(Vector3(0.5f, 0.2f, -5.0f)));
		const Projection pinv = persp.inverse();
		printf("proj.inv4,%.9g,%.9g,%.9g,%.9g\n", (double)pinv.xform(Vector4(0.3f, -0.7f, -5.0f, 1.0f)).x, (double)pinv.xform(Vector4(0.3f, -0.7f, -5.0f, 1.0f)).y, (double)pinv.xform(Vector4(0.3f, -0.7f, -5.0f, 1.0f)).z, (double)pinv.xform(Vector4(0.3f, -0.7f, -5.0f, 1.0f)).w);
		for (int i = 0; i < 6; i++) {
			const Plane pp = persp.get_projection_plane((Projection::Planes)i);
			char label[32];
			snprintf(label, sizeof(label), "proj.plane%d", i);
			out(label, pp.normal);
			snprintf(label, sizeof(label), "proj.plane_d%d", i);
			outf(label, pp.d);
		}
		const Projection hmdL = Projection::create_for_hmd(1, 1.7777778f, 0.064f, 0.14f, 0.045f, 1.2f, 0.05f, 1000.0f);
		const Projection hmdR = Projection::create_for_hmd(2, 1.7777778f, 0.064f, 0.14f, 0.045f, 1.2f, 0.05f, 1000.0f);
		const Projection hmdM = Projection::create_for_hmd(0, 1.7777778f, 0.064f, 0.14f, 0.045f, 1.2f, 0.05f, 1000.0f);
		const Vector2 vp = persp.get_viewport_half_extents();
		out("proj.vp_half", vp);
		out("proj.far_half", persp.get_far_plane_half_extents());
		outu("proj.px_m", (unsigned)persp.get_pixels_per_meter(1920));
		outf("proj.lod_mul", persp.get_lod_multiplier());
		Projection scaled_p;
		scaled_p.make_scale(Vector3(2.0f, 0.5f, 3.0f));
		out("proj.scale_col0", Vector3(scaled_p[0][0], scaled_p[0][1], scaled_p[0][2]));
		out("proj.scale_origin", Vector3(scaled_p[3][0], scaled_p[3][1], scaled_p[3][2]));
		Projection adj = persp;
		adj.adjust_perspective_znear(0.25f);
		outf("adj.znear", adj.get_z_near());
		outf("adj.zfar", adj.get_z_far());
		out("adj.xform", adj.xform(Vector3(0.3f, 0.2f, -3.0f)));
		out("hmd.left_pt", hmdL.xform(Vector3(0.1f, 0.05f, -2.0f)));
		out("hmd.right_pt", hmdR.xform(Vector3(0.1f, 0.05f, -2.0f)));
		outu("hmd.mono_identity", hmdM == Projection() ? 1 : 0);
		Projection fit;
		fit.scale_translate_to_fit(AABB(Vector3(-2.0f, -1.0f, 3.0f), Vector3(4.0f, 2.0f, 6.0f)));
		out("proj.fit_min", fit.xform(Vector3(-2.0f, -1.0f, 3.0f)));
		out("proj.fit_max", fit.xform(Vector3(2.0f, 1.0f, 9.0f)));
		out("proj.fit_mid", fit.xform(Vector3(0.0f, 0.0f, 6.0f)));
		const Transform3D view(Basis(Vector3(0.36f, 0.48f, 0.8f), 0.4f), Vector3(3.0f, -2.0f, 7.0f));
		Vector<Plane> wplanes = persp.get_projection_planes(view);
		for (int i = 0; i < 6; i++) {
			char lbl[32]; snprintf(lbl, sizeof(lbl), "proj.wplane%d", i);
			out(lbl, wplanes[i].normal);
			snprintf(lbl, sizeof(lbl), "proj.wplane_d%d", i);
			outf(lbl, wplanes[i].d);
		}
		const Projection fit2 = Projection::create_fit_aabb(AABB(Vector3(-1.0f, -2.0f, 0.5f), Vector3(2.0f, 4.0f, 1.0f)));
		out("proj.fit2_min", fit2.xform(Vector3(-1.0f, -2.0f, 0.5f)));
		out("proj.fit2_max", fit2.xform(Vector3(1.0f, 2.0f, 1.5f)));
		const Projection zn = persp.perspective_znear_adjusted(0.75f);
		outf("proj.zn_near", zn.get_z_near());
		out("proj.zn_pt", zn.xform(Vector3(0.4f, -0.3f, -6.0f)));
		const Projection asym = Projection::create_frustum(-2.0f, 1.0f, -1.0f, 1.5f, 0.1f, 100.0f);
		outf("proj.asym_fov", asym.get_fov());
		outf("proj.asym_aspect", asym.get_aspect());
		out("proj.asym_pt", asym.xform(Vector3(0.0f, 0.0f, -5.0f)));
		const Projection fy = persp.flipped_y();
		printf("proj.flip_col1,%.9g,%.9g,%.9g,%.9g\n", (double)fy[1][0], (double)fy[1][1], (double)fy[1][2], (double)fy[1][3]);
	}

	// --- Basis::diagonalize (symmetric matrix)
	{
		Basis sym(Vector3(2.0f, 0.5f, 0.25f), Vector3(0.5f, 3.0f, 0.75f), Vector3(0.25f, 0.75f, 5.0f));
		const Basis acc = sym.diagonalize();
		out("diag.col0", sym.get_column(0));
		out("diag.col1", sym.get_column(1));
		out("diag.col2", sym.get_column(2));
		out("diag.rot", acc.get_column(0));
		out("diag.rot1", acc.get_column(1));
		Basis rotref;
		const Vector3 ps = Basis(Vector3(0.36f, 0.48f, 0.8f), 0.77f).scaled(Vector3(2.0f, 0.5f, 1.5f)).rotref_posscale_decomposition(rotref);
		out("rr.scale", ps);
		out("rr.rot0", rotref.get_column(0));
		out("rr.rot2", rotref.get_column(2));
		printf("diag.off,%.9g\n", (double)(Math::abs(sym.rows[0][1]) + Math::abs(sym.rows[0][2]) + Math::abs(sym.rows[1][2])));
	}

	// --- Basis::rotate_sh
	{
		Basis b = Basis::from_euler(Vector3(0.3f, -0.7f, 1.1f));
		real_t sh[9] = { 0.5f, -0.25f, 0.75f, 1.0f, -0.5f, 0.125f, 0.3f, -0.8f, 0.6f };
		b.rotate_sh(sh);
		printf("sh,");
		for (int i = 0; i < 9; i++) {
			printf("%.9g%s", (double)sh[i], i < 8 ? "," : "\n");
		}
	}

	// --- Vector4i
	{
		const Vector4i a(1, -2, 3, -4);
		const Vector4i b(5, 6, -7, 8);
		const Vector4i s = a + b;
		printf("v4i.sum,%d,%d,%d,%d\n", s.x, s.y, s.z, s.w);
		const Vector4i mn = a.min(b);
		printf("v4i.min,%d,%d,%d,%d\n", mn.x, mn.y, mn.z, mn.w);
		const Vector4i ab = a.abs();
		printf("v4i.abs,%d,%d,%d,%d\n", ab.x, ab.y, ab.z, ab.w);
		outi("v4i.len_sq", a.length_squared());
		outi("v4i.sum64", (long long)a.x * b.x + (long long)a.y * b.y + (long long)a.z * b.z + (long long)a.w * b.w);
		outu("v4i.hash", a.hash());
		const Vector4i sn = Vector4i(-5, 3, 17, -1).snappedi(4);
		printf("v4i.snap,%d,%d,%d,%d\n", sn.x, sn.y, sn.z, sn.w);
		outi("v4i.min_axis", a.min_axis_index());
		outi("v4i.max_axis", a.max_axis_index());
	}

	// --- Face3
	{
		const Face3 f(Vector3(0.0f, -1.0f, 0.0f), Vector3(2.0f, -1.0f, 0.0f), Vector3(0.0f, 1.0f, 0.0f));
		outf("f3.area", f.get_area());
		outu("f3.degenerate", f.is_degenerate() ? 1 : 0);
		out("f3.plane_n", f.get_plane().normal);
		outf("f3.plane_d", f.get_plane().d);
		out("f3.aabb_pos", f.get_aabb().position);
		out("f3.aabb_size", f.get_aabb().size);
		Face3 res[3];
		bool over[3];
		const int n = f.split_by_plane(Plane(Vector3(0.0f, 1.0f, 0.0f), 0.0f), res, over);
		outu("f3.split_count", (unsigned)n);
		outf("f3.split_area0", n > 0 ? res[0].get_area() : 0.0f);
		outf("f3.split_area1", n > 1 ? res[1].get_area() : 0.0);
		out("f3.closest", f.get_closest_point_to(Vector3(0.6f, -0.2f, 1.5f)));
		out("f3.closest_in", f.get_closest_point_to(Vector3(0.3f, -0.5f, 0.0f)));
		outu("f3.aabb_hit", f.intersects_aabb(AABB(Vector3(-0.5f, -0.5f, -0.5f), Vector3(1.0f, 1.0f, 1.0f))) ? 1 : 0);
		outu("f3.aabb_miss", f.intersects_aabb(AABB(Vector3(5.0f, 5.0f, 5.0f), Vector3(1.0f, 1.0f, 1.0f))) ? 1 : 0);
		Vector3 hitp;
		const bool rh = f.intersects_ray(Vector3(0.0f, 0.0f, -5.0f), Vector3(0.0f, 0.0f, 1.0f), &hitp);
		outu("f3.ray_hit", rh ? 1 : 0);
		out("f3.ray_pt", hitp);
		const bool sh = f.intersects_segment(Vector3(0.0f, 0.0f, -1.0f), Vector3(0.0f, 0.0f, 1.0f), &hitp);
		outu("f3.seg_hit", sh ? 1 : 0);
		out("f3.seg_pt", hitp);
		Vector3 sup[3]; int sup_n = 0;
		const Transform3D identity_t;
		const Transform3D xf_t(Basis(Vector3(0.0f, 1.0f, 0.0f), 0.5f), Vector3(1.0f, 2.0f, 3.0f));
		f.get_support(Vector3(0, 0, 1), identity_t, sup, &sup_n, 3);
		outu("f3.sup_face_n", (unsigned)sup_n);
		out("f3.sup_face0", sup[0]);
		f.get_support(Vector3(0, 1, 0), identity_t, sup, &sup_n, 3);
		outu("f3.sup_vertex_n", (unsigned)sup_n);
		out("f3.sup_vertex0", sup[0]);
		f.get_support(Vector3(0, 0, 1), xf_t, sup, &sup_n, 3);
		real_t pr_min, pr_max;
		f.project_range(Vector3(0, 1, 0), xf_t, pr_min, pr_max);
		outf("f3.pr_min", pr_min);
		outf("f3.pr_max", pr_max);
		outu("f3.sup_xf_n", (unsigned)sup_n);
		out("f3.sup_xf0", sup[0]);
		out("f3.sup_xf1", sup[1]);
		out("f3.sup_xf2", sup[2]);
		outu("f3.ray_miss", f.intersects_ray(Vector3(0.0f, 0.0f, -5.0f), Vector3(0.0f, 0.0f, -1.0f), &hitp) ? 1 : 0);
	}

	// --- Geometry3D helpers
	{
		const Vector3 a(0.0f, 0.0f, 0.0f);
		const Vector3 b(2.0f, 2.0f, 2.0f);
		out("g3.closest_seg", Geometry3D::get_closest_point_to_segment(Vector3(3.0f, 0.0f, 1.0f), a, b));
		out("g3.closest_seg_before", Geometry3D::get_closest_point_to_segment(Vector3(-3.0f, -3.0f, -3.0f), a, b));
		out("g3.closest_seg_unc", Geometry3D::get_closest_point_to_segment_uncapped(Vector3(3.0f, 0.0f, 1.0f), a, b));
		out("g3.bary", Geometry3D::triangle_get_barycentric_coords(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0), Vector3(0.25f, 0.5f, 0.0f)));
		const Vector3 bd = Geometry3D::triangle_get_barycentric_coords(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(1, 0, 0), Vector3(0.25f, 0.5f, 0.0f));
		out("g3.bary_degenerate", bd);
		Vector3 sp, sn;
		const bool sh = Geometry3D::segment_intersects_sphere(Vector3(-5, 0, 0), Vector3(5, 0, 0), Vector3(0, 0, 0), 2.0f, &sp, &sn);
		outu("g3.sphere_hit", sh ? 1 : 0);
		out("g3.sphere_pt", sp);
		out("g3.sphere_n", sn);
		outu("g3.sphere_miss", Geometry3D::segment_intersects_sphere(Vector3(-5, 5, 0), Vector3(5, 5, 0), Vector3(0, 0, 0), 2.0f, &sp, &sn) ? 1 : 0);
		Vector3 cp, cn;
		const bool ch = Geometry3D::segment_intersects_cylinder(Vector3(-5, 0, 0), Vector3(5, 0, 0), 4.0f, 2.0f, &cp, &cn, 1);
		outu("g3.cyl_hit", ch ? 1 : 0);
		out("g3.cyl_pt", cp);
		out("g3.cyl_n", cn);
		outu("g3.cyl_miss", Geometry3D::segment_intersects_cylinder(Vector3(-5, 9, 0), Vector3(5, 9, 0), 4.0f, 2.0f, &cp, &cn, 1) ? 1 : 0);
		Plane box_planes[6] = {
			Plane(Vector3(1, 0, 0), 1.0f), Plane(Vector3(-1, 0, 0), 1.0f),
			Plane(Vector3(0, 1, 0), 1.0f), Plane(Vector3(0, -1, 0), 1.0f),
			Plane(Vector3(0, 0, 1), 1.0f), Plane(Vector3(0, 0, -1), 1.0f)
		};
		Vector3 vp, vn;
		const bool vh = Geometry3D::segment_intersects_convex(Vector3(-5, 0, 0), Vector3(5, 0, 0), box_planes, 6, &vp, &vn);
		outu("g3.convex_hit", vh ? 1 : 0);
		out("g3.convex_pt", vp);
		out("g3.convex_n", vn);
		outu("g3.pit_in", Geometry3D::point_in_projected_triangle(Vector3(0.2f, 0.2f, 0.0f), Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0)) ? 1 : 0);
		outu("g3.pit_out", Geometry3D::point_in_projected_triangle(Vector3(1.2f, 0.2f, 0.0f), Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0)) ? 1 : 0);
		outu("g3.pit_behind", Geometry3D::point_in_projected_triangle(Vector3(0.2f, 0.2f, -1.0f), Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0)) ? 1 : 0);
		Vector3 tc, sc;
		const Vector3 tn = vec3_cross(Vector3(1, 0, 0) - Vector3(0, 0, 0), Vector3(1, 0, 0) - Vector3(0, 1, 0)).normalized();
		const bool ts = Geometry3D::triangle_sphere_intersection_test(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0), tn, Vector3(0.3f, 0.3f, 0.5f), 1.0f, tc, sc);
		{
			Vector3 ps, qt;
			Geometry3D::get_closest_points_between_segments(Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(1, 2, 0), Vector3(3, 2, 0), ps, qt);
			out("g3.seg_ps", ps);
			out("g3.seg_qt", qt);
			outf("g3.seg_dist", Geometry3D::get_closest_distance_between_segments(Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(1, 2, 0), Vector3(3, 2, 0)));
			Geometry3D::get_closest_points_between_segments(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(3, 0, 0), Vector3(4, 0, 0), ps, qt);
			out("g3.seg_par_ps", ps);
			out("g3.seg_par_qt", qt);
			Geometry3D::get_closest_points_between_segments(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0.5f, 1, 0), Vector3(0.5f, 2, 0), ps, qt);
			out("g3.seg_cross_ps", ps);
			out("g3.seg_cross_qt", qt);
		}
		{
			Vector<Plane> bp = Geometry3D::build_box_planes(Vector3(1.0f, 2.0f, 3.0f));
			printf("g3.box_n,%d\n", bp.size());
			for (int i = 0; i < bp.size(); i++) {
				char lbl[32]; snprintf(lbl, sizeof(lbl), "g3.box%d", i);
				out(lbl, bp[i].normal);
				snprintf(lbl, sizeof(lbl), "g3.box_d%d", i);
				outf(lbl, bp[i].d);
			}
			Vector<Plane> cy = Geometry3D::build_cylinder_planes(2.0f, 3.0f, 4, Vector3::AXIS_Y);
			printf("g3.cylp_n,%d\n", cy.size());
			out("g3.cylp0", cy[0].normal);
			out("g3.cylp1", cy[1].normal);
			outf("g3.cylp_d0", cy[0].d);
		}
		{
			Vector<Plane> sp = Geometry3D::build_sphere_planes(2.0f, 2, 4, Vector3::AXIS_Y);
			printf("g3.sph_n,%d\n", sp.size());
			out("g3.sph0", sp[0].normal);
			out("g3.sph1", sp[1].normal);
			out("g3.sph2", sp[2].normal);
			outf("g3.sph_d0", sp[0].d);
		}
		{
			Vector<Plane> cap = Geometry3D::build_capsule_planes(2.0f, 4.0f, 4, 1, Vector3::AXIS_Y);
			printf("g3.cap_n,%d\n", cap.size());
			out("g3.cap0", cap[0].normal);
			out("g3.cap1", cap[1].normal);
			outf("g3.cap_d1", cap[1].d);
			Vector<Plane> boxp = Geometry3D::build_box_planes(Vector3(1.0f, 1.0f, 1.0f));
			Vector<Vector3> mp = Geometry3D::compute_convex_mesh_points(boxp.ptrw(), boxp.size());
			printf("g3.mesh_n,%d\n", mp.size());
			out("g3.mesh0", mp[0]);
		}
		outu("g3.tri_hit", ts ? 1 : 0);
		out("g3.tri_tc", tc);
		out("g3.tri_sc", sc);
		outu("g3.tri_miss", Geometry3D::triangle_sphere_intersection_test(Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 1, 0), tn, Vector3(5.0f, 5.0f, 5.0f), 1.0f, tc, sc) ? 1 : 0);
		Vector<Vector3> poly;
		poly.push_back(Vector3(-2, 0, -1)); poly.push_back(Vector3(2, 0, -1));
		poly.push_back(Vector3(2, 0, 1)); poly.push_back(Vector3(-2, 0, 1));
		Vector<Vector3> clp = Geometry3D::clip_polygon(poly, Plane(Vector3(1, 0, 0), 0.0f));
		printf("g3.clip_n,%d\n", clp.size());
		for (int i = 0; i < clp.size(); i++) {
			char lbl[32]; snprintf(lbl, sizeof(lbl), "g3.clip%d", i);
			out(lbl, clp[i]);
		}
		outu("g3.clip_all_in", Geometry3D::clip_polygon(poly, Plane(Vector3(0, 1, 0), 1.0f)).size() == poly.size() ? 1 : 0);
		outu("g3.clip_all_out", Geometry3D::clip_polygon(poly, Plane(Vector3(1, 0, 0), 5.0f)).size() == 0 ? 1 : 0);
		outu("g3.convex_miss", Geometry3D::segment_intersects_convex(Vector3(-5, 9, 0), Vector3(5, 9, 0), box_planes, 6, &vp, &vn) ? 1 : 0);
	}

	// --- AABB
	{
		const AABB box(Vector3(0.5f, -1.0f, 2.0f), Vector3(3.0f, 2.0f, 1.5f));
		const AABB other(Vector3(2.0f, 0.0f, 1.0f), Vector3(2.0f, 2.0f, 2.0f));
		out("aabb.center", box.get_center());
		outf("aabb.volume", box.get_volume());
		outu("aabb.intersects", box.intersects(other) ? 1 : 0);
		outu("aabb.intersects_no", box.intersects(AABB(Vector3(10, 10, 10), Vector3(1, 1, 1))) ? 1 : 0);
		outu("aabb.encloses", box.encloses(other) ? 1 : 0);
		out("aabb.merge", box.merge(other).position);
		out("aabb.merge_size", box.merge(other).size);
		out("aabb.intersection", box.intersection(other).position);
		out("aabb.intersection_size", box.intersection(other).size);
		outu("aabb.smits", box.smits_intersect_ray(Vector3(0.0f, 0.0f, 0.0f), Vector3(0.5f, 0.2f, 0.8f), 0.0f, 100.0f) ? 1 : 0);
		Vector3 hit;
		bool inside = false;
		box.find_intersects_ray(Vector3(0.0f, 0.0f, 0.0f), Vector3(0.5f, 0.2f, 0.8f), inside, &hit);
		out("aabb.ray_hit", hit);
		outu("aabb.longest_axis", (unsigned)box.get_longest_axis_index());
		outu("aabb.shortest_axis", (unsigned)box.get_shortest_axis_index());
		real_t rmin, rmax;
		box.project_range_in_plane(Plane(Vector3(0.0f, 1.0f, 0.0f), 0.5f), rmin, rmax);
		outf("aabb.proj_min", rmin);
		outf("aabb.proj_max", rmax);
		AABB grown = box;
		grown.grow_by(0.25f);
		out("aabb.grown", grown.position);
		out("aabb.grown_size", grown.size);
		AABB quant = box;
		quant.quantize(0.5f);
		out("aabb.quant", quant.position);
		out("aabb.quant_size", quant.size);
		outu("aabb.hash", box.hash());
	}

	// --- Plane
	{
		const Plane plane(Vector3(0.0f, 1.0f, 0.0f), 2.0f);
		outf("plane.distance", plane.distance_to(Vector3(1.0f, 5.5f, -2.0f)));
		out("plane.project", plane.project(Vector3(1.0f, 5.5f, -2.0f)));
		out("plane.center", plane.get_center());
		out("plane.perp", plane.get_any_perpendicular_normal());
		const Plane p1(Vector3(1.0f, 0.0f, 0.0f), 1.0f);
		const Plane p2(Vector3(0.0f, 0.0f, 1.0f), 3.0f);
		Vector3 p3;
		plane.intersect_3(p1, p2, &p3);
		out("plane.intersect3", p3);
		Vector3 pr;
		plane.intersects_ray(Vector3(0.0f, 8.0f, 0.0f), Vector3(0.0f, -2.0f, 0.0f), &pr);
		out("plane.ray", pr);
		plane.intersects_segment(Vector3(0.0f, 5.0f, 0.0f), Vector3(0.0f, 0.0f, 0.0f), &pr);
		out("plane.segment", pr);
		Plane scaled(Vector3(0.0f, 3.0f, 0.0f), 6.0f);
		scaled.normalize();
		outf("plane.norm_d", scaled.d);
		out("plane.norm_n", scaled.normal);
	}

	// --- Rect2
	{
		const Rect2 rect(1.0f, 2.0f, 4.0f, 3.0f);
		const Rect2 other(-1.0f, 0.5f, 3.0f, 3.0f);
		out("rect.center", rect.get_center());
		outf("rect.area", rect.get_area());
		outu("rect.intersects", rect.intersects(other) ? 1 : 0);
		out("rect.intersection", rect.intersection(other).position);
		out("rect.intersection_size", rect.intersection(other).size);
		out("rect.merge", rect.merge(other).position);
		out("rect.merge_size", rect.merge(other).size);
		outf("rect.distance_out", rect.distance_to(Vector2(7.0f, 9.0f)));
		outf("rect.distance_in", rect.distance_to(Vector2(2.0f, 3.0f)));
		out("rect.grow", rect.grow(0.5f).position);
		out("rect.grow_size", rect.grow(0.5f).size);
		out("rect.expand", rect.expand(Vector2(-2.0f, 7.0f)).position);
		out("rect.expand_size", rect.expand(Vector2(-2.0f, 7.0f)).size);
		out("rect.support", rect.get_support(Vector2(1.0f, -1.0f)));
		const Transform2D xf2(0.6f, Vector2(0.5f, 0.5f));
		outu("rect.xf_hit", rect.intersects_transformed(xf2, Rect2(0.0f, 0.0f, 2.0f, 2.0f)) ? 1 : 0);
		outu("rect.xf_miss", rect.intersects_transformed(xf2, Rect2(50.0f, 50.0f, 1.0f, 1.0f)) ? 1 : 0);
		Transform2D xf3(0.0f, Vector2(4.0f, 0.0f));
		outu("rect.xf_touch", rect.intersects_transformed(xf3, Rect2(0.0f, 0.0f, 2.0f, 2.0f)) ? 1 : 0);
	}

	// --- PCG stream (bit-exact requirement)
	{
		pcg32_random_t st;
		pcg32_srandom_r(&st, 123456789ULL, PCG_DEFAULT_INC_64);
		printf("pcg");
		for (int i = 0; i < 8; i++) {
			printf(",%u", pcg32_random_r(&st));
		}
		printf("\n");
	}

	return 0;
}
