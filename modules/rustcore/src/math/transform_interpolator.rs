//! `core/math/transform_interpolator.*`: fixed-timestep interpolation.
//!
//! Two stages, exactly as the engine: `find_method` decides between linear
//! interpolation and (scaled) spherical interpolation, then the chosen method
//! is applied. Splitting them lets a caller pay the classification once and
//! interpolate many frames.
//!
//! Ported 1:1, including the tolerances. The original comments are kept where
//! they explain a check that looks redundant but is not.

use crate::math::basis::Basis;
use crate::math::funcs;
use crate::math::quaternion::Quaternion;
use crate::math::transform_2d::Transform2D;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Lerp,
    Slerp,
    ScaledSlerp,
}

fn sign(val: f32) -> bool {
    val >= 0.0
}

fn vec3_sum(p: Vector3) -> f32 {
    p.x + p.y + p.z
}

fn vec3_normalize(v: &mut Vector3) -> f32 {
    let lengthsq = v.length_squared();
    if lengthsq == 0.0 {
        *v = Vector3::ZERO;
        return 0.0;
    }
    let length = lengthsq.sqrt();
    v.x /= length;
    v.y /= length;
    v.z /= length;
    length
}

fn vec3_is_equal_approx(a: Vector3, b: Vector3, tol: f32) -> bool {
    funcs::is_equal_approx_tol(a.x, b.x, tol)
        && funcs::is_equal_approx_tol(a.y, b.y, tol)
        && funcs::is_equal_approx_tol(a.z, b.z, tol)
}

/// Gram-Schmidt; returns the axis lengths that were divided out.
fn basis_orthonormalize(basis: &mut Basis) -> Vector3 {
    let mut x = basis.get_column(0);
    let mut y = basis.get_column(1);
    let mut z = basis.get_column(2);

    let mut lengths = Vector3::ZERO;
    lengths.x = vec3_normalize(&mut x);
    y = y - x * x.dot(y);
    lengths.y = vec3_normalize(&mut y);
    z = z - x * x.dot(z) - y * y.dot(z);
    lengths.z = vec3_normalize(&mut z);

    basis.set_column(0, x);
    basis.set_column(1, y);
    basis.set_column(2, z);
    lengths
}

/// Classifies `basis`. `needed_normalize` is an in/out flag: once one of the
/// two bases had to be normalized, the other must be normalized too so both
/// take the `ScaledSlerp` path.
fn test_basis(basis: Basis, needed_normalize: &mut bool) -> (Method, Quaternion) {
    let mut p_basis = basis;
    let mut al = Vector3::new(
        p_basis.get_column(0).length_squared(),
        p_basis.get_column(1).length_squared(),
        p_basis.get_column(2).length_squared(),
    );

    if *needed_normalize || !vec3_is_equal_approx(al, Vector3::ONE, 0.001) {
        // Non-unit scale. Slerp needs an orthonormal basis, but a uniformly
        // scaled basis can still be slerped after normalization, lerping the
        // scales separately.
        const SL_EPSILON: f32 = 0.00001;
        if al.x < SL_EPSILON || al.y < SL_EPSILON || al.z < SL_EPSILON {
            return (Method::Lerp, Quaternion::IDENTITY);
        }

        al.x = al.x.sqrt();
        al.y = al.y.sqrt();
        al.z = al.z.sqrt();

        let mut norm = p_basis;
        norm.set_column(0, norm.get_column(0) / al.x);
        norm.set_column(1, norm.get_column(1) / al.y);
        norm.set_column(2, norm.get_column(2) / al.z);

        p_basis = norm;
        *needed_normalize = true;
    }

    // Looser than `Basis::slerp`'s checks, which are too sensitive to float
    // error for interpolation use.
    let det = p_basis.determinant();
    if !funcs::is_equal_approx_tol(det, 1.0, 0.01) {
        return (Method::Lerp, Quaternion::IDENTITY);
    }

    if !basis_is_orthogonal(&p_basis, 0.01) {
        return (Method::Lerp, Quaternion::IDENTITY);
    }

    let quat = basis_to_quat_unchecked(&p_basis);
    if !quat.is_normalized() {
        return (Method::Lerp, Quaternion::IDENTITY);
    }

    (
        if *needed_normalize {
            Method::ScaledSlerp
        } else {
            Method::Slerp
        },
        quat,
    )
}

