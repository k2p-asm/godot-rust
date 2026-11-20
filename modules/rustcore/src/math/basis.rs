//! Port of `core/math/basis.h` + `basis.cpp` — the 3x3 rotation/scale matrix.
//!
//! `rows[i]` are the matrix rows, exactly as in the engine, and `xform` is
//! row·vector — so `Basis::xform` here is bit-compatible with the C++ version.
//!
//! Pending: `get_euler` (the 6-way decomposition, ~200 lines), `get_axis_angle`
//! singularity handling, `rotate_sh`, `diagonalize`, and
//! `rotref_posscale_decomposition`.

use crate::math::funcs;
use crate::math::quaternion::Quaternion;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EulerOrder {
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

/// 3x3 matrix stored as three row vectors, matching `Basis::rows`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis {
    pub rows: [Vector3; 3],
}

impl Default for Basis {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Basis {
    pub const IDENTITY: Basis = Basis {
        rows: [
            Vector3 { x: 1.0, y: 0.0, z: 0.0 },
            Vector3 { x: 0.0, y: 1.0, z: 0.0 },
            Vector3 { x: 0.0, y: 0.0, z: 1.0 },
        ],
    };

    #[inline]
    pub const fn from_rows(x: Vector3, y: Vector3, z: Vector3) -> Self {
        Self { rows: [x, y, z] }
    }

    /// Column-major constructor, as `Basis(x_axis, y_axis, z_axis)`.
    #[inline]
    pub const fn from_columns(x: Vector3, y: Vector3, z: Vector3) -> Self {
        Self {
            rows: [
                Vector3 { x: x.x, y: y.x, z: z.x },
                Vector3 { x: x.y, y: y.y, z: z.y },
                Vector3 { x: x.z, y: y.z, z: z.z },
            ],
        }
    }

    #[inline]
    pub fn from_scale(scale: Vector3) -> Self {
        Self::from_rows(
            Vector3::new(scale.x, 0.0, 0.0),
            Vector3::new(0.0, scale.y, 0.0),
            Vector3::new(0.0, 0.0, scale.z),
        )
    }

    #[inline]
    pub fn get_column(&self, i: usize) -> Vector3 {
        Vector3::new(self.rows[0][i], self.rows[1][i], self.rows[2][i])
    }

    #[inline]
    pub fn set_column(&mut self, i: usize, v: Vector3) {
        self.rows[0][i] = v.x;
        self.rows[1][i] = v.y;
        self.rows[2][i] = v.z;
    }

    #[inline]
    pub fn set_columns(&mut self, x: Vector3, y: Vector3, z: Vector3) {
        *self = Self::from_columns(x, y, z);
    }

    /// Cofactor used by `invert`.
    #[inline]
    fn cofac(&self, row1: usize, col1: usize, row2: usize, col2: usize) -> f32 {
        self.rows[row1][col1] * self.rows[row2][col2]
            - self.rows[row1][col2] * self.rows[row2][col1]
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.rows[0][0] * self.cofac(1, 1, 2, 2)
            + self.rows[0][1] * self.cofac(1, 2, 2, 0)
            + self.rows[0][2] * self.cofac(1, 0, 2, 1)
    }

    pub fn invert(&mut self) {
        let co = [
            self.cofac(1, 1, 2, 2),
            self.cofac(1, 2, 2, 0),
            self.cofac(1, 0, 2, 1),
        ];
        let det = self.rows[0][0] * co[0] + self.rows[0][1] * co[1] + self.rows[0][2] * co[2];
        // The C++ version divides by det; a singular basis produces inf/NaN there
        // too, so the behaviour is preserved instead of panicking.
        let s = 1.0 / det;
        *self = Self::from_rows(
            Vector3::new(
                co[0] * s,
                self.cofac(0, 2, 2, 1) * s,
                self.cofac(0, 1, 1, 2) * s,
            ),
            Vector3::new(
                co[1] * s,
                self.cofac(0, 0, 2, 2) * s,
                self.cofac(0, 2, 1, 0) * s,
            ),
            Vector3::new(
                co[2] * s,
                self.cofac(0, 1, 2, 0) * s,
                self.cofac(0, 0, 1, 1) * s,
            ),
        );
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let mut inv = *self;
        inv.invert();
        inv
    }

    pub fn transpose(&mut self) {
        let (a, b, c) = (self.rows[0][1], self.rows[0][2], self.rows[1][2]);
        self.rows[0][1] = self.rows[1][0];
        self.rows[0][2] = self.rows[2][0];
        self.rows[1][2] = self.rows[2][1];
        self.rows[1][0] = a;
        self.rows[2][0] = b;
        self.rows[2][1] = c;
    }

    #[inline]
    pub fn transposed(&self) -> Self {
        let mut t = *self;
        t.transpose();
        t
    }

