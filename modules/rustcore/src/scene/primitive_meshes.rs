//! `scene/resources/3d/primitive_meshes.*`: the primitive mesh parameters.
//!
//! `PrimitiveMesh` adds `flip_faces` (off), `add_uv2` (off), `uv2_padding` (2.0)
//! and the lightmap `texel_size` (0.2) to `Mesh`.
//!
//! Every concrete primitive's constructor defaults are reproduced, and the
//! setters keep the engine's guards:
//!
//! - `CapsuleMesh`/`CylinderMesh`: `rings >= 0`;
//! - `SphereMesh`: `rings >= 1`;
//! - `TorusMesh`: `rings >= 3`, `ring_segments >= 3`, and `inner_radius` must
//!   differ from `outer_radius` ("Inner radius and outer radius cannot be the
//!   same.").
//!
//! Everything else (radii, heights, subdivisions, `left_to_right`) is stored
//! unvalidated, like the C++.
//!
//! Mesh generation (`_create_mesh_array`), surface creation and the lightmap UV2
//! layout are not ported; the parameters and their guards are.

use crate::math::vector3::Vector3;

/// `PrimitiveMesh`'s shared parameters.
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveMeshParams {
    pub flip_faces: bool,
    pub add_uv2: bool,
    pub uv2_padding: f32,
    pub texel_size: f32,
}

impl Default for PrimitiveMeshParams {
    fn default() -> Self {
        Self {
            flip_faces: false,
            add_uv2: false,
            uv2_padding: 2.0,
            texel_size: 0.2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CapsuleMeshData {
    pub params: PrimitiveMeshParams,
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
}

impl Default for CapsuleMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            radius: 0.5,
            height: 2.0,
            radial_segments: 64,
            rings: 8,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        // CylinderMesh clamps to at least four segments.
        self.radial_segments = segments.max(4);
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 0)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 0 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }
}

#[derive(Clone, Debug)]
pub struct BoxMeshData {
    pub params: PrimitiveMeshParams,
    size: Vector3,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
}

impl Default for BoxMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            size: Vector3::new(1.0, 1.0, 1.0),
            subdivide_w: 0,
            subdivide_h: 0,
            subdivide_d: 0,
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_height(&mut self, subdivide: i32) {
        self.subdivide_h = subdivide;
    }

    pub fn get_subdivide_height(&self) -> i32 {
        self.subdivide_h
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }
}

#[derive(Clone, Debug)]
pub struct CylinderMeshData {
    pub params: PrimitiveMeshParams,
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    cap_top: bool,
    cap_bottom: bool,
}