fn basis_is_orthogonal(basis: &Basis, epsilon: f32) -> bool {
    let identity = Basis::IDENTITY;
    let m = *basis * basis.transposed();

    vec3_is_equal_approx(m.rows[0], identity.rows[0], epsilon)
        && vec3_is_equal_approx(m.rows[1], identity.rows[1], epsilon)
        && vec3_is_equal_approx(m.rows[2], identity.rows[2], epsilon)
}

/// `_basis_to_quat_unchecked`: trace method, without the normalization check.
fn basis_to_quat_unchecked(basis: &Basis) -> Quaternion {
    let m = basis;
    let trace = m.rows[0].x + m.rows[1].y + m.rows[2].z;
    let mut temp = [0.0f32; 4];

    if trace > 0.0 {
        let mut s = (trace + 1.0).sqrt();
        temp[3] = s * 0.5;
        s = 0.5 / s;

        temp[0] = (m.rows[2].y - m.rows[1].z) * s;
        temp[1] = (m.rows[0].z - m.rows[2].x) * s;
        temp[2] = (m.rows[1].x - m.rows[0].y) * s;
    } else {
        let i = if m.rows[0].x < m.rows[1].y {
            if m.rows[1].y < m.rows[2].z {
                2
            } else {
                1
            }
        } else if m.rows[0].x < m.rows[2].z {
            2
        } else {
            0
        };
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;

        let mut s = (m.rows[i][i] - m.rows[j][j] - m.rows[k][k] + 1.0).sqrt();
        temp[i] = s * 0.5;
        s = 0.5 / s;

        temp[3] = (m.rows[k][j] - m.rows[j][k]) * s;
        temp[j] = (m.rows[j][i] + m.rows[i][j]) * s;
        temp[k] = (m.rows[k][i] + m.rows[i][k]) * s;
    }

    Quaternion::new(temp[0], temp[1], temp[2], temp[3])
}

/// Slerp without the normalization checks, matching the engine's local copy.
fn quat_slerp_unchecked(from: &Quaternion, to: &Quaternion, fraction: f32) -> Quaternion {
    let mut cosom = from.dot(to);
    let to1 = if cosom < 0.0 {
        cosom = -cosom;
        -*to
    } else {
        *to
    };

    let (scale0, scale1) = if (1.0 - cosom) > funcs::CMP_EPSILON as f32 {
        let omega = funcs::acos(cosom);
        let sinom = omega.sin();
        (
            ((1.0 - fraction) * omega).sin() / sinom,
            (fraction * omega).sin() / sinom,
        )
    } else {
        // Very close: linear interpolation is indistinguishable.
        (1.0 - fraction, fraction)
    };

    Quaternion::new(
        scale0 * from.x + scale1 * to1.x,
        scale0 * from.y + scale1 * to1.y,
        scale0 * from.z + scale1 * to1.z,
        scale0 * from.w + scale1 * to1.w,
    )
}

fn basis_slerp_unchecked(from: &Basis, to: &Basis, fraction: f32) -> Basis {
    let q_from = basis_to_quat_unchecked(from);
    let q_to = basis_to_quat_unchecked(to);
    Basis::from_quaternion(quat_slerp_unchecked(&q_from, &q_to, fraction))
}

fn interpolate_basis_scaled_slerp(prev: &Basis, curr: &Basis, fraction: f32) -> Basis {
    let mut p = *prev;
    let mut c = *curr;
    let lengths_prev = basis_orthonormalize(&mut p);
    let lengths_curr = basis_orthonormalize(&mut c);

    let mut result = basis_slerp_unchecked(&p, &c, fraction);

    let lengths_lerped = lengths_prev + ((lengths_curr - lengths_prev) * fraction);

    // Column order matters here: each column is scaled by its own length.
    result.set_column(0, result.get_column(0) * lengths_lerped.x);
    result.set_column(1, result.get_column(1) * lengths_lerped.y);
    result.set_column(2, result.get_column(2) * lengths_lerped.z);
    result
}