    #[inline]
    pub fn tdotx(&self, v: Vector3) -> f32 {
        self.rows[0][0] * v[0] + self.rows[1][0] * v[1] + self.rows[2][0] * v[2]
    }

    #[inline]
    pub fn tdoty(&self, v: Vector3) -> f32 {
        self.rows[0][1] * v[0] + self.rows[1][1] * v[1] + self.rows[2][1] * v[2]
    }

    #[inline]
    pub fn tdotz(&self, v: Vector3) -> f32 {
        self.rows[0][2] * v[0] + self.rows[1][2] * v[1] + self.rows[2][2] * v[2]
    }

    #[inline]
    pub fn xform(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            self.rows[0].dot(v),
            self.rows[1].dot(v),
            self.rows[2].dot(v),
        )
    }

    #[inline]
    pub fn xform_inv(&self, v: Vector3) -> Vector3 {
        Vector3::new(self.tdotx(v), self.tdoty(v), self.tdotz(v))
    }

    pub fn orthonormalize(&mut self) {
        // Gram-Schmidt.
        let mut x = self.get_column(0);
        let mut y = self.get_column(1);
        let mut z = self.get_column(2);

        x.normalize();
        y = y - x * x.dot(y);
        y.normalize();
        z = z - x * x.dot(z) - y * y.dot(z);
        z.normalize();

        self.set_column(0, x);
        self.set_column(1, y);
        self.set_column(2, z);
    }

    #[inline]
    pub fn orthonormalized(&self) -> Self {
        let mut c = *self;
        c.orthonormalize();
        c
    }

    pub fn orthogonalize(&mut self) {
        let scl = self.get_scale();
        self.orthonormalize();
        self.scale_local(scl);
    }

    #[inline]
    pub fn orthogonalized(&self) -> Self {
        let mut c = *self;
        c.orthogonalize();
        c
    }

    #[inline]
    pub fn is_orthogonal(&self) -> bool {
        let x = self.get_column(0);
        let y = self.get_column(1);
        let z = self.get_column(2);
        funcs::is_zero_approx(x.dot(y))
            && funcs::is_zero_approx(x.dot(z))
            && funcs::is_zero_approx(y.dot(z))
    }

    #[inline]
    pub fn is_orthonormal(&self) -> bool {
        let x = self.get_column(0);
        let y = self.get_column(1);
        let z = self.get_column(2);
        funcs::is_equal_approx(x.length_squared(), 1.0)
            && funcs::is_equal_approx(y.length_squared(), 1.0)
            && funcs::is_equal_approx(z.length_squared(), 1.0)
            && funcs::is_zero_approx(x.dot(y))
            && funcs::is_zero_approx(x.dot(z))
            && funcs::is_zero_approx(y.dot(z))
    }

    /// Orthogonal with uniform scale: a rotation (plus possible reflection).
    pub fn is_conformal(&self) -> bool {
        let x = self.get_column(0);
        let y = self.get_column(1);
        let z = self.get_column(2);
        let lx = x.length_squared();
        funcs::is_equal_approx(lx, y.length_squared())
            && funcs::is_equal_approx(lx, z.length_squared())
            && funcs::is_zero_approx(x.dot(y))
            && funcs::is_zero_approx(x.dot(z))
            && funcs::is_zero_approx(y.dot(z))
    }

    #[inline]
    pub fn is_rotation(&self) -> bool {
        self.is_conformal() && self.determinant() > 0.0
    }

    /// Multiplies from the left by the scaling matrix: `M -> S * M`.
    #[inline]
    pub fn scale(&mut self, scale: Vector3) {
        self.rows[0] *= scale.x;
        self.rows[1] *= scale.y;
        self.rows[2] *= scale.z;
    }

    #[inline]
    pub fn scaled(&self, scale: Vector3) -> Self {
        let mut m = *self;
        m.scale(scale);
        m
    }

    /// Scales in the object-local frame: `M -> M * S`.
    #[inline]
    pub fn scale_local(&mut self, scale: Vector3) {
        self.rows[0] *= scale;
        self.rows[1] *= scale;
        self.rows[2] *= scale;
    }

    #[inline]
    pub fn scaled_local(&self, scale: Vector3) -> Self {
        let mut m = *self;
        m.scale_local(scale);
        m
    }

    #[inline]
    pub fn get_scale_abs(&self) -> Vector3 {
        Vector3::new(
            self.get_column(0).length(),
            self.get_column(1).length(),
            self.get_column(2).length(),
        )
    }

    #[inline]
    pub fn get_scale_global(&self) -> Vector3 {
        let det_sign = funcs::sign_f32(self.determinant());
        det_sign
            * Vector3::new(
                self.rows[0].length(),
                self.rows[1].length(),
                self.rows[2].length(),
            )
    }

