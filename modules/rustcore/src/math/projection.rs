//! Port of `core/math/projection.h` + `projection.cpp` (camera projection).
//!
//! Storage is four columns, exactly like the engine, so `xform` matches the C++
//! arithmetic term by term.
//!
//! Pending: `invert`/`inverse` (full 4x4 cofactor inverse), `set_for_hmd`,
//! `get_projection_plane(s)`, `create_fit_aabb`, `scale_translate_to_fit` and
//! the depth-correction helpers.

use crate::math::funcs;
use crate::math::plane::Plane;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;
use crate::math::vector4::Vector4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub columns: [Vector4; 4],
}

impl Default for Projection {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Projection {
    pub const IDENTITY: Projection = Projection {
        columns: [
            Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
        ],
    };

    pub const ZERO: Projection = Projection {
        columns: [
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
        ],
    };

    #[inline]
    pub const fn new(x: Vector4, y: Vector4, z: Vector4, w: Vector4) -> Self {
        Self { columns: [x, y, z, w] }
    }

    /// `Projection::determinant`, term for term.
    #[allow(clippy::too_many_lines)]
    pub fn determinant(&self) -> f32 {
        let c = &self.columns;
        c[0][3] * c[1][2] * c[2][1] * c[3][0]
            - c[0][2] * c[1][3] * c[2][1] * c[3][0]
            - c[0][3] * c[1][1] * c[2][2] * c[3][0]
            + c[0][1] * c[1][3] * c[2][2] * c[3][0]
            + c[0][2] * c[1][1] * c[2][3] * c[3][0]
            - c[0][1] * c[1][2] * c[2][3] * c[3][0]
            - c[0][3] * c[1][2] * c[2][0] * c[3][1]
            + c[0][2] * c[1][3] * c[2][0] * c[3][1]
            + c[0][3] * c[1][0] * c[2][2] * c[3][1]
            - c[0][0] * c[1][3] * c[2][2] * c[3][1]
            - c[0][2] * c[1][0] * c[2][3] * c[3][1]
            + c[0][0] * c[1][2] * c[2][3] * c[3][1]
            + c[0][3] * c[1][1] * c[2][0] * c[3][2]
            - c[0][1] * c[1][3] * c[2][0] * c[3][2]
            - c[0][3] * c[1][0] * c[2][1] * c[3][2]
            + c[0][0] * c[1][3] * c[2][1] * c[3][2]
            + c[0][1] * c[1][0] * c[2][3] * c[3][2]
            - c[0][0] * c[1][1] * c[2][3] * c[3][2]
            - c[0][2] * c[1][1] * c[2][0] * c[3][3]
            + c[0][1] * c[1][2] * c[2][0] * c[3][3]
            + c[0][2] * c[1][0] * c[2][1] * c[3][3]
            - c[0][0] * c[1][2] * c[2][1] * c[3][3]
            - c[0][1] * c[1][0] * c[2][2] * c[3][3]
            + c[0][0] * c[1][1] * c[2][2] * c[3][3]
    }

    #[inline]
    pub fn xform(&self, v: Vector4) -> Vector4 {
        Vector4::new(
            self.columns[0][0] * v.x
                + self.columns[1][0] * v.y
                + self.columns[2][0] * v.z
                + self.columns[3][0] * v.w,
            self.columns[0][1] * v.x
                + self.columns[1][1] * v.y
                + self.columns[2][1] * v.z
                + self.columns[3][1] * v.w,
            self.columns[0][2] * v.x
                + self.columns[1][2] * v.y
                + self.columns[2][2] * v.z
                + self.columns[3][2] * v.w,
            self.columns[0][3] * v.x
                + self.columns[1][3] * v.y
                + self.columns[2][3] * v.z
                + self.columns[3][3] * v.w,
        )
    }