fn interpolate_basis_linear(prev: &Basis, curr: &Basis, fraction: f32) -> Basis {
    let mut result = prev.lerp(curr, fraction);

    // Guard against zero-scale axes, which break normals and cross products
    // elsewhere in the engine. A different component per axis avoids all three
    // axes collapsing onto the same direction.
    const SMALLEST: f32 = 0.0001;
    const SMALLEST_SQUARED: f32 = SMALLEST * SMALLEST;
    for n in 0..3 {
        let mut axis = result.get_column(n);
        if axis.length_squared() < SMALLEST_SQUARED {
            axis[n] = SMALLEST;
            result.set_column(n, axis);
        }
    }
    result
}

pub fn interpolate_basis_via_method(
    prev: &Basis,
    curr: &Basis,
    fraction: f32,
    method: Method,
) -> Basis {
    match method {
        Method::Slerp => basis_slerp_unchecked(prev, curr, fraction),
        Method::ScaledSlerp => interpolate_basis_scaled_slerp(prev, curr, fraction),
        Method::Lerp => interpolate_basis_linear(prev, curr, fraction),
    }
}

pub fn interpolate_basis(prev: &Basis, curr: &Basis, fraction: f32) -> Basis {
    let method = find_method(prev, curr);
    interpolate_basis_via_method(prev, curr, fraction, method)
}

pub fn interpolate_transform_3d_via_method(
    prev: &Transform3D,
    curr: &Transform3D,
    fraction: f32,
    method: Method,
) -> Transform3D {
    Transform3D {
        basis: interpolate_basis_via_method(&prev.basis, &curr.basis, fraction, method),
        origin: prev.origin + ((curr.origin - prev.origin) * fraction),
    }
}

pub fn interpolate_transform_3d(
    prev: &Transform3D,
    curr: &Transform3D,
    fraction: f32,
) -> Transform3D {
    Transform3D {
        basis: interpolate_basis(&prev.basis, &curr.basis, fraction),
        origin: prev.origin + ((curr.origin - prev.origin) * fraction),
    }
}

pub fn interpolate_transform_2d(
    prev: &Transform2D,
    curr: &Transform2D,
    fraction: f32,
) -> Transform2D {
    // A determinant sign change means the handedness flipped; interpolating the
    // basis between the two would collapse it through zero, so only the origin
    // is interpolated.
    if sign(prev.determinant()) != sign(curr.determinant()) {
        return Transform2D {
            columns: [
                curr.columns[0],
                curr.columns[1],
                prev.columns[2].lerp(curr.columns[2], fraction),
            ],
        };
    }

    prev.interpolate_with(curr, fraction)
}

/// Basic checksum used by the engine's tests to compare transforms quickly.
pub fn checksum_transform_3d(t: &Transform3D) -> f32 {
    let mut sum = vec3_sum(t.origin);
    sum -= vec3_sum(t.basis.rows[0]);
    sum += vec3_sum(t.basis.rows[1]);
    sum -= vec3_sum(t.basis.rows[2]);
    sum
}