    #[inline]
    pub fn get_scale(&self) -> Vector3 {
        funcs::sign_f32(self.determinant()) * self.get_scale_abs()
    }

    #[inline]
    pub fn get_uniform_scale(&self) -> f32 {
        (self.rows[0].length() + self.rows[1].length() + self.rows[2].length()) / 3.0
    }

    /// Rotation matrix from axis and angle (`Basis::set_axis_angle`).
    /// The axis is assumed to be normalized.
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        let axis_sq = Vector3::new(axis.x * axis.x, axis.y * axis.y, axis.z * axis.z);
        let cosine = angle.cos();
        let sine = angle.sin();
        let t = 1.0 - cosine;

        let mut rows = [[0.0f32; 3]; 3];
        rows[0][0] = axis_sq.x + cosine * (1.0 - axis_sq.x);
        rows[1][1] = axis_sq.y + cosine * (1.0 - axis_sq.y);
        rows[2][2] = axis_sq.z + cosine * (1.0 - axis_sq.z);

        let mut xyzt = axis.x * axis.y * t;
        let mut zyxs = axis.z * sine;
        rows[0][1] = xyzt - zyxs;
        rows[1][0] = xyzt + zyxs;

        xyzt = axis.x * axis.z * t;
        zyxs = axis.y * sine;
        rows[0][2] = xyzt + zyxs;
        rows[2][0] = xyzt - zyxs;

        xyzt = axis.y * axis.z * t;
        zyxs = axis.x * sine;
        rows[1][2] = xyzt - zyxs;
        rows[2][1] = xyzt + zyxs;