impl Default for CylinderMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl CylinderMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            top_radius: 0.5,
            bottom_radius: 0.5,
            height: 2.0,
            radial_segments: 64,
            rings: 4,
            cap_top: true,
            cap_bottom: true,
        }
    }

    pub fn set_top_radius(&mut self, radius: f32) {
        self.top_radius = radius;
    }

    pub fn get_top_radius(&self) -> f32 {
        self.top_radius
    }

    pub fn set_bottom_radius(&mut self, radius: f32) {
        self.bottom_radius = radius;
    }

    pub fn get_bottom_radius(&self) -> f32 {
        self.bottom_radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        // CylinderMesh clamps to at least four segments.
        self.radial_segments = segments.max(4);
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 0)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 0 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    pub fn set_cap_top(&mut self, enabled: bool) {
        self.cap_top = enabled;
    }

    pub fn is_cap_top(&self) -> bool {
        self.cap_top
    }

    pub fn set_cap_bottom(&mut self, enabled: bool) {
        self.cap_bottom = enabled;
    }

    pub fn is_cap_bottom(&self) -> bool {
        self.cap_bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PlaneOrientation {
    FaceX,
    FaceY,
    FaceZ,
}

/// `_create_mesh_array` output for the ported generators.
#[derive(Clone, Debug, Default)]
pub struct GeneratedArrays {
    pub points: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    /// Four floats per vertex (`xyz`, `w`).
    pub tangents: Vec<f32>,
    pub uvs: Vec<crate::math::vector2::Vector2>,
    /// Filled by the box generator when dd_uv2 is set.
    pub uv2s: Vec<crate::math::vector2::Vector2>,
    pub indices: Vec<i32>,
}

/// `PlaneMesh::_create_mesh_array`.
pub fn generate_plane_arrays(
    size: crate::math::vector2::Vector2,
    subdivide_w: i32,
    subdivide_d: i32,
    center_offset: Vector3,
    orientation: PlaneOrientation,
) -> GeneratedArrays {
    let start_x = size.x * -0.5;
    let start_z = size.y * -0.5;

    let normal = match orientation {
        PlaneOrientation::FaceX => Vector3::new(1.0, 0.0, 0.0),
        PlaneOrientation::FaceY => Vector3::new(0.0, 1.0, 0.0),
        PlaneOrientation::FaceZ => Vector3::new(0.0, 0.0, 1.0),
    };

    let mut arrays = GeneratedArrays::default();
    let num_points = ((subdivide_d + 2) * (subdivide_w + 2)) as usize;
    arrays.points.reserve(num_points);
    arrays.normals.reserve(num_points);
    arrays.tangents.reserve(num_points * 4);
    arrays.uvs.reserve(num_points);
    arrays
        .indices
        .reserve(((subdivide_d + 1) * (subdivide_w + 1) * 6) as usize);

    let mut point = 0i32;
    let mut z = start_z;
    let mut thisrow = point;
    let mut prevrow = 0i32;

    for j in 0..=(subdivide_d + 1) {
        let mut x = start_x;
        for i in 0..=(subdivide_w + 1) {
            let u = i as f32 / (subdivide_w + 1) as f32;
            let v = j as f32 / (subdivide_d + 1) as f32;

            let position = match orientation {
                PlaneOrientation::FaceX => Vector3::new(0.0, z, x),
                PlaneOrientation::FaceY => Vector3::new(-x, 0.0, -z),
                PlaneOrientation::FaceZ => Vector3::new(-x, z, 0.0),
            };
            arrays.points.push(position + center_offset);
            arrays.normals.push(normal);
            if orientation == PlaneOrientation::FaceX {
                arrays.tangents.extend_from_slice(&[0.0, 0.0, -1.0, 1.0]);
            } else {
                arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            }
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(1.0 - u, 1.0 - v));
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }

            x += size.x / (subdivide_w + 1) as f32;
        }

        z += size.y / (subdivide_d + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    arrays
}

/// `BoxMesh::create_mesh_array`, including the optional UV2 layout.
pub fn generate_box_arrays(
    size: Vector3,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let onethird = 1.0f32 / 3.0;
    let twothirds = 2.0f32 / 3.0;

    let total_h = size.x + size.z + (2.0 * uv2_padding);
    let padding_h = uv2_padding / total_h;
    let width_h = size.x / total_h;
    let depth_h = size.z / total_h;
    let total_v = size.y + size.y + size.x.max(size.z) + (3.0 * uv2_padding);
    let padding_v = uv2_padding / total_v;
    let width_v = size.x / total_v;
    let height_v = size.y / total_v;
    let depth_v = size.z / total_v;

    let start_pos = size * -0.5;
    let mut arrays = GeneratedArrays::default();
    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut point = 0i32;

    // front + back
    let mut y = start_pos.y;
    let mut thisrow = point;
    let mut prevrow = 0i32;
    for j in 0..=(subdivide_h + 1) {
        let mut v = j as f32;
        let v2 = v / (subdivide_h + 1) as f32;
        v /= 2.0 * (subdivide_h + 1) as f32;

        let mut x = start_pos.x;
        for i in 0..=(subdivide_w + 1) {
            let mut u = i as f32;
            let u2 = u / (subdivide_w + 1) as f32;
            u /= 3.0 * (subdivide_w + 1) as f32;

            // front
            arrays
                .points
                .push(Vector3::new(x, -y, -start_pos.z)); // double negative on Z
            arrays.normals.push(Vector3::new(0.0, 0.0, 1.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * width_h,
                    v2 * height_v,
                ));
            }
            point += 1;

            // back
            arrays
                .points
                .push(Vector3::new(-x, -y, start_pos.z));
            arrays.normals.push(Vector3::new(0.0, 0.0, -1.0));
            arrays.tangents.extend_from_slice(&[-1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(twothirds + u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * width_h,
                    height_v + padding_v + (v2 * height_v),
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                let i2 = i * 2;
                push_quad(&mut arrays.indices, prevrow + i2 - 2, prevrow + i2, thisrow + i2 - 2, prevrow + i2, thisrow + i2, thisrow + i2 - 2);
                push_quad(&mut arrays.indices, prevrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 + 1, thisrow + i2 - 1);
            }

            x += size.x / (subdivide_w + 1) as f32;
        }

        y += size.y / (subdivide_h + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    // left + right
    y = start_pos.y;
    thisrow = point;
    prevrow = 0;
    for j in 0..=(subdivide_h + 1) {
        let mut v = j as f32;
        let v2 = v / (subdivide_h + 1) as f32;
        v /= 2.0 * (subdivide_h + 1) as f32;

        let mut z = start_pos.z;
        for i in 0..=(subdivide_d + 1) {
            let mut u = i as f32;
            let u2 = u / (subdivide_d + 1) as f32;
            u /= 3.0 * (subdivide_d + 1) as f32;

            // right
            arrays
                .points
                .push(Vector3::new(-start_pos.x, -y, -z));
            arrays.normals.push(Vector3::new(1.0, 0.0, 0.0));
            arrays.tangents.extend_from_slice(&[0.0, 0.0, -1.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(onethird + u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    width_h + padding_h + (u2 * depth_h),
                    v2 * height_v,
                ));
            }
            point += 1;

            // left
            arrays
                .points
                .push(Vector3::new(start_pos.x, -y, z));
            arrays.normals.push(Vector3::new(-1.0, 0.0, 0.0));
            arrays.tangents.extend_from_slice(&[0.0, 0.0, 1.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, 0.5 + v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    width_h + padding_h + (u2 * depth_h),
                    height_v + padding_v + (v2 * height_v),
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                let i2 = i * 2;
                push_quad(&mut arrays.indices, prevrow + i2 - 2, prevrow + i2, thisrow + i2 - 2, prevrow + i2, thisrow + i2, thisrow + i2 - 2);
                push_quad(&mut arrays.indices, prevrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 + 1, thisrow + i2 - 1);
            }

            z += size.z / (subdivide_d + 1) as f32;
        }

        y += size.y / (subdivide_h + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    // top + bottom
    let mut z = start_pos.z;
    thisrow = point;
    prevrow = 0;
    for j in 0..=(subdivide_d + 1) {
        let mut v = j as f32;
        let v2 = v / (subdivide_d + 1) as f32;
        v /= 2.0 * (subdivide_d + 1) as f32;

        let mut x = start_pos.x;
        for i in 0..=(subdivide_w + 1) {
            let mut u = i as f32;
            let u2 = u / (subdivide_w + 1) as f32;
            u /= 3.0 * (subdivide_w + 1) as f32;

            // top
            arrays
                .points
                .push(Vector3::new(-x, -start_pos.y, -z));
            arrays.normals.push(Vector3::new(0.0, 1.0, 0.0));
            arrays.tangents.extend_from_slice(&[-1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(onethird + u, 0.5 + v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * width_h,
                    ((height_v + padding_v) * 2.0) + (v2 * depth_v),
                ));
            }
            point += 1;

            // bottom
            arrays
                .points
                .push(Vector3::new(x, start_pos.y, -z));
            arrays.normals.push(Vector3::new(0.0, -1.0, 0.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(twothirds + u, 0.5 + v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    width_h + padding_h + (u2 * depth_h),
                    ((height_v + padding_v) * 2.0) + (v2 * width_v),
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                let i2 = i * 2;
                push_quad(&mut arrays.indices, prevrow + i2 - 2, prevrow + i2, thisrow + i2 - 2, prevrow + i2, thisrow + i2, thisrow + i2 - 2);
                push_quad(&mut arrays.indices, prevrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 - 1, prevrow + i2 + 1, thisrow + i2 + 1, thisrow + i2 - 1);
            }

            x += size.x / (subdivide_w + 1) as f32;
        }

        z += size.z / (subdivide_d + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

fn push_quad(indices: &mut Vec<i32>, a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {
    indices.push(a);
    indices.push(b);
    indices.push(c);
    indices.push(d);
    indices.push(e);
    indices.push(f);
}

/// `SphereMesh::create_mesh_array`.
pub fn generate_sphere_arrays(
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    is_hemisphere: bool,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let tau = std::f32::consts::TAU;
    let pi = std::f32::consts::PI;
    let scale = height / radius * if is_hemisphere { 1.0 } else { 0.5 };

    let circumference = radius * tau;
    let horizontal_length = circumference + uv2_padding;
    let center_h = 0.5 * circumference / horizontal_length;
    let height_v = scale * pi / ((scale * pi) + uv2_padding / radius);

    let mut arrays = GeneratedArrays::default();
    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut point = 0i32;
    let mut thisrow = 0i32;
    let mut prevrow = 0i32;

    for j in 0..=(rings + 1) {
        let mut v = j as f32;
        v /= (rings + 1) as f32;
        let (w, y) = if j == rings + 1 {
            (0.0, -1.0)
        } else {
            ((pi * v).sin(), (pi * v).cos())
        };

        for i in 0..=radial_segments {
            let mut u = i as f32;
            u /= radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                ((u * tau).sin(), (u * tau).cos())
            };

            if is_hemisphere && y < 0.0 {
                arrays
                    .points
                    .push(Vector3::new(x * radius * w, 0.0, z * radius * w));
                arrays.normals.push(Vector3::new(0.0, -1.0, 0.0));
            } else {
                let p = Vector3::new(x * w, y * scale, z * w);
                arrays.points.push(p * radius);
                arrays.normals.push(
                    Vector3::new(x * w * scale, y, z * w * scale).normalized(),
                );
            }
            arrays.tangents.extend_from_slice(&[z, 0.0, -x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v));
            if add_uv2 {
                let w_h = w * 2.0 * center_h;
                uv2s.push(crate::math::vector2::Vector2::new(
                    center_h + ((u - 0.5) * w_h),
                    v * height_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }
        }

        prevrow = thisrow;
        thisrow = point;
    }

    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

/// `CylinderMesh::create_mesh_array`.
#[allow(clippy::too_many_arguments)]
pub fn generate_cylinder_arrays(
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    cap_top: bool,
    cap_bottom: bool,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let tau = std::f32::consts::TAU;
    let top_circumference = top_radius * std::f32::consts::PI * 2.0;
    let bottom_circumference = bottom_radius * std::f32::consts::PI * 2.0;
    let vertical_length =
        height + (2.0 * top_radius).max(2.0 * bottom_radius) + (2.0 * uv2_padding);
    let height_v = height / vertical_length;
    let padding_v = uv2_padding / vertical_length;

    let horizontal_length = (2.0 * (top_radius + bottom_radius + uv2_padding))
        .max(top_circumference + uv2_padding)
        .max(bottom_circumference + uv2_padding);
    let center_h = 0.5 * (horizontal_length - uv2_padding) / horizontal_length;
    let mut top_h = top_circumference / horizontal_length;
    let mut bottom_h = bottom_circumference / horizontal_length;
    let padding_h = uv2_padding / horizontal_length;

    let mut arrays = GeneratedArrays::default();
    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut point = 0i32;
    let mut thisrow = 0i32;
    let mut prevrow = 0i32;
    let side_normal_y = (bottom_radius - top_radius) / height;

    // Side.
    for j in 0..=(rings + 1) {
        let v = j as f32 / (rings + 1) as f32;
        let radius = top_radius + ((bottom_radius - top_radius) * v);
        let radius_h = top_h + ((bottom_h - top_h) * v);
        let y = (height * 0.5) - (height * v);

        for i in 0..=radial_segments {
            let u = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                ((u * tau).sin(), (u * tau).cos())
            };

            arrays
                .points
                .push(Vector3::new(x * radius, y, z * radius));
            arrays.normals.push(
                Vector3::new(x, side_normal_y, z).normalized(),
            );
            arrays.tangents.extend_from_slice(&[z, 0.0, -x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v * 0.5));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    center_h + (u - 0.5) * radius_h,
                    v * height_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }
        }

        prevrow = thisrow;
        thisrow = point;
    }

    // Bottom-section UV2 basis.
    top_h = top_radius / horizontal_length;
    let top_v = top_radius / vertical_length;
    bottom_h = bottom_radius / horizontal_length;
    let bottom_v = bottom_radius / vertical_length;

    // Top cap.
    if cap_top && top_radius > 0.0 {
        let y = height * 0.5;
        thisrow = point;
        arrays.points.push(Vector3::new(0.0, y, 0.0));
        arrays.normals.push(Vector3::new(0.0, 1.0, 0.0));
        arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
        arrays
            .uvs
            .push(crate::math::vector2::Vector2::new(0.25, 0.75));
        if add_uv2 {
            uv2s.push(crate::math::vector2::Vector2::new(
                top_h,
                height_v + padding_v + top_v.max(bottom_v),
            ));
        }
        point += 1;

        for i in 0..=radial_segments {
            let r = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                ((r * tau).sin(), (r * tau).cos())
            };
            let u = (x + 1.0) * 0.25;
            let v = 0.5 + ((z + 1.0) * 0.25);

            arrays
                .points
                .push(Vector3::new(x * top_radius, y, z * top_radius));
            arrays.normals.push(Vector3::new(0.0, 1.0, 0.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    top_h + (x * top_h),
                    height_v + padding_v + top_v.max(bottom_v) + (z * top_v),
                ));
            }
            point += 1;

            if i > 0 {
                arrays.indices.push(thisrow);
                arrays.indices.push(point - 1);
                arrays.indices.push(point - 2);
            }
        }
    }

    // Bottom cap.
    if cap_bottom && bottom_radius > 0.0 {
        let y = height * -0.5;
        thisrow = point;
        arrays.points.push(Vector3::new(0.0, y, 0.0));
        arrays.normals.push(Vector3::new(0.0, -1.0, 0.0));
        arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
        arrays
            .uvs
            .push(crate::math::vector2::Vector2::new(0.75, 0.75));
        if add_uv2 {
            uv2s.push(crate::math::vector2::Vector2::new(
                top_h + top_h + padding_h + bottom_h,
                height_v + padding_v + top_v.max(bottom_v),
            ));
        }
        point += 1;

        for i in 0..=radial_segments {
            let r = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                ((r * tau).sin(), (r * tau).cos())
            };
            let u = 0.5 + ((x + 1.0) * 0.25);
            let v = 1.0 - ((z + 1.0) * 0.25);

            arrays.points.push(Vector3::new(
                x * bottom_radius,
                y,
                z * bottom_radius,
            ));
            arrays.normals.push(Vector3::new(0.0, -1.0, 0.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    top_h + top_h + padding_h + bottom_h + (x * bottom_h),
                    height_v + padding_v + top_v.max(bottom_v) - (z * bottom_v),
                ));
            }
            point += 1;

            if i > 0 {
                arrays.indices.push(thisrow);
                arrays.indices.push(point - 2);
                arrays.indices.push(point - 1);
            }
        }
    }

    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

/// `CapsuleMesh::create_mesh_array`.
pub fn generate_capsule_arrays(
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let tau = std::f32::consts::TAU;
    let half_pi = 0.5 * std::f32::consts::PI;
    let onethird = 1.0f32 / 3.0;
    let twothirds = 2.0f32 / 3.0;

    let radial_width = 2.0 * radius * std::f32::consts::PI;
    let radial_h = radial_width / (radial_width + uv2_padding);
    let radial_length = radius * std::f32::consts::PI * 0.5;
    let vertical_length = radial_length * 2.0 + (height - 2.0 * radius) + uv2_padding;
    let radial_v = radial_length / vertical_length;
    let height_v = (height - 2.0 * radius) / vertical_length;

    let mut arrays = GeneratedArrays::default();
    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut point = 0i32;
    let mut thisrow = 0i32;
    let mut prevrow = 0i32;

    // Top hemisphere.
    for j in 0..=(rings + 1) {
        let v = j as f32 / (rings + 1) as f32;
        let (w, y) = if j == rings + 1 {
            (1.0, 0.0)
        } else {
            ((half_pi * v).sin(), (half_pi * v).cos())
        };

        for i in 0..=radial_segments {
            let u = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                (-(u * tau).sin(), (u * tau).cos())
            };

            let p = Vector3::new(x * w, y, -z * w);
            arrays.points.push(
                p * radius + Vector3::new(0.0, 0.5 * height - radius, 0.0),
            );
            arrays.normals.push(p);
            arrays.tangents.extend_from_slice(&[-z, 0.0, -x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, v * onethird));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u * radial_h,
                    v * radial_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }
        }

        prevrow = thisrow;
        thisrow = point;
    }

    // Cylinder band.
    thisrow = point;
    prevrow = 0;
    for j in 0..=(rings + 1) {
        let v = j as f32 / (rings + 1) as f32;
        let y = (0.5 * height - radius) - ((height - 2.0 * radius) * v);

        for i in 0..=radial_segments {
            let u = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                (-(u * tau).sin(), (u * tau).cos())
            };

            arrays.points.push(Vector3::new(x * radius, y, -z * radius));
            arrays.normals.push(Vector3::new(x, 0.0, -z));
            arrays.tangents.extend_from_slice(&[-z, 0.0, -x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(
                    u,
                    onethird + (v * onethird),
                ));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u * radial_h,
                    radial_v + (v * height_v),
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }
        }

        prevrow = thisrow;
        thisrow = point;
    }

    // Bottom hemisphere.
    thisrow = point;
    prevrow = 0;
    for j in 0..=(rings + 1) {
        let v = j as f32 / (rings + 1) as f32;
        let (w, y) = if j == rings + 1 {
            (0.0, -1.0)
        } else {
            ((half_pi * v).cos(), -(half_pi * v).sin())
        };

        for i in 0..=radial_segments {
            let u = i as f32 / radial_segments as f32;
            let (x, z) = if i == radial_segments {
                (0.0, 1.0)
            } else {
                (-(u * tau).sin(), (u * tau).cos())
            };

            let p = Vector3::new(x * w, y, -z * w);
            arrays.points.push(
                p * radius + Vector3::new(0.0, -0.5 * height + radius, 0.0),
            );
            arrays.normals.push(p);
            arrays.tangents.extend_from_slice(&[-z, 0.0, -x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(
                    u,
                    twothirds + v * onethird,
                ));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u * radial_h,
                    radial_v + height_v + v * radial_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }
        }

        prevrow = thisrow;
        thisrow = point;
    }

    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

/// `TorusMesh::create_mesh_array`.
#[allow(clippy::too_many_arguments)]
pub fn generate_torus_arrays(
    inner_radius: f32,
    outer_radius: f32,
    rings: i32,
    ring_segments: i32,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let tau = std::f32::consts::TAU;
    let mut arrays = GeneratedArrays::default();
    if inner_radius == outer_radius {
        return arrays;
    }

    let (min_radius, max_radius) = if inner_radius > outer_radius {
        (outer_radius, inner_radius)
    } else {
        (inner_radius, outer_radius)
    };
    let radius = (max_radius - min_radius) * 0.5;

    let horizontal_total = max_radius * tau + uv2_padding;
    let max_h = max_radius * tau / horizontal_total;
    let delta_h = (max_radius - min_radius) * tau / horizontal_total;
    let height_v = radius * tau / (radius * tau + uv2_padding);

    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut points = 0i32;

    for i in 0..=rings {
        let prevrow = (i - 1) * (ring_segments + 1);
        let thisrow = i * (ring_segments + 1);
        let inci = i as f32 / rings as f32;
        let angi = inci * tau;
        let normali = if i == rings {
            crate::math::vector2::Vector2::new(0.0, -1.0)
        } else {
            crate::math::vector2::Vector2::new(-angi.sin(), -angi.cos())
        };

        for j in 0..=ring_segments {
            let incj = j as f32 / ring_segments as f32;
            let angj = incj * tau;
            let normalj = if j == ring_segments {
                crate::math::vector2::Vector2::new(-1.0, 0.0)
            } else {
                crate::math::vector2::Vector2::new(-angj.cos(), angj.sin())
            };
            let normalk = normalj * radius
                + crate::math::vector2::Vector2::new(min_radius + radius, 0.0);

            let mut offset_h = 0.5 * (1.0 - normalj.x) * delta_h;
            let adj_h = max_h - offset_h;
            offset_h *= 0.5;

            arrays.points.push(Vector3::new(
                normali.x * normalk.x,
                normalk.y,
                normali.y * normalk.x,
            ));
            arrays.normals.push(Vector3::new(
                normali.x * normalj.x,
                normalj.y,
                normali.y * normalj.x,
            ));
            arrays
                .tangents
                .extend_from_slice(&[normali.y, 0.0, -normali.x, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(inci, incj));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    offset_h + inci * adj_h,
                    incj * height_v,
                ));
            }
            points += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(thisrow + j - 1);
                arrays.indices.push(prevrow + j);
                arrays.indices.push(prevrow + j - 1);
                arrays.indices.push(thisrow + j - 1);
                arrays.indices.push(thisrow + j);
                arrays.indices.push(prevrow + j);
            }
        }
    }

    let _ = points;
    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

/// `PrismMesh::create_mesh_array`.
pub fn generate_prism_arrays(
    size: Vector3,
    left_to_right: f32,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
    add_uv2: bool,
    uv2_padding: f32,
) -> GeneratedArrays {
    let onethird = 1.0f32 / 3.0;
    let twothirds = 2.0f32 / 3.0;

    let horizontal_total = size.x + size.z + 2.0 * uv2_padding;
    let width_h = size.x / horizontal_total;
    let depth_h = size.z / horizontal_total;
    let padding_h = uv2_padding / horizontal_total;

    let vertical_total = (size.y + size.y + size.z) + (3.0 * uv2_padding);
    let height_v = size.y / vertical_total;
    let depth_v = size.z / vertical_total;
    let padding_v = uv2_padding / vertical_total;

    let start_pos = size * -0.5;
    let mut arrays = GeneratedArrays::default();
    let mut uv2s: Vec<crate::math::vector2::Vector2> = Vec::new();
    let mut point = 0i32;
    let mut thisrow = 0i32;
    let mut prevrow = 0i32;

    // Front + back.
    let mut y = start_pos.y;
    for j in 0..=(subdivide_h + 1) {
        let scale = j as f32 / (subdivide_h + 1) as f32;
        let scaled_size_x = size.x * scale;
        let start_x = start_pos.x + (1.0 - scale) * size.x * left_to_right;
        let offset_front = (1.0 - scale) * onethird * left_to_right;
        let offset_back = (1.0 - scale) * onethird * (1.0 - left_to_right);

        let mut v = j as f32;
        let v2 = scale;
        v /= 2.0 * (subdivide_h + 1) as f32;

        let mut x = 0.0f32;
        for i in 0..=(subdivide_w + 1) {
            let mut u = i as f32;
            let u2 = i as f32 / (subdivide_w + 1) as f32;
            u /= 3.0 * (subdivide_w + 1) as f32;
            u *= scale;

            // Front.
            arrays
                .points
                .push(Vector3::new(start_x + x, -y, -start_pos.z));
            arrays.normals.push(Vector3::new(0.0, 0.0, 1.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(offset_front + u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * scale * width_h,
                    v2 * height_v,
                ));
            }
            point += 1;

            // Back.
            arrays.points.push(Vector3::new(
                start_x + scaled_size_x - x,
                -y,
                start_pos.z,
            ));
            arrays.normals.push(Vector3::new(0.0, 0.0, -1.0));
            arrays.tangents.extend_from_slice(&[-1.0, 0.0, 0.0, 1.0]);
            arrays.uvs.push(crate::math::vector2::Vector2::new(
                twothirds + offset_back + u,
                v,
            ));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * scale * width_h,
                    height_v + padding_v + v2 * height_v,
                ));
            }
            point += 1;

            if i > 0 && j == 1 {
                let i2 = i * 2;
                arrays.indices.push(prevrow + i2);
                arrays.indices.push(thisrow + i2);
                arrays.indices.push(thisrow + i2 - 2);
                arrays.indices.push(prevrow + i2 + 1);
                arrays.indices.push(thisrow + i2 + 1);
                arrays.indices.push(thisrow + i2 - 1);
            } else if i > 0 && j > 0 {
                let i2 = i * 2;
                push_quad(
                    &mut arrays.indices,
                    prevrow + i2 - 2,
                    prevrow + i2,
                    thisrow + i2 - 2,
                    prevrow + i2,
                    thisrow + i2,
                    thisrow + i2 - 2,
                );
                push_quad(
                    &mut arrays.indices,
                    prevrow + i2 - 1,
                    prevrow + i2 + 1,
                    thisrow + i2 - 1,
                    prevrow + i2 + 1,
                    thisrow + i2 + 1,
                    thisrow + i2 - 1,
                );
            }

            x += scale * size.x / (subdivide_w + 1) as f32;
        }

        y += size.y / (subdivide_h + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    // Left + right.
    let normal_left = Vector3::new(-size.y, size.x * left_to_right, 0.0).normalized();
    let normal_right = Vector3::new(size.y, size.x * (1.0 - left_to_right), 0.0).normalized();

    y = start_pos.y;
    thisrow = point;
    prevrow = 0;
    for j in 0..=(subdivide_h + 1) {
        let scale = j as f32 / (subdivide_h + 1) as f32;
        let left = start_pos.x + (size.x * (1.0 - scale) * left_to_right);
        let right = left + (size.x * scale);

        let mut v = j as f32;
        let v2 = scale;
        v /= 2.0 * (subdivide_h + 1) as f32;

        let mut z = start_pos.z;
        for i in 0..=(subdivide_d + 1) {
            let mut u = i as f32;
            let u2 = i as f32 / (subdivide_d + 1) as f32;
            u /= 3.0 * (subdivide_d + 1) as f32;

            // Right.
            arrays.points.push(Vector3::new(right, -y, -z));
            arrays.normals.push(normal_right);
            arrays.tangents.extend_from_slice(&[0.0, 0.0, -1.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(onethird + u, v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    width_h + padding_h + u2 * depth_h,
                    v2 * height_v,
                ));
            }
            point += 1;

            // Left.
            arrays.points.push(Vector3::new(left, -y, z));
            arrays.normals.push(normal_left);
            arrays.tangents.extend_from_slice(&[0.0, 0.0, 1.0, 1.0]);
            arrays
                .uvs
                .push(crate::math::vector2::Vector2::new(u, 0.5 + v));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    width_h + padding_h + u2 * depth_h,
                    height_v + padding_v + v2 * height_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                let i2 = i * 2;
                push_quad(
                    &mut arrays.indices,
                    prevrow + i2 - 2,
                    prevrow + i2,
                    thisrow + i2 - 2,
                    prevrow + i2,
                    thisrow + i2,
                    thisrow + i2 - 2,
                );
                push_quad(
                    &mut arrays.indices,
                    prevrow + i2 - 1,
                    prevrow + i2 + 1,
                    thisrow + i2 - 1,
                    prevrow + i2 + 1,
                    thisrow + i2 + 1,
                    thisrow + i2 - 1,
                );
            }

            z += size.z / (subdivide_d + 1) as f32;
        }

        y += size.y / (subdivide_h + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    // Bottom.
    let mut z = start_pos.z;
    thisrow = point;
    prevrow = 0;
    for j in 0..=(subdivide_d + 1) {
        let mut v = j as f32;
        let v2 = v / (subdivide_d + 1) as f32;
        v /= 2.0 * (subdivide_d + 1) as f32;

        let mut x = start_pos.x;
        for i in 0..=(subdivide_w + 1) {
            let mut u = i as f32;
            let u2 = u / (subdivide_w + 1) as f32;
            u /= 3.0 * (subdivide_w + 1) as f32;

            arrays.points.push(Vector3::new(x, start_pos.y, -z));
            arrays.normals.push(Vector3::new(0.0, -1.0, 0.0));
            arrays.tangents.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
            arrays.uvs.push(crate::math::vector2::Vector2::new(
                twothirds + u,
                0.5 + v,
            ));
            if add_uv2 {
                uv2s.push(crate::math::vector2::Vector2::new(
                    u2 * width_h,
                    2.0 * (height_v + padding_v) + v2 * depth_v,
                ));
            }
            point += 1;

            if i > 0 && j > 0 {
                arrays.indices.push(prevrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i - 1);
                arrays.indices.push(prevrow + i);
                arrays.indices.push(thisrow + i);
                arrays.indices.push(thisrow + i - 1);
            }

            x += size.x / (subdivide_w + 1) as f32;
        }

        z += size.z / (subdivide_d + 1) as f32;
        prevrow = thisrow;
        thisrow = point;
    }

    if add_uv2 {
        arrays.uv2s = uv2s;
    }
    arrays
}

#[derive(Clone, Debug)]
pub struct PlaneMeshData {
    pub params: PrimitiveMeshParams,
    size: crate::math::vector2::Vector2,
    subdivide_w: i32,
    subdivide_d: i32,
    center_offset: Vector3,
    orientation: PlaneOrientation,
}

impl Default for PlaneMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaneMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            size: crate::math::vector2::Vector2::new(2.0, 2.0),
            subdivide_w: 0,
            subdivide_d: 0,
            center_offset: Vector3::ZERO,
            orientation: PlaneOrientation::FaceY,
        }
    }

    pub fn set_size(&mut self, size: crate::math::vector2::Vector2) {
        self.size = size;
    }

    pub fn get_size(&self) -> crate::math::vector2::Vector2 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }

    pub fn set_center_offset(&mut self, offset: Vector3) {
        self.center_offset = offset;
    }
    pub fn set_orientation(&mut self, orientation: PlaneOrientation) {
        self.orientation = orientation;
    }

    pub fn get_orientation(&self) -> PlaneOrientation {
        self.orientation
    }

    /// _create_mesh_array with this mesh's parameters.
    pub fn generate_arrays(&self) -> GeneratedArrays {
        generate_plane_arrays(
            self.size,
            self.subdivide_w,
            self.subdivide_d,
            self.center_offset,
            self.orientation,
        )
    }



    pub fn get_center_offset(&self) -> Vector3 {
        self.center_offset
    }
}

/// `QuadMesh` is a `PlaneMesh` facing +Z with zero center offset.
#[derive(Clone, Debug)]
pub struct QuadMeshData {
    pub plane: PlaneMeshData,
}

impl Default for QuadMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadMeshData {
    pub fn new() -> Self {
        let mut plane = PlaneMeshData::new();
        plane.set_center_offset(Vector3::ZERO);
        Self { plane }
    }
}

#[derive(Clone, Debug)]
pub struct PrismMeshData {
    pub params: PrimitiveMeshParams,
    left_to_right: f32,
    size: Vector3,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
}

impl Default for PrismMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl PrismMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            left_to_right: 0.5,
            size: Vector3::new(1.0, 1.0, 1.0),
            subdivide_w: 0,
            subdivide_h: 0,
            subdivide_d: 0,
        }
    }

    pub fn set_left_to_right(&mut self, value: f32) {
        self.left_to_right = value;
    }

    pub fn get_left_to_right(&self) -> f32 {
        self.left_to_right
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_height(&mut self, subdivide: i32) {
        self.subdivide_h = subdivide;
    }

    pub fn get_subdivide_height(&self) -> i32 {
        self.subdivide_h
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }
}