    #[inline]
    pub fn xform_inv(&self, v: Vector4) -> Vector4 {
        Vector4::new(
            self.columns[0][0] * v.x
                + self.columns[0][1] * v.y
                + self.columns[0][2] * v.z
                + self.columns[0][3] * v.w,
            self.columns[1][0] * v.x
                + self.columns[1][1] * v.y
                + self.columns[1][2] * v.z
                + self.columns[1][3] * v.w,
            self.columns[2][0] * v.x
                + self.columns[2][1] * v.y
                + self.columns[2][2] * v.z
                + self.columns[2][3] * v.w,
            self.columns[3][0] * v.x
                + self.columns[3][1] * v.y
                + self.columns[3][2] * v.z
                + self.columns[3][3] * v.w,
        )
    }

    /// Projects a 3D point and divides by w.
    #[inline]
    pub fn xform_vec3(&self, p: Vector3) -> Vector3 {
        let mut ret = Vector3::new(
            self.columns[0][0] * p.x
                + self.columns[1][0] * p.y
                + self.columns[2][0] * p.z
                + self.columns[3][0],
            self.columns[0][1] * p.x
                + self.columns[1][1] * p.y
                + self.columns[2][1] * p.z
                + self.columns[3][1],
            self.columns[0][2] * p.x
                + self.columns[1][2] * p.y
                + self.columns[2][2] * p.z
                + self.columns[3][2],
        );
        let w = self.columns[0][3] * p.x
            + self.columns[1][3] * p.y
            + self.columns[2][3] * p.z
            + self.columns[3][3];
        ret /= w;
        ret
    }

    pub fn set_perspective(
        &mut self,
        mut fovy_degrees: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        flip_fov: bool,
    ) {
        if flip_fov {
            fovy_degrees = get_fovy(fovy_degrees, 1.0 / aspect);
        }

        let radians = funcs::deg_to_rad(fovy_degrees / 2.0);
        let delta_z = z_far - z_near;
        let sine = radians.sin();

        if delta_z == 0.0 || sine == 0.0 || aspect == 0.0 {
            return;
        }
        let cotangent = radians.cos() / sine;

        *self = Self::IDENTITY;
        self.columns[0][0] = cotangent / aspect;
        self.columns[1][1] = cotangent;
        self.columns[2][2] = -(z_far + z_near) / delta_z;
        self.columns[2][3] = -1.0;
        self.columns[3][2] = -2.0 * z_near * z_far / delta_z;
        self.columns[3][3] = 0.0;
    }

    pub fn create_perspective(
        fovy_degrees: f32,
        aspect: f32,
        z_near: f32,
        z_far: f32,
        flip_fov: bool,
    ) -> Self {
        let mut p = Self::IDENTITY;
        p.set_perspective(fovy_degrees, aspect, z_near, z_far, flip_fov);
        p
    }

    pub fn set_orthogonal(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) {
        *self = Self::IDENTITY;
        self.columns[0][0] = 2.0 / (right - left);
        self.columns[3][0] = -((right + left) / (right - left));
        self.columns[1][1] = 2.0 / (top - bottom);
        self.columns[3][1] = -((top + bottom) / (top - bottom));
        self.columns[2][2] = -2.0 / (zfar - znear);
        self.columns[3][2] = -((zfar + znear) / (zfar - znear));
        self.columns[3][3] = 1.0;
    }

    pub fn create_orthogonal(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut p = Self::IDENTITY;
        p.set_orthogonal(left, right, bottom, top, znear, zfar);
        p
    }

    /// Note: the C++ version refuses to build an invalid frustum; here an
    /// invalid frustum produces the identity, matching that early return.
    pub fn set_frustum(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        if right <= left || top <= bottom || far <= near {
            return;
        }

        let x = 2.0 * near / (right - left);
        let y = 2.0 * near / (top - bottom);
        let a = (right + left) / (right - left);
        let b = (top + bottom) / (top - bottom);
        let c = -(far + near) / (far - near);
        let d = -2.0 * far * near / (far - near);

        self.columns[0] = Vector4::new(x, 0.0, 0.0, 0.0);
        self.columns[1] = Vector4::new(0.0, y, 0.0, 0.0);
        self.columns[2] = Vector4::new(a, b, c, -1.0);
        self.columns[3] = Vector4::new(0.0, 0.0, d, 0.0);
    }

    pub fn create_frustum(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut p = Self::IDENTITY;
        p.set_frustum(left, right, bottom, top, near, far);
        p
    }

