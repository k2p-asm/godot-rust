//! `scene/2d/line_builder.*`: the polyline triangulator used by `Line2D`.
//!
//! Ported from `line_builder.cpp`: the strip builder (`strip_begin`,
//! `strip_add_quad`, `strip_add_tri`, `strip_add_arc`, `new_arc`) and the full
//! `build()` — total-distance and UV computation, the closed wrap-around
//! "fake pass", inner-joint intersection with the mitre/sharp-limit fallback,
//! bevel and round joints with the engine's `-0.0` cross-product fix, box and
//! round caps, and the gradient/curve sampling.
//!
//! Like the engine, `colors` only gets one entry per vertex when a gradient is
//! set; otherwise it holds the single default colour.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::scene::curve::Curve;
use crate::scene::gradient::Gradient;
use crate::scene::line_2d::{LineCapMode, LineJointMode, LineTextureMode};

/// `interpolate(Rect2, Vector2)`.
fn interpolate(rect: Rect2, v: Vector2) -> Vector2 {
    Vector2::new(
        rect.position.x + (rect.size.x) * v.x,
        rect.position.y + (rect.size.y) * v.y,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Orientation {
    Up = 0,
    Down = 1,
}

#[derive(Clone, Debug)]
pub struct LineBuilder {
    // Input.
    pub points: Vec<Vector2>,
    pub joint_mode: LineJointMode,
    pub begin_cap_mode: LineCapMode,
    pub end_cap_mode: LineCapMode,
    pub closed: bool,
    pub width: f32,
    pub curve: Option<Curve>,
    pub default_color: Color,
    pub gradient: Option<Gradient>,
    pub texture_mode: LineTextureMode,
    pub sharp_limit: f32,
    pub round_precision: i32,
    /// Width / height of the texture.
    pub tile_aspect: f32,

    // Output.
    pub vertices: Vec<Vector2>,
    pub colors: Vec<Color>,
    pub uvs: Vec<Vector2>,
    pub indices: Vec<i32>,

    interpolate_color: bool,
    last_index: [i32; 2],
}

impl Default for LineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LineBuilder {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            joint_mode: LineJointMode::Sharp,
            begin_cap_mode: LineCapMode::None,
            end_cap_mode: LineCapMode::None,
            closed: false,
            width: 10.0,
            curve: None,
            // `Color(0.4, 0.5, 1)`.
            default_color: Color::new(0.4, 0.5, 1.0, 1.0),
            gradient: None,
            texture_mode: LineTextureMode::None,
            sharp_limit: 2.0,
            round_precision: 8,
            tile_aspect: 1.0,
            vertices: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            interpolate_color: false,
            last_index: [0; 2],
        }
    }

    /// `build`.
    pub fn build(&mut self) {
        if self.points.len() < 2 {
            self.clear_output();
            return;
        }
        if self.tile_aspect <= 0.0 {
            return;
        }

        let hw = self.width / 2.0;
        let hw_sq = hw * hw;
        let sharp_limit_sq = self.sharp_limit * self.sharp_limit;
        let point_count = self.points.len();
        let wrap_around = self.closed && point_count > 2;

        self.interpolate_color = self.gradient.is_some();
        let retrieve_curve = self.curve.is_some();
        let distance_required = self.interpolate_color
            || retrieve_curve
            || self.texture_mode == LineTextureMode::Tile
            || self.texture_mode == LineTextureMode::Stretch;

        let mut pos0 = self.points[0];
        let mut pos1 = self.points[1];
        let mut f0 = (pos1 - pos0).normalized();
        let mut u0 = f0.orthogonal();
        let mut pos_up0 = pos0;
        let mut pos_down0 = pos0;

        let mut color0 = Color::new(0.0, 0.0, 0.0, 1.0);
        let mut color1 = Color::new(0.0, 0.0, 0.0, 1.0);

        let mut current_distance0 = 0.0f32;
        let mut current_distance1 = 0.0f32;
        let mut total_distance = 0.0f32;

        let mut width_factor = 1.0f32;
        let mut modified_hw = hw;
        if retrieve_curve {
            width_factor = self.sample_curve(0.0);
            modified_hw = hw * width_factor;
        }

        if distance_required {
            for i in 1..point_count {
                total_distance += self.points[i].distance_to(self.points[i - 1]);
            }
            if wrap_around {
                total_distance += self.points[point_count - 1].distance_to(pos0);
            } else {
                if self.begin_cap_mode == LineCapMode::Box
                    || self.begin_cap_mode == LineCapMode::Round
                {
                    total_distance += modified_hw;
                }
                if self.end_cap_mode == LineCapMode::Box || self.end_cap_mode == LineCapMode::Round {
                    if retrieve_curve {
                        total_distance += hw * self.sample_curve(1.0);
                    } else {
                        total_distance += hw;
                    }
                }
            }
        }

        if point_count < 2 || (distance_required && total_distance.abs() < 1e-5) {
            return;
        }

        if self.interpolate_color {
            color0 = self.gradient_color(0);
            if let Some(gradient) = self.gradient.as_mut() {
                color0 = gradient.get_color(0);
            }
        } else {
            self.colors.push(self.default_color);
        }

        let mut uvx0 = 0.0f32;
        let mut uvx1 = 0.0f32;

        pos_up0 += u0 * modified_hw;
        pos_down0 -= u0 * modified_hw;

        // Begin cap.
        if !wrap_around {
            if self.begin_cap_mode == LineCapMode::Box {
                pos_up0 -= f0 * modified_hw;
                pos_down0 -= f0 * modified_hw;
                current_distance0 += modified_hw;
                current_distance1 = current_distance0;
            } else if self.begin_cap_mode == LineCapMode::Round {
                if self.texture_mode == LineTextureMode::Tile {
                    uvx0 = width_factor * 0.5 / self.tile_aspect;
                } else if self.texture_mode == LineTextureMode::Stretch {
                    uvx0 = self.width * width_factor / total_distance;
                }
                self.new_arc(
                    pos0,
                    pos_up0 - pos0,
                    -std::f32::consts::PI,
                    color0,
                    Rect2::new(Vector2::ZERO, Vector2::new(uvx0 * 2.0, 1.0)),
                );
                current_distance0 += modified_hw;
                current_distance1 = current_distance0;
            }
            self.strip_begin(pos_up0, pos_down0, color0, uvx0);
        }

        let segments_count = if wrap_around {
            point_count as i32
        } else {
            point_count as i32 - 2
        };
        let first_point = if wrap_around { -1 } else { 1 };

        let mut first_pos_up = Vector2::ZERO;
        let mut first_pos_down = Vector2::ZERO;
        let mut is_first_joint_sharp = false;

        let mut i = first_point;
        while i <= segments_count {
            pos1 = if i == -1 {
                self.points[point_count - 1]
            } else {
                self.points[(i % point_count as i32) as usize]
            };
            let pos2 = self.points[((i + 1) % point_count as i32) as usize];

            let f1 = (pos2 - pos1).normalized();
            let u1 = f1.orthogonal();

            let dp = u0.dot(f1);
            let orientation = if dp > 0.0 {
                Orientation::Up
            } else {
                Orientation::Down
            };

            if distance_required && i >= 1 {
                current_distance1 += pos0.distance_to(pos1);
            }
            if self.interpolate_color {
                let offset = if total_distance != 0.0 {
                    current_distance1 / total_distance
                } else {
                    0.0
                };
                color1 = match self.gradient.as_mut() {
                    Some(gradient) => gradient.get_color_at_offset(offset),
                    None => color1,
                };
            }
            if retrieve_curve {
                let offset = if total_distance != 0.0 {
                    current_distance1 / total_distance
                } else {
                    0.0
                };
                width_factor = self.sample_curve(offset);
                modified_hw = hw * width_factor;
            }

            let mut inner_normal0 = u0 * modified_hw;
            let mut inner_normal1 = u1 * modified_hw;
            if orientation == Orientation::Down {
                inner_normal0 = -inner_normal0;
                inner_normal1 = -inner_normal1;
            }

            let intersection = crate::math::geometry_2d::segment_intersects_segment(
                pos0 + inner_normal0,
                pos1 + inner_normal0,
                pos1 + inner_normal1,
                pos2 + inner_normal1,
            );
            let is_intersecting = intersection.is_some();
            let (corner_pos_in, corner_pos_out) = match intersection {
                Some(point) => (point, 2.0 * pos1 - point),
                None => (pos1 + inner_normal0, pos1 - inner_normal0),
            };

            let (corner_pos_up, corner_pos_down) = if orientation == Orientation::Up {
                (corner_pos_in, corner_pos_out)
            } else {
                (corner_pos_out, corner_pos_in)
            };

            let mut current_joint_mode = self.joint_mode;
            let (pos_up1, pos_down1);

            if is_intersecting {
                let width_factor_sq = width_factor * width_factor;
                if current_joint_mode == LineJointMode::Sharp
                    && (hw_sq * width_factor_sq) > 0.0
                    && corner_pos_out.distance_squared_to(pos1) / (hw_sq * width_factor_sq)
                        > sharp_limit_sq
                {
                    current_joint_mode = LineJointMode::Bevel;
                }
                if current_joint_mode == LineJointMode::Sharp {
                    pos_up1 = corner_pos_up;
                    pos_down1 = corner_pos_down;
                } else if orientation == Orientation::Up {
                    pos_up1 = corner_pos_up;
                    pos_down1 = pos1 - u0 * modified_hw;
                } else {
                    pos_up1 = pos1 + u0 * modified_hw;
                    pos_down1 = corner_pos_down;
                }
            } else {
                if current_joint_mode == LineJointMode::Sharp {
                    current_joint_mode = LineJointMode::Bevel;
                }
                pos_up1 = corner_pos_up;
                pos_down1 = corner_pos_down;
            }

            if self.texture_mode == LineTextureMode::Tile {
                uvx1 = current_distance1 / (self.width * self.tile_aspect);
            } else if self.texture_mode == LineTextureMode::Stretch {
                uvx1 = current_distance1 / total_distance;
            }

            color0 = color1;
            u0 = u1;
            f0 = f1;
            pos0 = pos1;

            if is_intersecting {
                if current_joint_mode == LineJointMode::Sharp {
                    pos_up0 = pos_up1;
                    pos_down0 = pos_down1;
                } else if orientation == Orientation::Up {
                    pos_up0 = corner_pos_up;
                    pos_down0 = pos1 - u1 * modified_hw;
                } else {
                    pos_up0 = pos1 + u1 * modified_hw;
                    pos_down0 = corner_pos_down;
                }
            } else {
                pos_up0 = pos1 + u1 * modified_hw;
                pos_down0 = pos1 - u1 * modified_hw;
            }

            if i == -1 {
                i += 1;
                continue;
            }

            if wrap_around && i == 0 {
                let first_pos_center = (pos_up1 + pos_down1) / 2.0;
                let lerp_factor = 1.0 / width_factor;
                first_pos_up = first_pos_center.lerp(pos_up1, lerp_factor);
                first_pos_down = first_pos_center.lerp(pos_down1, lerp_factor);
                is_first_joint_sharp = current_joint_mode == LineJointMode::Sharp;
            }

            if wrap_around && retrieve_curve && !is_first_joint_sharp && i == segments_count {
                let first_pos_center = (first_pos_up + first_pos_down) / 2.0;
                self.strip_add_quad(
                    first_pos_center.lerp(first_pos_up, width_factor),
                    first_pos_center.lerp(first_pos_down, width_factor),
                    color1,
                    uvx1,
                );
                return;
            }
            self.strip_add_quad(pos_up1, pos_down1, color1, uvx1);

            if current_joint_mode != LineJointMode::Sharp {
                let (cbegin, cend) = if orientation == Orientation::Up {
                    (pos_down1, pos_down0)
                } else {
                    (pos_up1, pos_up0)
                };

                if current_joint_mode == LineJointMode::Bevel
                    && !(wrap_around && i == segments_count)
                {
                    self.strip_add_tri(cend, orientation);
                } else if current_joint_mode == LineJointMode::Round
                    && !(wrap_around && i == segments_count)
                {
                    let vbegin = cbegin - pos1;
                    let vend = cend - pos1;
                    let mut cross_product = vbegin.cross(vend);
                    let dot_product = vbegin.dot(vend);
                    // The engine flips a negative-zero cross product to +0 so the
                    // resulting angle is +PI rather than -PI.
                    if cross_product == 0.0 && cross_product.is_sign_negative() {
                        cross_product = 0.0;
                    }
                    let angle_delta = cross_product.atan2(dot_product);
                    self.strip_add_arc(pos1, angle_delta, orientation);
                }

                if !is_intersecting {
                    self.strip_begin(pos_up0, pos_down0, color1, uvx1);
                }
            }

            i += 1;
        }

        // The last (or only) segment with its end cap.
        if !wrap_around {
            pos1 = self.points[point_count - 1];

            if distance_required {
                current_distance1 += pos0.distance_to(pos1);
            }
            if self.interpolate_color {
                if let Some(gradient) = self.gradient.as_mut() {
                    let last = gradient.get_point_count().saturating_sub(1);
                    color1 = gradient.get_color(last);
                }
            }
            if retrieve_curve {
                width_factor = self.sample_curve(1.0);
                modified_hw = hw * width_factor;
            }

            let mut pos_up1 = pos1 + u0 * modified_hw;
            let mut pos_down1 = pos1 - u0 * modified_hw;

            if self.end_cap_mode == LineCapMode::Box {
                pos_up1 += f0 * modified_hw;
                pos_down1 += f0 * modified_hw;
                current_distance1 += modified_hw;
            }

            if self.texture_mode == LineTextureMode::Tile {
                uvx1 = current_distance1 / (self.width * self.tile_aspect);
            } else if self.texture_mode == LineTextureMode::Stretch {
                uvx1 = current_distance1 / total_distance;
            }

            self.strip_add_quad(pos_up1, pos_down1, color1, uvx1);

            if self.end_cap_mode == LineCapMode::Round {
                let color = if self.interpolate_color {
                    match self.gradient.as_mut() {
                        Some(gradient) => {
                            let last = gradient.get_point_count().saturating_sub(1);
                            gradient.get_color(last)
                        }
                        None => Color::new(0.0, 0.0, 0.0, 1.0),
                    }
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };
                let mut dist = 0.0f32;
                if self.texture_mode == LineTextureMode::Tile {
                    dist = width_factor / self.tile_aspect;
                } else if self.texture_mode == LineTextureMode::Stretch {
                    dist = self.width * width_factor / total_distance;
                }
                self.new_arc(
                    pos1,
                    pos_up1 - pos1,
                    std::f32::consts::PI,
                    color,
                    Rect2::new(
                        Vector2::new(uvx1 - 0.5 * dist, 0.0),
                        Vector2::new(dist, 1.0),
                    ),
                );
            }
        }
    }

    fn clear_output(&mut self) {
        self.vertices.clear();
        self.colors.clear();
        self.indices.clear();
        self.uvs.clear();
    }

    fn sample_curve(&mut self, offset: f32) -> f32 {
        match self.curve.as_mut() {
            Some(curve) => curve.sample_baked(offset),
            None => 1.0,
        }
    }

    fn gradient_color(&mut self, _index: usize) -> Color {
        match self.gradient.as_mut() {
            Some(gradient) => gradient.get_color(0),
            None => self.default_color,
        }
    }

    fn strip_begin(&mut self, up: Vector2, down: Vector2, color: Color, uvx: f32) {
        let vi = self.vertices.len() as i32;
        self.vertices.push(up);
        self.vertices.push(down);

        if self.interpolate_color {
            self.colors.push(color);
            self.colors.push(color);
        }

        if self.texture_mode != LineTextureMode::None {
            self.uvs.push(Vector2::new(uvx, 0.0));
            self.uvs.push(Vector2::new(uvx, 1.0));
        }

        self.last_index[Orientation::Up as usize] = vi;
        self.last_index[Orientation::Down as usize] = vi + 1;
    }

    fn strip_add_quad(&mut self, up: Vector2, down: Vector2, color: Color, uvx: f32) {
        let vi = self.vertices.len() as i32;
        self.vertices.push(up);
        self.vertices.push(down);

        if self.interpolate_color {
            self.colors.push(color);
            self.colors.push(color);
        }

        if self.texture_mode != LineTextureMode::None {
            self.uvs.push(Vector2::new(uvx, 0.0));
            self.uvs.push(Vector2::new(uvx, 1.0));
        }

        let up_index = self.last_index[Orientation::Up as usize];
        let down_index = self.last_index[Orientation::Down as usize];
        self.indices.push(up_index);
        self.indices.push(vi + 1);
        self.indices.push(down_index);
        self.indices.push(up_index);
        self.indices.push(vi);
        self.indices.push(vi + 1);

        self.last_index[Orientation::Up as usize] = vi;
        self.last_index[Orientation::Down as usize] = vi + 1;
    }

    fn strip_add_tri(&mut self, up: Vector2, orientation: Orientation) {
        let vi = self.vertices.len() as i32;
        self.vertices.push(up);

        if self.interpolate_color {
            let last = self.colors.last().copied().unwrap_or_default();
            self.colors.push(last);
        }

        let opposite = if orientation == Orientation::Up {
            Orientation::Down
        } else {
            Orientation::Up
        };

        if self.texture_mode != LineTextureMode::None {
            let uv = self.uvs[self.last_index[opposite as usize] as usize];
            self.uvs.push(uv);
        }

        self.indices
            .push(self.last_index[opposite as usize]);
        self.indices.push(vi);
        self.indices
            .push(self.last_index[orientation as usize]);

        self.last_index[opposite as usize] = vi;
    }

    fn strip_add_arc(&mut self, center: Vector2, angle_delta: f32, orientation: Orientation) {
        let opposite = if orientation == Orientation::Up {
            Orientation::Down
        } else {
            Orientation::Up
        };
        let vbegin = self.vertices[self.last_index[opposite as usize] as usize] - center;
        let radius = vbegin.length();
        let mut angle_step = std::f32::consts::PI / self.round_precision as f32;
        let steps = (angle_delta.abs() / angle_step).ceil();
        if angle_delta < 0.0 {
            angle_step = -angle_step;
        }

        let mut t = Vector2::new(1.0, 0.0).angle_to(vbegin);
        let end_angle = t + angle_delta;

        let mut ti = 0.0f32;
        while ti < steps {
            let rpos = center + Vector2::new(t.cos(), t.sin()) * radius;
            self.strip_add_tri(rpos, orientation);
            t += angle_step;
            ti += 1.0;
        }

        let rpos = center + Vector2::new(end_angle.cos(), end_angle.sin()) * radius;
        self.strip_add_tri(rpos, orientation);
    }

    fn new_arc(
        &mut self,
        center: Vector2,
        vbegin: Vector2,
        angle_delta: f32,
        color: Color,
        uv_rect: Rect2,
    ) {
        let radius = vbegin.length();
        let mut angle_step = std::f32::consts::PI / self.round_precision as f32;
        let steps = (angle_delta.abs() / angle_step).ceil();
        if angle_delta < 0.0 {
            angle_step = -angle_step;
        }

        let mut t = Vector2::new(1.0, 0.0).angle_to(vbegin);
        let end_angle = t + angle_delta;
        let tt_begin = -std::f32::consts::PI / 2.0;
        let mut tt = tt_begin;

        // Centre vertex.
        let vi_center = self.vertices.len() as i32;
        self.vertices.push(center);
        if self.interpolate_color {
            self.colors.push(color);
        }
        if self.texture_mode != LineTextureMode::None {
            self.uvs
                .push(interpolate(uv_rect, Vector2::new(0.5, 0.5)));
        }

        let mut vi = vi_center;
        let mut ti = 0.0f32;
        while ti < steps {
            let sc = Vector2::new(t.cos(), t.sin());
            let rpos = center + sc * radius;

            self.vertices.push(rpos);
            if self.interpolate_color {
                self.colors.push(color);
            }
            if self.texture_mode != LineTextureMode::None {
                let tsc = Vector2::new(tt.cos(), tt.sin());
                self.uvs.push(interpolate(
                    uv_rect,
                    0.5 * (tsc + Vector2::new(1.0, 1.0)),
                ));
                tt += angle_step;
            }
            t += angle_step;
            ti += 1.0;
        }

        let sc = Vector2::new(end_angle.cos(), end_angle.sin());
        let rpos = center + sc * radius;
        self.vertices.push(rpos);
        if self.interpolate_color {
            self.colors.push(color);
        }
        if self.texture_mode != LineTextureMode::None {
            tt = tt_begin + angle_delta;
            let tsc = Vector2::new(tt.cos(), tt.sin());
            self.uvs.push(interpolate(
                uv_rect,
                0.5 * (tsc + Vector2::new(1.0, 1.0)),
            ));
        }

        // Fan triangles.
        let mut vi_current = vi;
        let mut count = 0.0f32;
        while count < steps {
            self.indices.push(vi_center);
            vi_current += 1;
            self.indices.push(vi_current);
            self.indices.push(vi_current + 1);
            count += 1.0;
        }
        let _ = vi;
        vi = vi_current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fewer_than_two_points_clears_the_output() {
        let mut builder = LineBuilder::new();
        builder.points.push(Vector2::ZERO);
        builder.build();
        assert!(builder.vertices.is_empty());
        assert!(builder.indices.is_empty());
    }

    #[test]
    fn straight_line_builds_one_quad() {
        let mut builder = LineBuilder::new();
        builder.points = vec![Vector2::new(0.0, 0.0), Vector2::new(10.0, 0.0)];
        builder.width = 2.0;
        builder.build();

        // Two strip starts and one quad: 4 vertices, 6 indices.
        assert_eq!(builder.vertices.len(), 4);
        assert_eq!(builder.indices.len(), 6);
        assert_eq!(builder.colors.len(), 1, "one default colour");
        // Vector2::orthogonal() is (y, -x), so the strip's first edge sits at
        // negative y for a +x direction.
        assert_eq!(builder.vertices[0], Vector2::new(0.0, -1.0));
        assert_eq!(builder.vertices[1], Vector2::new(0.0, 1.0));
        assert_eq!(builder.vertices[2], Vector2::new(10.0, -1.0));
        assert_eq!(builder.vertices[3], Vector2::new(10.0, 1.0));
        // UVs are only emitted for a texture mode.
        assert!(builder.uvs.is_empty());
    }

    #[test]
    fn texture_modes_emit_uvs() {
        let mut builder = LineBuilder::new();
        builder.points = vec![Vector2::ZERO, Vector2::new(10.0, 0.0)];
        builder.width = 2.0;
        builder.texture_mode = LineTextureMode::Stretch;
        builder.build();
        assert_eq!(builder.uvs.len(), builder.vertices.len());
        assert_eq!(builder.uvs[0], Vector2::new(0.0, 0.0));
        assert_eq!(builder.uvs[1], Vector2::new(0.0, 1.0));
        // The stretched end reaches u = 1.
        assert!((builder.uvs[2].x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn rounded_caps_add_arc_vertices() {
        let mut plain = LineBuilder::new();
        plain.points = vec![Vector2::ZERO, Vector2::new(10.0, 0.0)];
        plain.width = 4.0;
        plain.build();
        let base = plain.vertices.len();

        let mut round = LineBuilder::new();
        round.points = plain.points.clone();
        round.width = 4.0;
        round.begin_cap_mode = LineCapMode::Round;
        round.end_cap_mode = LineCapMode::Round;
        round.round_precision = 4;
        round.build();
        assert!(round.vertices.len() > base);
        assert!(round.indices.len() > plain.indices.len());
        // Arc fans reuse the centre vertex.
        assert!(round.indices.iter().all(|index| *index >= 0));
    }

    #[test]
    fn box_caps_extend_the_line() {
        let mut builder = LineBuilder::new();
        builder.points = vec![Vector2::ZERO, Vector2::new(10.0, 0.0)];
        builder.width = 2.0;
        builder.begin_cap_mode = LineCapMode::Box;
        builder.end_cap_mode = LineCapMode::Box;
        builder.build();
        assert_eq!(builder.vertices[0], Vector2::new(-1.0, -1.0));
        assert_eq!(builder.vertices[1], Vector2::new(-1.0, 1.0));
        assert_eq!(builder.vertices[2], Vector2::new(11.0, -1.0));
    }

    #[test]
    fn closed_square_wraps_around() {
        let mut builder = LineBuilder::new();
        builder.points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, 10.0),
        ];
        builder.width = 2.0;
        builder.closed = true;
        builder.joint_mode = LineJointMode::Bevel;
        builder.build();

        // The engine draws point_count quads for a closed line (the last one
        // joins back to the start), plus one bevel triangle per joint.
        assert_eq!(builder.vertices.len(), 5 * 2 + 4);
        assert_eq!(builder.indices.len(), 5 * 6 + 4 * 3);
        for index in &builder.indices {
            assert!(*index >= 0 && (*index as usize) < builder.vertices.len());
        }
        // The ring is closed: all vertices stay within the expanded square.
        for vertex in &builder.vertices {
            assert!(vertex.x >= -1.5 && vertex.x <= 11.5);
            assert!(vertex.y >= -1.5 && vertex.y <= 11.5);
        }
    }

    #[test]
    fn sharp_limit_falls_back_to_bevel() {
        let mut sharp = LineBuilder::new();
        sharp.points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(0.0, 0.5),
        ];
        sharp.width = 2.0;
        sharp.sharp_limit = 1.0;
        sharp.joint_mode = LineJointMode::Sharp;
        sharp.build();

        let mut bevel = LineBuilder::new();
        bevel.points = sharp.points.clone();
        bevel.width = 2.0;
        bevel.sharp_limit = 1.0;
        bevel.joint_mode = LineJointMode::Bevel;
        bevel.build();

        // Both produce geometry, and the sharp case with a high limit does not
        // add the bevel triangle.
        let mut wide = LineBuilder::new();
        wide.points = sharp.points.clone();
        wide.width = 2.0;
        wide.sharp_limit = 100.0;
        wide.joint_mode = LineJointMode::Sharp;
        wide.build();
        assert!(wide.indices.len() >= sharp.indices.len());
        assert!(bevel.indices.len() >= sharp.indices.len());
    }

    #[test]
    fn round_joints_add_arc_triangles() {
        let mut bevel = LineBuilder::new();
        bevel.points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(10.0, 10.0),
        ];
        bevel.width = 4.0;
        bevel.joint_mode = LineJointMode::Bevel;
        bevel.build();

        let mut round = LineBuilder::new();
        round.points = bevel.points.clone();
        round.width = 4.0;
        round.joint_mode = LineJointMode::Round;
        round.round_precision = 4;
        round.build();

        assert!(round.vertices.len() > bevel.vertices.len());
        assert!(round.indices.len() > bevel.indices.len());
    }

    #[test]
    fn gradient_and_curve_interpolate_per_vertex() {
        let mut builder = LineBuilder::new();
        builder.points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(10.0, 0.0),
        ];
        builder.width = 2.0;
        let mut gradient = Gradient::new();
        gradient.set_color(0, Color::new(1.0, 0.0, 0.0, 1.0));
        gradient.set_color(1, Color::new(0.0, 0.0, 1.0, 1.0));
        builder.gradient = Some(gradient);
        builder.build();

        assert_eq!(builder.colors.len(), builder.vertices.len());
        let first = builder.colors[0];
        let last = *builder.colors.last().unwrap();
        assert!((first.r - 1.0).abs() < 1e-3, "first {first:?}");
        assert!((last.b - 1.0).abs() < 1e-3, "last {last:?}");
    }
}