pub fn find_method(a: &Basis, b: &Basis) -> Method {
    let mut needed_normalize = false;

    let (method, q0) = test_basis(*a, &mut needed_normalize);
    if method == Method::Lerp {
        return method;
    }

    let (method, q1) = test_basis(*b, &mut needed_normalize);
    if method == Method::Lerp {
        return method;
    }

    // Same threshold the slerp routine uses to fall back to lerp.
    let cosom = q0.dot(&q1).abs();
    if (1.0 - cosom) <= funcs::CMP_EPSILON as f32 {
        return Method::Lerp;
    }

    method
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::basis::EulerOrder;
    use crate::math::vector2::Vector2;

    fn basis_from_euler(x: f32, y: f32, z: f32) -> Basis {
        Basis::from_euler(Vector3::new(x, y, z), EulerOrder::YXZ)
    }

    #[test]
    fn matching_rotations_slerp() {
        let a = basis_from_euler(0.0, 0.0, 0.0);
        let b = basis_from_euler(0.0, 1.0, 0.0);
        assert_eq!(find_method(&a, &b), Method::Slerp);

        let mid = interpolate_basis(&a, &b, 0.5);
        let expected = basis_from_euler(0.0, 0.5, 0.0);
        for c in 0..3 {
            let d = (mid.get_column(c) - expected.get_column(c)).length();
            assert!(d < 1e-4, "column {c} differs by {d}");
        }
    }

    #[test]
    fn endpoints_are_exact() {
        let a = basis_from_euler(0.2, -0.4, 0.7);
        let b = basis_from_euler(-0.3, 0.8, 0.1);
        for m in [Method::Lerp, Method::Slerp, Method::ScaledSlerp] {
            let at0 = interpolate_basis_via_method(&a, &b, 0.0, m);
            let at1 = interpolate_basis_via_method(&a, &b, 1.0, m);
            for c in 0..3 {
                assert!((at0.get_column(c) - a.get_column(c)).length() < 1e-5, "{m:?} at 0");
                assert!((at1.get_column(c) - b.get_column(c)).length() < 1e-5, "{m:?} at 1");
            }
        }
    }

    #[test]
    fn uniform_scale_uses_scaled_slerp_and_lerps_lengths() {
        let rot_a = basis_from_euler(0.0, 0.0, 0.0);
        let rot_b = basis_from_euler(0.0, 1.0, 0.0);
        let a = rot_a.scaled(Vector3::splat(2.0));
        let b = rot_b.scaled(Vector3::splat(4.0));

        assert_eq!(find_method(&a, &b), Method::ScaledSlerp);

        let mid = interpolate_basis(&a, &b, 0.5);
        // Halfway rotation, scale 3 on every axis.
        for c in 0..3 {
            let len = mid.get_column(c).length();
            assert!((len - 3.0).abs() < 1e-3, "column {c} length {len}");
        }
    }

    #[test]
    fn shear_falls_back_to_linear() {
        // Sheared basis: not a rotation times a scale.
        let mut a = Basis::IDENTITY;
        a.set_column(0, Vector3::new(1.0, 0.3, 0.0));
        let b = basis_from_euler(0.0, 0.5, 0.0);
        assert_eq!(find_method(&a, &b), Method::Lerp);
    }

    #[test]
    fn zero_scale_axes_are_nudged_in_linear_mode() {
        let a = Basis::IDENTITY;
        let zero = Basis::from_scale(Vector3::ZERO);
        let r = interpolate_basis_linear(&a, &zero, 1.0);
        for c in 0..3 {
            // The nudge is on the axis's own component.
            assert_eq!(r.get_column(c)[c], 0.0001, "column {c}");
        }
    }

    #[test]
    fn transform_3d_interpolates_origin_linearly() {
        let a = Transform3D {
            basis: basis_from_euler(0.0, 0.0, 0.0),
            origin: Vector3::new(0.0, 0.0, 0.0),
        };
        let b = Transform3D {
            basis: basis_from_euler(0.0, 1.0, 0.0),
            origin: Vector3::new(10.0, -4.0, 2.0),
        };
        let mid = interpolate_transform_3d(&a, &b, 0.25);
        assert!((mid.origin - Vector3::new(2.5, -1.0, 0.5)).length() < 1e-6);
        assert_eq!(find_method(&a.basis, &b.basis), Method::Slerp);
    }

    #[test]
    fn transform_2d_flip_keeps_current_basis() {
        // Flipped X: determinant signs differ, so the basis must not be
        // interpolated (it would pass through zero).
        let a = Transform2D::from_rotation_origin(0.3, Vector2::new(0.0, 0.0));
        let mut b = Transform2D::from_rotation_origin(0.3, Vector2::new(4.0, 2.0));
        b.columns[0] = -b.columns[0];

        let r = interpolate_transform_2d(&a, &b, 0.5);
        assert_eq!(r.columns[0], b.columns[0]);
        assert_eq!(r.columns[1], b.columns[1]);
        assert!((r.columns[2] - Vector2::new(2.0, 1.0)).length() < 1e-6);
    }

    #[test]
    fn checksum_matches_its_definition() {
        let t = Transform3D {
            basis: Basis::IDENTITY,
            origin: Vector3::new(1.0, 2.0, 3.0),
        };
        // origin sum 6, minus row0 (1), plus row1 (1), minus row2 (1).
        assert!((checksum_transform_3d(&t) - 5.0).abs() < 1e-6);
    }
}