    /// HMD per-eye frustum (`Projection::set_for_hmd`). Eye 1 is left, 2 is
    /// right; anything else leaves the projection untouched (mono).
    #[allow(clippy::too_many_arguments)]
    pub fn set_for_hmd(
        &mut self,
        eye: i32,
        aspect: f32,
        intraocular_dist: f32,
        display_width: f32,
        display_to_lens: f32,
        oversample: f32,
        z_near: f32,
        z_far: f32,
    ) {
        // Base frustum without lens magnification.
        let mut f1 = (intraocular_dist * 0.5) / display_to_lens;
        let mut f2 = ((display_width - intraocular_dist) * 0.5) / display_to_lens;
        let mut f3 = (display_width / 4.0) / display_to_lens;

        // Oversample widens the FOV.
        let add = ((f1 + f2) * (oversample - 1.0)) / 2.0;
        f1 += add;
        f2 += add;
        f3 *= oversample;

        // Always KEEP_WIDTH aspect.
        f3 /= aspect;

        match eye {
            1 => {
                // Left eye.
                self.set_frustum(-f2 * z_near, f1 * z_near, -f3 * z_near, f3 * z_near, z_near, z_far);
            }
            2 => {
                // Right eye.
                self.set_frustum(-f1 * z_near, f2 * z_near, -f3 * z_near, f3 * z_near, z_near, z_far);
            }
            _ => {
                // Mono: does not apply here.
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_for_hmd(
        eye: i32,
        aspect: f32,
        intraocular_dist: f32,
        display_width: f32,
        display_to_lens: f32,
        oversample: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        let mut p = Self::IDENTITY;
        p.set_for_hmd(
            eye,
            aspect,
            intraocular_dist,
            display_width,
            display_to_lens,
            oversample,
            z_near,
            z_far,
        );
        p
    }
    /// Half extents of the view at the near plane (`get_viewport_half_extents`).
    /// Assumes a symmetrical, rectangular frustum.
    pub fn get_viewport_half_extents(&self) -> crate::math::vector2::Vector2 {
        let w = -self.get_z_near() * self.columns[2][3] + self.columns[3][3];
        crate::math::vector2::Vector2::new(w / self.columns[0][0], w / self.columns[1][1])
    }

    /// Half extents at the far plane.
    pub fn get_far_plane_half_extents(&self) -> crate::math::vector2::Vector2 {
        let w = -self.get_z_far() * self.columns[2][3] + self.columns[3][3];
        crate::math::vector2::Vector2::new(w / self.columns[0][0], w / self.columns[1][1])
    }

    /// Pixels per world unit at the near plane (`get_pixels_per_meter`). The
    /// engine returns an `int`, truncating.
    pub fn get_pixels_per_meter(&self, for_pixel_width: i32) -> i32 {
        let width =
            2.0 * (-self.get_z_near() * self.columns[2][3] + self.columns[3][3]) / self.columns[0][0];
        (for_pixel_width as f32 / width) as i32
    }

    /// `get_lod_multiplier`: `2 / columns[0][0]`.
    #[inline]
    pub fn get_lod_multiplier(&self) -> f32 {
        2.0 / self.columns[0][0]
    }

    /// `make_scale`: identity with the scale on the diagonal.
    pub fn make_scale(&mut self, scale: Vector3) {
        *self = Self::IDENTITY;
        self.columns[0][0] = scale.x;
        self.columns[1][1] = scale.y;
        self.columns[2][2] = scale.z;
    }
    /// Builds an orthographic projection that maps `aabb` to the clip cube
    /// (`Projection::scale_translate_to_fit`).
    pub fn scale_translate_to_fit(&mut self, aabb: &crate::math::aabb::Aabb) {
        let min = aabb.position;
        let max = aabb.position + aabb.size;

        let sx = 2.0 / (max.x - min.x);
        let sy = 2.0 / (max.y - min.y);
        let sz = 2.0 / (max.z - min.z);

        self.columns[0] = Vector4::new(sx, 0.0, 0.0, 0.0);
        self.columns[1] = Vector4::new(0.0, sy, 0.0, 0.0);
        self.columns[2] = Vector4::new(0.0, 0.0, sz, 0.0);
        self.columns[3] = Vector4::new(
            -(max.x + min.x) / (max.x - min.x),
            -(max.y + min.y) / (max.y - min.y),
            -(max.z + min.z) / (max.z - min.z),
            1.0,
        );
    }
    /// The six frustum planes transformed into world space
    /// (`Projection::get_projection_planes`). Order matches `Planes`:
    /// near, far, left, top, right, bottom.
    pub fn get_projection_planes(&self, transform: &Transform3D) -> [Plane; 6] {
        let mut out = [Plane::default(); 6];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = transform.xform_plane(&self.get_projection_plane(i));
        }
        out
    }
    /// `create_fit_aabb`: an identity projection fitted to `aabb`.
    pub fn create_fit_aabb(aabb: &crate::math::aabb::Aabb) -> Self {
        let mut p = Self::IDENTITY;
        p.scale_translate_to_fit(aabb);
        p
    }

    /// `perspective_znear_adjusted`: a copy with a different near plane.
    pub fn perspective_znear_adjusted(&self, new_znear: f32) -> Self {
        let mut p = *self;
        p.adjust_perspective_znear(new_znear);
        p
    }

    /// Rewrites the depth rows for a new near plane, keeping the far plane
    /// (`Projection::adjust_perspective_znear`).
    pub fn adjust_perspective_znear(&mut self, new_znear: f32) {
        let zfar = self.get_z_far();
        let znear = new_znear;
        let delta_z = zfar - znear;

        self.columns[2][2] = -(zfar + znear) / delta_z;
        self.columns[3][2] = -2.0 * znear * zfar / delta_z;
    }
    /// One of the six frustum planes, built from the matrix rows like the engine.
    /// The enum order matches `Projection::Planes` (near, far, left, top, right, bottom).
    pub fn get_projection_plane(&self, plane: usize) -> Plane {
        // matrix[i] in the C++ code is columns[i / 4][i % 4].
        let m = |i: usize| self.columns[i / 4][i % 4];

        let (nx, ny, nz, d) = match plane {
            0 => {
                // Near: row3 + row2
                let p = (m(3) + m(2), m(7) + m(6), m(11) + m(10), m(15) + m(14));
                (-p.0, -p.1, -p.2, p.3)
            }
            1 => {
                // Far: row3 - row2
                let p = (m(3) - m(2), m(7) - m(6), m(11) - m(10), m(15) - m(14));
                (-p.0, -p.1, -p.2, p.3)
            }
            2 => {
                // Left: row3 + row0
                let p = (m(3) + m(0), m(7) + m(4), m(11) + m(8), m(15) + m(12));
                (-p.0, -p.1, -p.2, p.3)
            }
            3 => {
                // Top: row3 - row1
                let p = (m(3) - m(1), m(7) - m(5), m(11) - m(9), m(15) - m(13));
                (-p.0, -p.1, -p.2, p.3)
            }
            4 => {
                // Right: row3 - row0
                let p = (m(3) - m(0), m(7) - m(4), m(11) - m(8), m(15) - m(12));
                (-p.0, -p.1, -p.2, p.3)
            }
            _ => {
                // Bottom: row3 + row1
                let p = (m(3) + m(1), m(7) + m(5), m(11) + m(9), m(15) + m(13));
                (-p.0, -p.1, -p.2, p.3)
            }
        };

        let mut plane = Plane::new(nx, ny, nz, d);
        plane.normalize();
        plane
    }
    #[inline]
    pub fn get_z_far(&self) -> f32 {
        (self.columns[3][3] - self.columns[3][2]) / (self.columns[2][3] - self.columns[2][2])
    }

    #[inline]
    pub fn get_z_near(&self) -> f32 {
        (self.columns[3][3] + self.columns[3][2]) / (self.columns[2][3] + self.columns[2][2])
    }

    #[inline]
    pub fn get_aspect(&self) -> f32 {
        self.columns[1][1] / self.columns[0][0]
    }

    pub fn get_fov(&self) -> f32 {
        if self.columns[2][0] == 0.0 {
            funcs::rad_to_deg(2.0 * funcs::atan2(1.0, self.columns[0][0]))
        } else {
            let right = funcs::atan2(self.columns[2][0] + 1.0, self.columns[0][0]);
            let left = funcs::atan2(self.columns[2][0] - 1.0, self.columns[0][0]);
            funcs::rad_to_deg(right - left)
        }
    }

    #[inline]
    pub fn is_orthogonal(&self) -> bool {
        self.columns[2][3] == 0.0
    }

    /// Full 4x4 inverse (Mesa's `util_invert_mat4x4`, same as the engine).
    /// A singular matrix leaves `self` untouched, matching `ERR_FAIL_COND`.
    #[allow(clippy::needless_range_loop)]
    pub fn invert(&mut self) {
        let columns = self.columns;
        // MAT(m, r, c) in the C++ code is column-major: columns[c][r].
        let mat = |r: usize, c: usize| columns[c][r];

        let mut r = [[0.0f32; 8]; 4];
        for i in 0..4 {
            r[i][0] = mat(i, 0);
            r[i][1] = mat(i, 1);
            r[i][2] = mat(i, 2);
            r[i][3] = mat(i, 3);
            r[i][4 + i] = 1.0;
        }

        // Pivot for the first variable.
        if r[3][0].abs() > r[2][0].abs() {
            r.swap(3, 2);
        }
        if r[2][0].abs() > r[1][0].abs() {
            r.swap(2, 1);
        }
        if r[1][0].abs() > r[0][0].abs() {
            r.swap(1, 0);
        }
        if r[0][0] == 0.0 {
            return;
        }

        // Eliminate the first variable.
        let m1 = r[1][0] / r[0][0];
        let m2 = r[2][0] / r[0][0];
        let m3 = r[3][0] / r[0][0];
        for c in 1..4 {
            let s = r[0][c];
            r[1][c] -= m1 * s;
            r[2][c] -= m2 * s;
            r[3][c] -= m3 * s;
        }
        for c in 4..8 {
            let s = r[0][c];
            if s != 0.0 {
                r[1][c] -= m1 * s;
                r[2][c] -= m2 * s;
                r[3][c] -= m3 * s;
            }
        }

        // Pivot for the second variable.
        if r[3][1].abs() > r[2][1].abs() {
            r.swap(3, 2);
        }
        if r[2][1].abs() > r[1][1].abs() {
            r.swap(2, 1);
        }
        if r[1][1] == 0.0 {
            return;
        }

        // Eliminate the second variable.
        let m2 = r[2][1] / r[1][1];
        let m3 = r[3][1] / r[1][1];
        for c in 2..4 {
            r[2][c] -= m2 * r[1][c];
            r[3][c] -= m3 * r[1][c];
        }
        for c in 4..8 {
            let s = r[1][c];
            if s != 0.0 {
                r[2][c] -= m2 * s;
                r[3][c] -= m3 * s;
            }
        }

        // Pivot for the third variable.
        if r[3][2].abs() > r[2][2].abs() {
            r.swap(3, 2);
        }
        if r[2][2] == 0.0 {
            return;
        }

        // Eliminate the third variable.
        let m3 = r[3][2] / r[2][2];
        for c in 3..8 {
            r[3][c] -= m3 * r[2][c];
        }

        if r[3][3] == 0.0 {
            return;
        }

        // Back substitute row 3.
        let s = 1.0 / r[3][3];
        for c in 4..8 {
            r[3][c] *= s;
        }

        // Back substitute row 2.
        let m2 = r[2][3];
        let s = 1.0 / r[2][2];
        for c in 4..8 {
            r[2][c] = s * (r[2][c] - r[3][c] * m2);
        }
        let m1 = r[1][3];
        let m0 = r[0][3];
        for c in 4..8 {
            r[1][c] -= r[3][c] * m1;
            r[0][c] -= r[3][c] * m0;
        }

        // Back substitute row 1.
        let m1 = r[1][2];
        let s = 1.0 / r[1][1];
        for c in 4..8 {
            r[1][c] = s * (r[1][c] - r[2][c] * m1);
        }
        let m0 = r[0][2];
        for c in 4..8 {
            r[0][c] -= r[2][c] * m0;
        }

        // Back substitute row 0.
        let m0 = r[0][1];
        let s = 1.0 / r[0][0];
        for c in 4..8 {
            r[0][c] = s * (r[0][c] - r[1][c] * m0);
        }

        // MAT(out, r, c) = columns[c][r]
        for i in 0..4 {
            self.columns[i] = Vector4::new(r[0][4 + i], r[1][4 + i], r[2][4 + i], r[3][4 + i]);
        }
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let mut p = *self;
        p.invert();
        p
    }

    pub fn flip_y(&mut self) {
        for i in 0..4 {
            self.columns[1][i] = -self.columns[1][i];
        }
    }

    #[inline]
    pub fn flipped_y(&self) -> Self {
        let mut p = *self;
        p.flip_y();
        p
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Projection) -> bool {
        self.columns[0].is_equal_approx(other.columns[0])
            && self.columns[1].is_equal_approx(other.columns[1])
            && self.columns[2].is_equal_approx(other.columns[2])
            && self.columns[3].is_equal_approx(other.columns[3])
    }

    #[inline]
    pub fn is_same(&self, other: &Projection) -> bool {
        self.columns[0].is_same(other.columns[0])
            && self.columns[1].is_same(other.columns[1])
            && self.columns[2].is_same(other.columns[2])
            && self.columns[3].is_same(other.columns[3])
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.columns[0].is_finite()
            && self.columns[1].is_finite()
            && self.columns[2].is_finite()
            && self.columns[3].is_finite()
    }
}

/// `Projection::get_fovy`: converts a horizontal FOV into a vertical one.
#[inline]
pub fn get_fovy(fovx: f32, aspect: f32) -> f32 {
    funcs::rad_to_deg(
        funcs::atan2(
            aspect * funcs::tan(funcs::deg_to_rad(fovx) * 0.5),
            1.0f32,
        ) * 2.0,
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_roundtrips_near_far() {
        let p = Projection::create_perspective(75.0, 16.0 / 9.0, 0.05, 1000.0, false);
        assert!((p.get_z_near() - 0.05).abs() < 1e-4, "{}", p.get_z_near());
        // Recovering z_far from the matrix loses precision in f32.\n        assert!((p.get_z_far() - 1000.0).abs() < 1.0, "{}", p.get_z_far());
        assert!(!p.is_orthogonal());
        assert!((p.get_aspect() - 16.0 / 9.0).abs() < 1e-4);
    }

    #[test]
    fn orthogonal_is_orthogonal() {
        let p = Projection::create_orthogonal(-2.0, 2.0, -1.0, 1.0, 0.1, 100.0);
        assert!(p.is_orthogonal());
        // Center of the view maps to x = y = 0; z stays at the depth-map value.
        let v = p.xform_vec3(Vector3::ZERO);
        assert!(v.x.abs() < 1e-6 && v.y.abs() < 1e-6, "{v:?}");
    }

    #[test]
    fn perspective_projection_center() {
        let p = Projection::create_perspective(70.0, 1.5, 0.1, 500.0, false);
        // A point in front of the camera maps to the center of the screen.
        let v = p.xform_vec3(Vector3::new(0.0, 0.0, -10.0));
        assert!(v.x.abs() < 1e-6 && v.y.abs() < 1e-6, "{v:?}");
    }

    #[test]
    fn inverse_roundtrip() {
        let p = Projection::create_perspective(60.0, 1.5, 0.1, 100.0, false);
        let inv = p.inverse();
        let v = Vector4::new(0.3, -0.7, -5.0, 1.0);
        let fwd = p.xform(v);
        let back = inv.xform(fwd);
        let got = Vector4::new(back.x / back.w, back.y / back.w, back.z / back.w, 1.0);
        assert!((got.x - v.x).abs() < 1e-4, "{got:?}");
        assert!((got.y - v.y).abs() < 1e-4, "{got:?}");
        assert!((got.z - v.z).abs() < 1e-3, "{got:?}");
    }

    #[test]
    fn flip_y_negates_second_column() {
        let p = Projection::create_perspective(60.0, 1.0, 0.1, 100.0, false);
        let f = p.flipped_y();
        assert_eq!(f.columns[1].y, -p.columns[1].y);
        assert_eq!(f.columns[0], p.columns[0]);
    }
}