        Self {
            rows: [
                Vector3::new(rows[0][0], rows[0][1], rows[0][2]),
                Vector3::new(rows[1][0], rows[1][1], rows[1][2]),
                Vector3::new(rows[2][0], rows[2][1], rows[2][2]),
            ],
        }
    }

    /// `R -> R_axis * R`, i.e. rotating the object in the parent frame.
    #[inline]
    pub fn rotated(&self, axis: Vector3, angle: f32) -> Self {
        Self::from_axis_angle(axis, angle) * *self
    }

    #[inline]
    pub fn rotate(&mut self, axis: Vector3, angle: f32) {
        *self = self.rotated(axis, angle);
    }

    #[inline]
    pub fn rotated_local(&self, axis: Vector3, angle: f32) -> Self {
        *self * Self::from_axis_angle(axis, angle)
    }

    #[inline]
    pub fn rotate_local(&mut self, axis: Vector3, angle: f32) {
        *self = self.rotated_local(axis, angle);
    }

    pub fn set_quaternion(&mut self, q: Quaternion) {
        let d = q.length_squared();
        let s = 2.0 / d;
        let xs = q.x * s;
        let ys = q.y * s;
        let zs = q.z * s;
        let wx = q.w * xs;
        let wy = q.w * ys;
        let wz = q.w * zs;
        let xx = q.x * xs;
        let xy = q.x * ys;
        let xz = q.x * zs;
        let yy = q.y * ys;
        let yz = q.y * zs;
        let zz = q.z * zs;
        *self = Self::from_rows(
            Vector3::new(1.0 - (yy + zz), xy - wz, xz + wy),
            Vector3::new(xy + wz, 1.0 - (xx + zz), yz - wx),
            Vector3::new(xz - wy, yz + wx, 1.0 - (xx + yy)),
        );
    }

    pub fn from_quaternion(q: Quaternion) -> Self {
        let mut b = Self::IDENTITY;
        b.set_quaternion(q);
        b
    }

    pub fn get_quaternion(&self) -> Quaternion {
        // Trace-based extraction, identical to the engine's implementation.
        let m = *self;
        let trace = m.rows[0][0] + m.rows[1][1] + m.rows[2][2];
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt();
            let w = s * 0.5;
            let s = 0.5 / s;
            Quaternion::new(
                (m.rows[2][1] - m.rows[1][2]) * s,
                (m.rows[0][2] - m.rows[2][0]) * s,
                (m.rows[1][0] - m.rows[0][1]) * s,
                w,
            )
        } else {
            let i = if m.rows[0][0] < m.rows[1][1] {
                if m.rows[1][1] < m.rows[2][2] {
                    2
                } else {
                    1
                }
            } else if m.rows[0][0] < m.rows[2][2] {
                2
            } else {
                0
            };
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;

            let mut s = (m.rows[i][i] - m.rows[j][j] - m.rows[k][k] + 1.0).sqrt();
            let mut q = [0.0f32; 3];
            q[i] = s * 0.5;
            s = 0.5 / s;
            q[j] = (m.rows[i][j] + m.rows[j][i]) * s;
            q[k] = (m.rows[i][k] + m.rows[k][i]) * s;
            let w = (m.rows[k][j] - m.rows[j][k]) * s;
            Quaternion::new(q[0], q[1], q[2], w)
        }
    }

    pub fn get_rotation_quaternion(&self) -> Quaternion {
        let mut m = self.orthonormalized();
        if m.determinant() < 0.0 {
            m.scale(Vector3::splat(-1.0));
        }
        m.get_quaternion()
    }

    /// Builds the matrix from Euler angles for the given order, using the
    /// engine's sign conventions (`Basis::set_euler`).
    pub fn from_euler(euler: Vector3, order: EulerOrder) -> Self {
        let c = euler.x.cos();
        let s = euler.x.sin();
        let xmat = Self::from_rows(
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, c, -s),
            Vector3::new(0.0, s, c),
        );

        let c = euler.y.cos();
        let s = euler.y.sin();
        let ymat = Self::from_rows(
            Vector3::new(c, 0.0, s),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(-s, 0.0, c),
        );

        let c = euler.z.cos();
        let s = euler.z.sin();
        let zmat = Self::from_rows(
            Vector3::new(c, -s, 0.0),
            Vector3::new(s, c, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        );

        match order {
            EulerOrder::XYZ => xmat * (ymat * zmat),
            EulerOrder::XZY => xmat * zmat * ymat,
            EulerOrder::YXZ => ymat * xmat * zmat,
            EulerOrder::YZX => ymat * zmat * xmat,
            EulerOrder::ZXY => zmat * xmat * ymat,
            EulerOrder::ZYX => zmat * ymat * xmat,
        }
    }

    /// Axis-angle representation of the rotation part (`Basis::get_axis_angle`).
    /// Returns `(axis, angle)`; the axis is `(0, 1, 0)` with angle 0 for identity.
    pub fn get_axis_angle(&self) -> (Vector3, f32) {
        let r = &self.rows;
        let eps = funcs::CMP_EPSILON as f32;

        if funcs::is_zero_approx(r[0][1] - r[1][0])
            && funcs::is_zero_approx(r[0][2] - r[2][0])
            && funcs::is_zero_approx(r[1][2] - r[2][1])
        {
            // Singularity: either identity or a 180 degree rotation.
            if self.is_diagonal()
                && (r[0][0] + r[1][1] + r[2][2] - 3.0).abs() < 3.0 * eps
            {
                return (Vector3::UP, 0.0);
            }

            let xx = (r[0][0] + 1.0) / 2.0;
            let yy = (r[1][1] + 1.0) / 2.0;
            let zz = (r[2][2] + 1.0) / 2.0;
            let xy = (r[0][1] + r[1][0]) / 4.0;
            let xz = (r[0][2] + r[2][0]) / 4.0;
            let yz = (r[1][2] + r[2][1]) / 4.0;

            let axis;
            if xx > yy && xx > zz {
                if xx < eps {
                    axis = Vector3::new(0.0, funcs::SQRT12 as f32, funcs::SQRT12 as f32);
                } else {
                    let x = xx.sqrt();
                    axis = Vector3::new(x, xy / x, xz / x);
                }
            } else if yy > zz {
                if yy < eps {
                    axis = Vector3::new(funcs::SQRT12 as f32, 0.0, funcs::SQRT12 as f32);
                } else {
                    let y = yy.sqrt();
                    axis = Vector3::new(xy / y, y, yz / y);
                }
            } else if zz < eps {
                axis = Vector3::new(funcs::SQRT12 as f32, funcs::SQRT12 as f32, 0.0);
            } else {
                let z = zz.sqrt();
                axis = Vector3::new(xz / z, yz / z, z);
            }

            return (axis, funcs::PI as f32);
        }

        let mut s = ((r[2][1] - r[1][2]) * (r[2][1] - r[1][2])
            + (r[0][2] - r[2][0]) * (r[0][2] - r[2][0])
            + (r[1][0] - r[0][1]) * (r[1][0] - r[0][1]))
            .sqrt() as f64;
        if s.abs() < funcs::CMP_EPSILON {
            s = 1.0;
        }

        let x = ((r[2][1] - r[1][2]) as f64 / s) as f32;
        let y = ((r[0][2] - r[2][0]) as f64 / s) as f32;
        let z = ((r[1][0] - r[0][1]) as f64 / s) as f32;
        let angle = funcs::acos(((r[0][0] + r[1][1] + r[2][2] - 1.0) / 2.0) as f64) as f32;

        (Vector3::new(x, y, z), angle)
    }

    #[inline]
    pub fn is_diagonal(&self) -> bool {
        funcs::is_zero_approx(self.rows[0][1])
            && funcs::is_zero_approx(self.rows[0][2])
            && funcs::is_zero_approx(self.rows[1][0])
            && funcs::is_zero_approx(self.rows[1][2])
            && funcs::is_zero_approx(self.rows[2][0])
            && funcs::is_zero_approx(self.rows[2][1])
    }
    /// Rotates 9 spherical-harmonic coefficients in place
    /// (`Basis::rotate_sh`). Ported from John Hable's public-domain code that
    /// the engine uses, constants included.
    pub fn rotate_sh(&self, values: &mut [f32; 9]) {
        const C3: f32 = 0.94617469575;
        const C4: f32 = -0.31539156525;
        const C5: f32 = 0.54627421529;
        const C_SCALE: f32 = 1.0 / 0.91529123286551084;
        const C_SCALE_INV: f32 = 0.91529123286551084;
        const RC2: f32 = 1.5853309190550713 * C_SCALE;
        const C4_DIV_C3: f32 = C4 / C3;
        const C4_DIV_C3_X2: f32 = (C4 / C3) * 2.0;
        const SCALE_DST2: f32 = C3 * C_SCALE_INV;
        const SCALE_DST4: f32 = C5 * C_SCALE_INV;

        let src = *values;
        let m = &self.rows;

        values[0] = src[0];
        values[1] = m[1][1] * src[1] - m[1][2] * src[2] + m[1][0] * src[3];
        values[2] = -m[2][1] * src[1] + m[2][2] * src[2] - m[2][0] * src[3];
        values[3] = m[0][1] * src[1] - m[0][2] * src[2] + m[0][0] * src[3];

        let sh0 = src[7] + src[8] + src[8] - src[5];
        let sh1 = src[4] + RC2 * src[6] + src[7] + src[8];
        let sh2 = src[4];
        let sh3 = -src[7];
        let sh4 = -src[5];

        let r2x = m[0][0] + m[0][1];
        let r2y = m[1][0] + m[1][1];
        let r2z = m[2][0] + m[2][1];

        let r3x = m[0][0] + m[0][2];
        let r3y = m[1][0] + m[1][2];
        let r3z = m[2][0] + m[2][2];

        let r4x = m[0][1] + m[0][2];
        let r4y = m[1][1] + m[1][2];
        let r4z = m[2][1] + m[2][2];

        // Column 0.
        let sh0_x = sh0 * m[0][0];
        let sh0_y = sh0 * m[1][0];
        let mut d0 = sh0_x * m[1][0];
        let mut d1 = sh0_y * m[2][0];
        let mut d2 = sh0 * (m[2][0] * m[2][0] + C4_DIV_C3);
        let mut d3 = sh0_x * m[2][0];
        let mut d4 = sh0_x * m[0][0] - sh0_y * m[1][0];

        // Column 1.
        let sh1_x = sh1 * m[0][2];
        let sh1_y = sh1 * m[1][2];
        d0 += sh1_x * m[1][2];
        d1 += sh1_y * m[2][2];
        d2 += sh1 * (m[2][2] * m[2][2] + C4_DIV_C3);
        d3 += sh1_x * m[2][2];
        d4 += sh1_x * m[0][2] - sh1_y * m[1][2];

        // Column 2.
        let sh2_x = sh2 * r2x;
        let sh2_y = sh2 * r2y;
        d0 += sh2_x * r2y;
        d1 += sh2_y * r2z;
        d2 += sh2 * (r2z * r2z + C4_DIV_C3_X2);
        d3 += sh2_x * r2z;
        d4 += sh2_x * r2x - sh2_y * r2y;

        // Column 3.
        let sh3_x = sh3 * r3x;
        let sh3_y = sh3 * r3y;
        d0 += sh3_x * r3y;
        d1 += sh3_y * r3z;
        d2 += sh3 * (r3z * r3z + C4_DIV_C3_X2);
        d3 += sh3_x * r3z;
        d4 += sh3_x * r3x - sh3_y * r3y;

        // Column 4.
        let sh4_x = sh4 * r4x;
        let sh4_y = sh4 * r4y;
        d0 += sh4_x * r4y;
        d1 += sh4_y * r4z;
        d2 += sh4 * (r4z * r4z + C4_DIV_C3_X2);
        d3 += sh4_x * r4z;
        d4 += sh4_x * r4x - sh4_y * r4y;

        values[4] = d0;
        values[5] = -d1;
        values[6] = d2 * SCALE_DST2;
        values[7] = -d3;
        values[8] = d4 * SCALE_DST4;
    }
    /// Splits the basis into a rotation-reflection matrix (returned by `rotref`)
    /// and a positive scaling vector: `B = O * S`.
    pub fn rotref_posscale_decomposition(&self) -> (Basis, Vector3) {
        let scale = self.get_scale();
        let inv_scale = Basis::IDENTITY.scaled(scale.inverse());
        let rotref = *self * inv_scale;
        (rotref, scale.abs())
    }
    /// Jacobi diagonalization for symmetric matrices
    /// (`Basis::diagonalize`). Returns the accumulated rotation that
    /// diagonalizes the matrix in place.
    pub fn diagonalize(&mut self) -> Basis {
        const ITE_MAX: i32 = 1024;
        let mut off_matrix_norm_2 =
            self.rows[0][1] * self.rows[0][1]
                + self.rows[0][2] * self.rows[0][2]
                + self.rows[1][2] * self.rows[1][2];

        let mut ite = 0;
        let mut acc_rot = Basis::IDENTITY;

        while off_matrix_norm_2 > funcs::CMP_EPSILON2 as f32 && ite < ITE_MAX {
            ite += 1;

            let el01_2 = self.rows[0][1] * self.rows[0][1];
            let el02_2 = self.rows[0][2] * self.rows[0][2];
            let el12_2 = self.rows[1][2] * self.rows[1][2];

            // Find the pivot element.
            let (i, j);
            if el01_2 > el02_2 {
                if el12_2 > el01_2 {
                    i = 1;
                    j = 2;
                } else {
                    i = 0;
                    j = 1;
                }
            } else if el12_2 > el02_2 {
                i = 1;
                j = 2;
            } else {
                i = 0;
                j = 2;
            }

            // Rotation angle.
            let angle = if funcs::is_equal_approx(self.rows[j][j], self.rows[i][i]) {
                funcs::PI as f32 / 4.0
            } else {
                0.5 * (2.0 * self.rows[i][j] / (self.rows[j][j] - self.rows[i][i])).atan()
            };

            // Rotation matrix.
            let mut rot = Basis::IDENTITY;
            let (c, s) = (angle.cos(), angle.sin());
            rot.rows[i][i] = c;
            rot.rows[j][j] = c;
            rot.rows[i][j] = -s;
            rot.rows[j][i] = s;

            // Update the off-diagonal norm.
            off_matrix_norm_2 -= self.rows[i][j] * self.rows[i][j];

            // Apply the rotation.
            *self = rot * *self * rot.transposed();
            acc_rot = rot * acc_rot;
        }

        acc_rot
    }
    /// Decomposes the basis into Euler angles for the given order
    /// (`Basis::get_euler`). The epsilon matches the engine's, which truncates
    /// angles within ~0.04 degrees to the canonical singularity form.
    pub fn get_euler(&self, order: EulerOrder) -> Vector3 {
        const EPSILON: f32 = 0.00000025;
        let r = &self.rows;
        let mut euler = Vector3::ZERO;

        match order {
            EulerOrder::XYZ => {
                let sy = r[0][2];
                if sy < 1.0 - EPSILON {
                    if sy > -(1.0 - EPSILON) {
                        if r[1][0] == 0.0
                            && r[0][1] == 0.0
                            && r[1][2] == 0.0
                            && r[2][1] == 0.0
                            && r[1][1] == 1.0
                        {
                            euler.x = 0.0;
                            euler.y = r[0][2].atan2(r[0][0]);
                            euler.z = 0.0;
                        } else {
                            euler.x = (-r[1][2]).atan2(r[2][2]);
                            euler.y = funcs::asin(sy);
                            euler.z = (-r[0][1]).atan2(r[0][0]);
                        }
                    } else {
                        euler.x = r[2][1].atan2(r[1][1]);
                        euler.y = -funcs::PI as f32 / 2.0;
                        euler.z = 0.0;
                    }
                } else {
                    euler.x = r[2][1].atan2(r[1][1]);
                    euler.y = funcs::PI as f32 / 2.0;
                    euler.z = 0.0;
                }
            }
            EulerOrder::XZY => {
                let sz = r[0][1];
                if sz < 1.0 - EPSILON {
                    if sz > -(1.0 - EPSILON) {
                        euler.x = r[2][1].atan2(r[1][1]);
                        euler.y = r[0][2].atan2(r[0][0]);
                        euler.z = funcs::asin(-sz);
                    } else {
                        euler.x = -r[1][2].atan2(r[2][2]);
                        euler.y = 0.0;
                        euler.z = funcs::PI as f32 / 2.0;
                    }
                } else {
                    euler.x = -r[1][2].atan2(r[2][2]);
                    euler.y = 0.0;
                    euler.z = -(funcs::PI as f32) / 2.0;
                }
            }
            EulerOrder::YXZ => {
                let m12 = r[1][2];
                if m12 < 1.0 - EPSILON {
                    if m12 > -(1.0 - EPSILON) {
                        if r[1][0] == 0.0
                            && r[0][1] == 0.0
                            && r[0][2] == 0.0
                            && r[2][0] == 0.0
                            && r[0][0] == 1.0
                        {
                            euler.x = (-m12).atan2(r[1][1]);
                            euler.y = 0.0;
                            euler.z = 0.0;
                        } else {
                            euler.x = funcs::asin(-m12);
                            euler.y = r[0][2].atan2(r[2][2]);
                            euler.z = r[1][0].atan2(r[1][1]);
                        }
                    } else {
                        euler.x = funcs::PI as f32 * 0.5;
                        euler.y = r[0][1].atan2(r[0][0]);
                        euler.z = 0.0;
                    }
                } else {
                    euler.x = -(funcs::PI as f32) * 0.5;
                    euler.y = -r[0][1].atan2(r[0][0]);
                    euler.z = 0.0;
                }
            }
            EulerOrder::YZX => {
                let sz = r[1][0];
                if sz < 1.0 - EPSILON {
                    if sz > -(1.0 - EPSILON) {
                        euler.x = (-r[1][2]).atan2(r[1][1]);
                        euler.y = (-r[2][0]).atan2(r[0][0]);
                        euler.z = funcs::asin(sz);
                    } else {
                        euler.x = r[2][1].atan2(r[2][2]);
                        euler.y = 0.0;
                        euler.z = -(funcs::PI as f32) / 2.0;
                    }
                } else {
                    euler.x = r[2][1].atan2(r[2][2]);
                    euler.y = 0.0;
                    euler.z = funcs::PI as f32 / 2.0;
                }
            }
            EulerOrder::ZXY => {
                let sx = r[2][1];
                if sx < 1.0 - EPSILON {
                    if sx > -(1.0 - EPSILON) {
                        euler.x = funcs::asin(sx);
                        euler.y = (-r[2][0]).atan2(r[2][2]);
                        euler.z = (-r[0][1]).atan2(r[1][1]);
                    } else {
                        euler.x = -(funcs::PI as f32) / 2.0;
                        euler.y = r[0][2].atan2(r[0][0]);
                        euler.z = 0.0;
                    }
                } else {
                    euler.x = funcs::PI as f32 / 2.0;
                    euler.y = r[0][2].atan2(r[0][0]);
                    euler.z = 0.0;
                }
            }
            EulerOrder::ZYX => {
                let sy = r[2][0];
                if sy < 1.0 - EPSILON {
                    if sy > -(1.0 - EPSILON) {
                        euler.x = r[2][1].atan2(r[2][2]);
                        euler.y = funcs::asin(-sy);
                        euler.z = r[1][0].atan2(r[0][0]);
                    } else {
                        euler.x = 0.0;
                        euler.y = funcs::PI as f32 / 2.0;
                        euler.z = -r[0][1].atan2(r[1][1]);
                    }
                } else {
                    euler.x = 0.0;
                    euler.y = -(funcs::PI as f32) / 2.0;
                    euler.z = -r[0][1].atan2(r[1][1]);
                }
            }
        }

        euler
    }
    #[inline]
    pub fn set_euler(&mut self, euler: Vector3, order: EulerOrder) {
        *self = Self::from_euler(euler, order);
    }

    #[inline]
    pub fn lerp(&self, to: &Basis, weight: f32) -> Self {
        Self::from_rows(
            self.rows[0].lerp(to.rows[0], weight),
            self.rows[1].lerp(to.rows[1], weight),
            self.rows[2].lerp(to.rows[2], weight),
        )
    }

    /// Interpolates rotation (quaternion slerp) and scale (linear) separately.
    pub fn slerp(&self, to: &Basis, weight: f32) -> Self {
        let from_q = self.get_quaternion();
        let to_q = to.get_quaternion();
        let mut b = Self::from_quaternion(from_q.slerp(&to_q, weight));
        b.rows[0] *= funcs::lerp(self.rows[0].length(), to.rows[0].length(), weight);
        b.rows[1] *= funcs::lerp(self.rows[1].length(), to.rows[1].length(), weight);
        b.rows[2] *= funcs::lerp(self.rows[2].length(), to.rows[2].length(), weight);
        b
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Basis) -> bool {
        self.rows[0].is_equal_approx(other.rows[0])
            && self.rows[1].is_equal_approx(other.rows[1])
            && self.rows[2].is_equal_approx(other.rows[2])
    }

    #[inline]
    pub fn is_same(&self, other: &Basis) -> bool {
        self.rows[0].is_same(other.rows[0])
            && self.rows[1].is_same(other.rows[1])
            && self.rows[2].is_same(other.rows[2])
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.rows[0].is_finite() && self.rows[1].is_finite() && self.rows[2].is_finite()
    }

    /// `B` looking at `target` (used by `Node3D::look_at`).
    pub fn looking_at(target: Vector3, up: Vector3, use_model_front: bool) -> Self {
        let mut v_z = target.normalized();
        if !use_model_front {
            v_z = -v_z;
        }
        let mut v_x = up.cross(v_z);
        if v_x.is_zero_approx() {
            v_x = up.get_any_perpendicular();
        }
        v_x.normalize();
        let v_y = v_z.cross(v_x);

        Basis::from_columns(v_x, v_y, v_z)
    }
}