impl PrismMeshData {
    /// `_create_mesh_array` with this mesh's parameters.
    pub fn generate_arrays(&self) -> GeneratedArrays {
        generate_prism_arrays(
            self.size,
            self.left_to_right,
            self.subdivide_w,
            self.subdivide_h,
            self.subdivide_d,
            self.params.add_uv2,
            self.params.uv2_padding * self.params.texel_size,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SphereMeshData {
    pub params: PrimitiveMeshParams,
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    is_hemisphere: bool,
}

impl Default for SphereMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl SphereMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            radius: 0.5,
            height: 1.0,
            radial_segments: 64,
            rings: 32,
            is_hemisphere: false,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        // CylinderMesh clamps to at least four segments.
        self.radial_segments = segments.max(4);
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 1)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 1 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    pub fn set_is_hemisphere(&mut self, hemisphere: bool) {
        self.is_hemisphere = hemisphere;
    }

    pub fn get_is_hemisphere(&self) -> bool {
        self.is_hemisphere
    }
}

#[derive(Clone, Debug)]
pub struct TorusMeshData {
    pub params: PrimitiveMeshParams,
    inner_radius: f32,
    outer_radius: f32,
    rings: i32,
    ring_segments: i32,
}

impl Default for TorusMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl TorusMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            inner_radius: 0.5,
            outer_radius: 1.0,
            rings: 64,
            ring_segments: 32,
        }
    }

    /// Equal radii are refused ("Inner radius and outer radius cannot be the
    /// same.").
    pub fn set_inner_radius(&mut self, radius: f32) -> bool {
        if radius == self.outer_radius {
            return false;
        }
        self.inner_radius = radius;
        true
    }

    pub fn get_inner_radius(&self) -> f32 {
        self.inner_radius
    }

    pub fn set_outer_radius(&mut self, radius: f32) -> bool {
        if radius == self.inner_radius {
            return false;
        }
        self.outer_radius = radius;
        true
    }

    pub fn get_outer_radius(&self) -> f32 {
        self.outer_radius
    }

    /// `ERR_FAIL_COND(p_rings < 3)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 3 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    /// `ERR_FAIL_COND(p_ring_segments < 3)`.
    pub fn set_ring_segments(&mut self, segments: i32) -> bool {
        if segments < 3 {
            return false;
        }
        self.ring_segments = segments;
        true
    }

    pub fn get_ring_segments(&self) -> i32 {
        self.ring_segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn base_parameters() {
        let p = PrimitiveMeshParams::default();
        assert!(!p.flip_faces);
        assert!(!p.add_uv2);
        assert_eq!(p.uv2_padding, 2.0);
        assert_eq!(p.texel_size, 0.2);
    }

    #[test]
    fn capsule_defaults_and_rings() {
        let mut c = CapsuleMeshData::new();
        assert_eq!(c.get_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert_eq!(c.get_radial_segments(), 64);
        assert_eq!(c.get_rings(), 8);

        assert!(!c.set_rings(-1));
        assert!(c.set_rings(0));
        assert_eq!(c.get_rings(), 0);
        c.set_radius(1.5);
        c.set_height(4.0);
        c.set_radial_segments(8);
        assert_eq!(c.get_radius(), 1.5);
        assert_eq!(c.get_height(), 4.0);
        assert_eq!(c.get_radial_segments(), 8);
    }

    #[test]
    fn box_and_plane_defaults() {
        let mut b = BoxMeshData::new();
        assert_eq!(b.get_size(), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(b.get_subdivide_width(), 0);
        assert_eq!(b.get_subdivide_height(), 0);
        assert_eq!(b.get_subdivide_depth(), 0);
        b.set_size(Vector3::new(2.0, 3.0, 4.0));
        b.set_subdivide_width(2);
        b.set_subdivide_height(3);
        b.set_subdivide_depth(4);
        assert_eq!(b.get_size(), Vector3::new(2.0, 3.0, 4.0));
        assert_eq!(b.get_subdivide_width(), 2);

        let mut p = PlaneMeshData::new();
        assert_eq!(p.get_size(), Vector2::new(2.0, 2.0));
        assert_eq!(p.get_center_offset(), Vector3::ZERO);
        p.set_size(Vector2::new(4.0, 6.0));
        p.set_subdivide_width(1);
        p.set_subdivide_depth(2);
        p.set_center_offset(Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(p.get_size(), Vector2::new(4.0, 6.0));
        assert_eq!(p.get_subdivide_depth(), 2);
        assert_eq!(p.get_center_offset(), Vector3::new(0.0, 1.0, 0.0));

        let q = QuadMeshData::new();
        assert_eq!(q.plane.get_center_offset(), Vector3::ZERO);
        assert_eq!(q.plane.get_size(), Vector2::new(2.0, 2.0));
    }

    #[test]
    fn cylinder_defaults_and_rings() {
        let mut c = CylinderMeshData::new();
        assert_eq!(c.get_top_radius(), 0.5);
        assert_eq!(c.get_bottom_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert_eq!(c.get_radial_segments(), 64);
        assert_eq!(c.get_rings(), 4);
        assert!(c.is_cap_top());
        assert!(c.is_cap_bottom());

        assert!(!c.set_rings(-5));
        assert!(c.set_rings(0));
        c.set_top_radius(1.0);
        c.set_bottom_radius(2.0);
        c.set_cap_top(false);
        c.set_cap_bottom(false);
        assert_eq!(c.get_top_radius(), 1.0);
        assert_eq!(c.get_bottom_radius(), 2.0);
        assert!(!c.is_cap_top());
        assert!(!c.is_cap_bottom());
    }

    #[test]
    fn prism_arrays_counts_and_extents() {
        // No subdivisions: 2×2 grid for front/back (8), 2×2 for left/right (8)
        // and 2×2 for the bottom (4).
        let a = generate_prism_arrays(Vector3::new(1.0, 1.0, 1.0), 0.5, 0, 0, 0, false, 0.0);
        assert_eq!(a.points.len(), 20);
        assert_eq!(a.indices.len(), 6 + 12 + 6);
        assert!(a.uv2s.is_empty());
        assert!(a.indices.iter().all(|&i| (0..20).contains(&i)));
        // Everything sits inside the requested box.
        assert!(a.points.iter().all(|p| {
            p.x.abs() <= 0.5 + 1e-6 && p.y.abs() <= 0.5 + 1e-6 && p.z.abs() <= 0.5 + 1e-6
        }));
        // The front face looks at +Z and the back at -Z.
        assert_eq!(a.normals[0], Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(a.normals[1], Vector3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn prism_arrays_subdivisions_and_uv2() {
        let a = generate_prism_arrays(Vector3::new(2.0, 2.0, 2.0), 0.25, 1, 1, 1, true, 0.1);
        assert_eq!(a.uv2s.len(), a.points.len());
        assert!(a.uv2s.iter().all(|uv| uv.x >= 0.0 && uv.y >= 0.0));
        assert!(a.indices.iter().all(|&i| (0..a.points.len() as i32).contains(&i)));
        // Subdividing adds more vertices than the default prism.
        let b = generate_prism_arrays(Vector3::new(2.0, 2.0, 2.0), 0.25, 1, 1, 1, false, 0.0);
        assert!(b.points.len() > 20);

        // The wrapper uses the mesh's parameters.
        let mut p = PrismMeshData::new();
        p.set_subdivide_width(1);
        p.set_subdivide_height(1);
        p.set_subdivide_depth(1);
        p.params.add_uv2 = true;
        let arrays = p.generate_arrays();
        assert_eq!(arrays.points.len(), b.points.len());
        assert_eq!(arrays.uv2s.len(), arrays.points.len());
    }

    #[test]
    fn prism_left_to_right_shifts_the_apex() {
        // left_to_right = 1 puts the apex on the left side; 0 on the right.
        let left = generate_prism_arrays(Vector3::new(1.0, 1.0, 1.0), 1.0, 0, 0, 0, false, 0.0);
        // The apex row is the third pair of vertices (row j = 1, i = 0).
        let left_apex_front = left.points[4].x;
        assert!((left_apex_front - (-0.5)).abs() < 1e-6);

        let right = generate_prism_arrays(Vector3::new(1.0, 1.0, 1.0), 0.0, 0, 0, 0, false, 0.0);
        // With left_to_right = 0 the front apex stays at the base's left edge.
        assert!((right.points[4].x - (-0.5)).abs() < 1e-6);
        // The left side normal flips with the slope.
        let left_side = generate_prism_arrays(Vector3::new(1.0, 2.0, 1.0), 1.0, 0, 0, 0, false, 0.0);
        let right_side = generate_prism_arrays(Vector3::new(1.0, 2.0, 1.0), 0.0, 0, 0, 0, false, 0.0);
        assert_ne!(left_side.normals[8], right_side.normals[8]);
    }

    #[test]
    fn prism_defaults() {
        let mut p = PrismMeshData::new();
        assert_eq!(p.get_left_to_right(), 0.5);
        assert_eq!(p.get_size(), Vector3::new(1.0, 1.0, 1.0));
        p.set_left_to_right(0.25);
        p.set_size(Vector3::new(2.0, 2.0, 2.0));
        p.set_subdivide_width(1);
        p.set_subdivide_height(2);
        p.set_subdivide_depth(3);
        assert_eq!(p.get_left_to_right(), 0.25);
        assert_eq!(p.get_subdivide_height(), 2);
        assert_eq!(p.get_subdivide_depth(), 3);
    }

    #[test]
    fn sphere_defaults_and_rings() {
        let mut s = SphereMeshData::new();
        assert_eq!(s.get_radius(), 0.5);
        assert_eq!(s.get_height(), 1.0);
        assert_eq!(s.get_radial_segments(), 64);
        assert_eq!(s.get_rings(), 32);
        assert!(!s.get_is_hemisphere());

        assert!(!s.set_rings(0), "at least one ring");
        assert!(s.set_rings(1));
        assert_eq!(s.get_rings(), 1);
        s.set_is_hemisphere(true);
        assert!(s.get_is_hemisphere());
    }

    #[test]
    fn torus_defaults_and_guards() {
        let mut t = TorusMeshData::new();
        assert_eq!(t.get_inner_radius(), 0.5);
        assert_eq!(t.get_outer_radius(), 1.0);
        assert_eq!(t.get_rings(), 64);
        assert_eq!(t.get_ring_segments(), 32);

        // Equal radii are refused in both directions.
        assert!(!t.set_inner_radius(1.0));
        assert!(!t.set_outer_radius(0.5));
        assert!(t.set_inner_radius(0.25));
        assert_eq!(t.get_inner_radius(), 0.25);
        assert!(t.set_outer_radius(2.0));
        assert_eq!(t.get_outer_radius(), 2.0);

        assert!(!t.set_rings(2));
        assert!(t.set_rings(3));
        assert!(!t.set_ring_segments(2));
        assert!(t.set_ring_segments(3));
        assert_eq!(t.get_rings(), 3);
        assert_eq!(t.get_ring_segments(), 3);
    }

    #[test]
    fn plane_arrays_default_grid() {
        // 2×2 plane, no subdivision: (0+2)*(0+2) = 4 vertices, 1 quad = 6 indices.
        let arrays = generate_plane_arrays(
            Vector2::new(2.0, 2.0),
            0,
            0,
            Vector3::ZERO,
            PlaneOrientation::FaceY,
        );
        assert_eq!(arrays.points.len(), 4);
        assert_eq!(arrays.normals.len(), 4);
        assert_eq!(arrays.tangents.len(), 16);
        assert_eq!(arrays.uvs.len(), 4);
        assert_eq!(arrays.indices.len(), 6);

        // Facing +Y, corners at ±1 with UVs flipped (1-u, 1-v).
        // FACE_Y maps (x, z) to (-x, 0, -z), so the first corner is (1, 0, 1).
        assert_eq!(arrays.points[0], Vector3::new(1.0, 0.0, 1.0));
        assert_eq!(arrays.normals[0], Vector3::UP);
        assert_eq!(arrays.uvs[0], Vector2::new(1.0, 1.0));
        assert_eq!(arrays.uvs[3], Vector2::new(0.0, 0.0));
        assert!(arrays.indices.iter().all(|&i| (0..4).contains(&i)));
    }

    #[test]
    fn plane_arrays_subdivisions_and_orientation() {
        let arrays = generate_plane_arrays(
            Vector2::new(2.0, 2.0),
            1,
            1,
            Vector3::new(0.0, 1.0, 0.0),
            PlaneOrientation::FaceY,
        );
        // (1+2)^2 = 9 vertices, 4 quads * 6 = 24 indices.
        assert_eq!(arrays.points.len(), 9);
        assert_eq!(arrays.indices.len(), 24);
        // The centre offset is applied to every point.
        assert!(arrays.points.iter().all(|p| (p.y - 1.0).abs() < 1e-6));

        // Face X uses the YZ plane with a -Z tangent.
        let x = generate_plane_arrays(
            Vector2::new(2.0, 2.0),
            0,
            0,
            Vector3::ZERO,
            PlaneOrientation::FaceX,
        );
        assert_eq!(x.normals[0], Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(&x.tangents[0..4], &[0.0, 0.0, -1.0, 1.0]);

        let z = generate_plane_arrays(
            Vector2::new(2.0, 2.0),
            0,
            0,
            Vector3::ZERO,
            PlaneOrientation::FaceZ,
        );
        assert_eq!(z.normals[0], Vector3::new(0.0, 0.0, 1.0));

        // The mesh wrapper uses its own parameters.
        let mut p = PlaneMeshData::new();
        p.set_subdivide_width(1);
        p.set_subdivide_depth(1);
        p.set_orientation(PlaneOrientation::FaceZ);
        let arrays = p.generate_arrays();
        assert_eq!(arrays.points.len(), 9);
        assert_eq!(p.get_orientation(), PlaneOrientation::FaceZ);
        assert_eq!(PlaneOrientation::FaceX as i32, 0);
        assert_eq!(PlaneOrientation::FaceZ as i32, 2);
    }

    #[test]
    fn box_arrays_default_cube() {
        let arrays = generate_box_arrays(Vector3::new(1.0, 1.0, 1.0), 0, 0, 0, false, 0.0);
        // Six faces, each a 2x2 grid.
        assert_eq!(arrays.points.len(), 24);
        assert_eq!(arrays.normals.len(), 24);
        assert_eq!(arrays.tangents.len(), 24 * 4);
        assert_eq!(arrays.uvs.len(), 24);
        assert_eq!(arrays.indices.len(), 36, "six quads");
        assert!(arrays.uv2s.is_empty());
        assert!(arrays.indices.iter().all(|&i| (0..24).contains(&i)));

        // The first six vertices cover front and back on the first row.
        assert_eq!(arrays.normals[0], Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(arrays.normals[1], Vector3::new(0.0, 0.0, -1.0));
        assert_eq!(arrays.normals[2], Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(arrays.normals[3], Vector3::new(0.0, 0.0, -1.0));
        // Corners sit at ±0.5 and the front half uses u in the first third.
        assert_eq!(arrays.points[0], Vector3::new(-0.5, 0.5, 0.5));
        assert!((arrays.uvs[0].x - 0.0).abs() < 1e-6);
    }

    #[test]
    fn box_arrays_subdivisions_and_uv2() {
        let arrays = generate_box_arrays(Vector3::new(2.0, 2.0, 2.0), 1, 1, 1, true, 0.1);
        // Each face is 3x3 = 9 vertices, six faces.
        assert_eq!(arrays.points.len(), 54);
        // Each face contributes (1+1)^2 = 4 quads = 24 indices.
        assert_eq!(arrays.indices.len(), 144);
        assert_eq!(arrays.uv2s.len(), 54, "UV2 is generated per vertex");
        assert!(arrays.uv2s.iter().all(|uv| uv.x >= 0.0 && uv.y >= 0.0));
        assert!(arrays.points.iter().all(|p| {
            p.x.abs() <= 1.0 + 1e-6 && p.y.abs() <= 1.0 + 1e-6 && p.z.abs() <= 1.0 + 1e-6
        }));
    }

    #[test]
    fn sphere_arrays_counts() {
        let a = generate_sphere_arrays(0.5, 1.0, 8, 4, false, false, 0.0);
        assert_eq!(a.points.len(), (4 + 2) * (8 + 1));
        assert_eq!(a.indices.len(), (4 + 1) * 8 * 6);
        assert_eq!(a.normals.len(), a.points.len());
        assert_eq!(a.tangents.len(), a.points.len() * 4);
        assert_eq!(a.uvs.len(), a.points.len());
        assert!(a.uv2s.is_empty());
        assert!(a.indices.iter().all(|&i| (0..a.points.len() as i32).contains(&i)));
        // Every normal is unit length (the degenerate rows keep the polar direction).
        assert!(a.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-4 || n.length() < 1e-6));
    }

    #[test]
    fn sphere_arrays_hemisphere_and_uv2() {
        let a = generate_sphere_arrays(1.0, 2.0, 6, 3, true, true, 0.1);
        assert_eq!(a.uv2s.len(), a.points.len());
        // The hemisphere clamps the lower half to y=0 with a -Y normal.
        let flat = a
            .points
            .iter()
            .zip(a.normals.iter())
            .filter(|(p, n)| p.y.abs() < 1e-6 && **n == Vector3::new(0.0, -1.0, 0.0))
            .count();
        assert!(flat > 0, "hemisphere bottom is flattened");
        assert!(a.points.iter().all(|p| p.y >= -1e-6));
    }

    #[test]
    fn cylinder_arrays_counts_and_caps() {
        // sides: (rings+2)*(radial+1); caps add 1 + radial each.
        let a = generate_cylinder_arrays(0.5, 0.5, 2.0, 8, 4, true, true, false, 0.0);
        assert_eq!(a.points.len(), (4 + 2) * (8 + 1) + 4 + 2 * 8);
        assert_eq!(a.indices.len(), (4 + 1) * 8 * 6 + 6 * 8);
        assert!(a.uv2s.is_empty());

        // Without caps only the side remains.
        let b = generate_cylinder_arrays(0.5, 0.5, 2.0, 8, 4, false, false, false, 0.0);
        assert_eq!(b.points.len(), (4 + 2) * (8 + 1));
        assert_eq!(b.indices.len(), (4 + 1) * 8 * 6);

        // A zero-radius cap is skipped even when enabled.
        let c = generate_cylinder_arrays(0.0, 0.5, 2.0, 8, 0, true, false, false, 0.0);
        assert_eq!(c.points.len(), (0 + 2) * (8 + 1));

        // The side's normals account for the slope between the two radii.
        let tapered = generate_cylinder_arrays(1.0, 0.25, 1.0, 8, 0, false, false, false, 0.0);
        let normal = tapered.normals[0];
        assert!((normal.length() - 1.0).abs() < 1e-5);
        assert!(normal.y < 0.0, "a wider top tilts the side normal down");
        // The side y goes from +height/2 to -height/2.
        assert!((tapered.points[0].y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn cylinder_arrays_uv2() {
        let a = generate_cylinder_arrays(0.5, 0.5, 2.0, 8, 2, true, true, true, 0.1);
        assert_eq!(a.uv2s.len(), a.points.len());
        assert!(a.uv2s.iter().all(|uv| uv.x >= 0.0 && uv.y >= 0.0));
    }

    #[test]
    fn radial_segments_clamp_to_four() {
        let mut cylinder = CylinderMeshData::new();
        cylinder.set_radial_segments(1);
        assert_eq!(cylinder.get_radial_segments(), 4);
        cylinder.set_radial_segments(12);
        assert_eq!(cylinder.get_radial_segments(), 12);

        let mut capsule = CapsuleMeshData::new();
        capsule.set_radial_segments(0);
        assert_eq!(capsule.get_radial_segments(), 4);

        let mut sphere = SphereMeshData::new();
        sphere.set_radial_segments(2);
        assert_eq!(sphere.get_radial_segments(), 4);
    }

    #[test]
    fn capsule_arrays_counts_and_extents() {
        // Three sections of (rings+2)*(radial+1) vertices.
        let a = generate_capsule_arrays(0.5, 2.0, 8, 4, false, 0.0);
        assert_eq!(a.points.len(), (4 + 2) * (8 + 1) * 3);
        assert_eq!(a.indices.len(), 3 * (4 + 1) * 8 * 6);
        assert!(a.uv2s.is_empty());

        // The top row is the upper pole at +height/2, the last row the lower pole.
        assert!((a.points[0].y - 1.0).abs() < 1e-6);
        let last = *a.points.last().unwrap();
        assert!((last.y + 1.0).abs() < 1e-6);
        // Normals are unit vectors on the sphere.
        assert!(a.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-5));

        // The band spans the straight section between the hemispheres.
        let band_start = (4 + 2) * (8 + 1);
        assert!((a.points[band_start].y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn capsule_arrays_uv2() {
        let a = generate_capsule_arrays(0.5, 2.0, 8, 4, true, 0.1);
        assert_eq!(a.uv2s.len(), a.points.len());
        assert!(a.uv2s.iter().all(|uv| uv.x >= 0.0 && uv.y >= 0.0));
    }

    #[test]
    fn torus_arrays_counts_and_shape() {
        let a = generate_torus_arrays(0.5, 1.0, 3, 3, false, 0.0);
        // (rings+1) * (ring_segments+1) vertices and rings*segments*6 indices.
        assert_eq!(a.points.len(), (3 + 1) * (3 + 1));
        assert_eq!(a.indices.len(), 3 * 3 * 6);
        assert!(a.uv2s.is_empty());
        assert!(a.indices.iter().all(|&i| (0..a.points.len() as i32).contains(&i)));

        // The tube spans inner+radius .. outer-radius+...: all points lie
        // between the two radii on the XZ ring, never at the origin.
        let radius = (1.0 - 0.5) * 0.5;
        let min_center = 0.5 + radius; // 0.75
        let max_center = 1.0 - radius; // 0.75 (same when radii are 0.5/1.0)
        let _ = (min_center, max_center);
        assert!(a.points.iter().all(|p| {
            let d = (p.x * p.x + p.z * p.z).sqrt();
            d >= 0.5 && d <= 1.0
        }));
        // Normals are unit length.
        assert!(a.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-5));
    }

    #[test]
    fn torus_arrays_swaps_and_rejects_equal_radii() {
        // Swapped radii produce the same mesh.
        let a = generate_torus_arrays(1.0, 0.5, 4, 4, false, 0.0);
        assert!(!a.points.is_empty());
        // Equal radii are refused, like the engine's ERR_FAIL_COND.
        let b = generate_torus_arrays(0.5, 0.5, 4, 4, false, 0.0);
        assert!(b.points.is_empty());
        assert!(b.indices.is_empty());

        // UV2 is generated per vertex.
        let c = generate_torus_arrays(0.5, 1.0, 4, 4, true, 0.1);
        assert_eq!(c.uv2s.len(), c.points.len());
        assert!(c.uv2s.iter().all(|uv| uv.x >= 0.0 && uv.y >= 0.0));
    }
}