impl std::ops::Mul for Basis {
    type Output = Basis;
    /// `self * other` follows the engine's `operator*`, which is
    /// `other * self` in row-vector terms: it computes
    /// `set(other.tdotx(rows[0]), other.tdoty(rows[0]), ...)`.
    #[inline]
    fn mul(self, other: Basis) -> Basis {
        Basis::from_rows(
            Vector3::new(
                other.tdotx(self.rows[0]),
                other.tdoty(self.rows[0]),
                other.tdotz(self.rows[0]),
            ),
            Vector3::new(
                other.tdotx(self.rows[1]),
                other.tdoty(self.rows[1]),
                other.tdotz(self.rows[1]),
            ),
            Vector3::new(
                other.tdotx(self.rows[2]),
                other.tdoty(self.rows[2]),
                other.tdotz(self.rows[2]),
            ),
        )
    }
}

impl std::ops::Div<f32> for Basis {
    type Output = Basis;
    #[inline]
    fn div(self, s: f32) -> Basis {
        Basis::from_rows(self.rows[0] / s, self.rows[1] / s, self.rows[2] / s)
    }
}

impl std::ops::Mul<Vector3> for Basis {
    type Output = Vector3;
    #[inline]
    fn mul(self, v: Vector3) -> Vector3 {
        self.xform(v)
    }
}

impl std::ops::Mul<f32> for Basis {
    type Output = Basis;
    #[inline]
    fn mul(self, s: f32) -> Basis {
        Basis::from_rows(self.rows[0] * s, self.rows[1] * s, self.rows[2] * s)
    }
}

impl std::ops::Add for Basis {
    type Output = Basis;
    #[inline]
    fn add(self, o: Basis) -> Basis {
        Basis::from_rows(
            self.rows[0] + o.rows[0],
            self.rows[1] + o.rows[1],
            self.rows[2] + o.rows[2],
        )
    }
}

impl std::ops::Sub for Basis {
    type Output = Basis;
    #[inline]
    fn sub(self, o: Basis) -> Basis {
        Basis::from_rows(
            self.rows[0] - o.rows[0],
            self.rows[1] - o.rows[1],
            self.rows[2] - o.rows[2],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_math() {
        let i = Basis::IDENTITY;
        assert_eq!(i.determinant(), 1.0);
        assert_eq!(i.xform(Vector3::new(1.0, 2.0, 3.0)), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(i.inverse(), i);
        assert_eq!(i.transposed(), i);
    }

    #[test]
    fn inverse_roundtrip() {
        let b = Basis::from_euler(Vector3::new(0.3, -0.7, 1.1), EulerOrder::YXZ)
            .scaled_local(Vector3::new(2.0, 0.5, 1.5));
        let v = Vector3::new(0.4, -1.2, 2.5);
        let back = b.inverse().xform(b.xform(v));
        assert!(back.is_equal_approx(v), "{back:?}");
    }

    #[test]
    fn axis_angle_matches_quaternion() {
        let axis = Vector3::new(0.36, 0.48, 0.8);
        let angle = 0.77;
        let b = Basis::from_axis_angle(axis, angle);
        let q = Quaternion::from_axis_angle(axis, angle);
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert!(b.xform(v).is_equal_approx(q.xform(v)), "{:?} vs {:?}", b.xform(v), q.xform(v));
    }

    #[test]
    fn scale_extraction() {
        // M * S scales the columns, which is what get_scale() reads back.
        let b = Basis::from_euler(Vector3::new(0.2, 0.4, -0.3), EulerOrder::YXZ)
            .scaled_local(Vector3::new(3.0, 2.0, 0.5));
        let s = b.get_scale();
        assert!((s.x.abs() - 3.0).abs() < 1e-4, "{s:?}");
        assert!((s.y.abs() - 2.0).abs() < 1e-4, "{s:?}");
        assert!((s.z.abs() - 0.5).abs() < 1e-4, "{s:?}");
    }

    #[test]
    fn quaternion_roundtrip() {
        let b = Basis::from_euler(Vector3::new(0.1, 0.5, -0.9), EulerOrder::YXZ);
        let q = b.get_quaternion();
        let b2 = Basis::from_quaternion(q);
        assert!(b.is_equal_approx(&b2));
    }

    #[test]
    fn euler_orders_differ() {
        let e = Vector3::new(0.3, 0.4, 0.5);
        let a = Basis::from_euler(e, EulerOrder::XYZ);
        let b = Basis::from_euler(e, EulerOrder::ZYX);
        assert!(!a.is_equal_approx(&b));
        // YXZ is the engine default and must be reproducible via quaternion.
        let c = Basis::from_euler(e, EulerOrder::YXZ);
        let q = Quaternion::from_euler(e);
        assert!(c.is_equal_approx(&Basis::from_quaternion(q)));
    }
}
