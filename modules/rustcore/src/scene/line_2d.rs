//! `scene/2d/line_2d.*`: state and width sampling.
//!
//! Stored here: the point list, closed flag, base width, the optional `Curve`
//! that scales the width along the line, default color, optional `Gradient`,
//! joint/cap modes, sharp limit, round precision and texture mode.
//!
//! Not ported: `_draw`'s triangle generation (strip/bevel/round joins and caps),
//! which is pure geometry but large; `get_width_at` gives the value it uses.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::scene::curve::Curve;
use crate::scene::gradient::Gradient;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineJointMode {
    Sharp,
    Bevel,
    Round,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineCapMode {
    None,
    Box,
    Round,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineTextureMode {
    None,
    Tile,
    Stretch,
}

pub struct Line2DData {
    points: Vec<Vector2>,
    closed: bool,
    width: f32,
    curve: Option<Curve>,
    default_color: Color,
    gradient: Option<Gradient>,
    joint_mode: LineJointMode,
    begin_cap: LineCapMode,
    end_cap: LineCapMode,
    sharp_limit: f32,
    round_precision: i32,
    texture_mode: LineTextureMode,
    antialiased: bool,
}

impl Default for Line2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Line2DData {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
            width: 0.0,
            curve: None,
            default_color: Color::new(1.0, 1.0, 1.0, 1.0),
            gradient: None,
            joint_mode: LineJointMode::Sharp,
            begin_cap: LineCapMode::None,
            end_cap: LineCapMode::None,
            sharp_limit: 2.0,
            round_precision: 8,
            texture_mode: LineTextureMode::None,
            antialiased: false,
        }
    }

    pub fn set_points(&mut self, points: Vec<Vector2>) {
        self.points = points;
    }

    pub fn get_points(&self) -> &[Vector2] {
        &self.points
    }

    pub fn set_point_position(&mut self, index: usize, pos: Vector2) -> bool {
        match self.points.get_mut(index) {
            Some(p) => {
                *p = pos;
                true
            }
            None => false,
        }
    }

    pub fn get_point_position(&self, index: usize) -> Option<Vector2> {
        self.points.get(index).copied()
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
    }

    /// `add_point`: negative `atpos` (or one past the end) appends.
    pub fn add_point(&mut self, pos: Vector2, atpos: i32) {
        if atpos < 0 || self.points.len() < atpos as usize {
            self.points.push(pos);
        } else {
            self.points.insert(atpos as usize, pos);
        }
    }

    pub fn remove_point(&mut self, index: usize) -> bool {
        if index >= self.points.len() {
            return false;
        }
        self.points.remove(index);
        true
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// `set_width`: negative widths are clamped to zero.
    pub fn set_width(&mut self, width: f32) {
        self.width = if width < 0.0 { 0.0 } else { width };
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    /// Width at `offset` in `[0, 1]`: the base width scaled by the curve, which
    /// is what `_draw` uses per tessellated point.
    pub fn get_width_at(&self, offset: f32) -> f32 {
        match &self.curve {
            Some(c) => self.width * c.sample(offset),
            None => self.width,
        }
    }

    pub fn set_curve(&mut self, curve: Option<Curve>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<&Curve> {
        self.curve.as_ref()
    }

    pub fn set_default_color(&mut self, color: Color) {
        self.default_color = color;
    }

    pub fn get_default_color(&self) -> Color {
        self.default_color
    }

    pub fn set_gradient(&mut self, gradient: Option<Gradient>) {
        self.gradient = gradient;
    }

    pub fn get_gradient(&self) -> Option<&Gradient> {
        self.gradient.as_ref()
    }

    pub fn set_texture_mode(&mut self, mode: LineTextureMode) {
        self.texture_mode = mode;
    }

    pub fn get_texture_mode(&self) -> LineTextureMode {
        self.texture_mode
    }

    pub fn set_joint_mode(&mut self, mode: LineJointMode) {
        self.joint_mode = mode;
    }

    pub fn get_joint_mode(&self) -> LineJointMode {
        self.joint_mode
    }

    pub fn set_begin_cap_mode(&mut self, mode: LineCapMode) {
        self.begin_cap = mode;
    }

    pub fn get_begin_cap_mode(&self) -> LineCapMode {
        self.begin_cap
    }

    pub fn set_end_cap_mode(&mut self, mode: LineCapMode) {
        self.end_cap = mode;
    }

    pub fn get_end_cap_mode(&self) -> LineCapMode {
        self.end_cap
    }

    /// `set_sharp_limit`: negative values clamp to zero.
    pub fn set_sharp_limit(&mut self, limit: f32) {
        self.sharp_limit = if limit < 0.0 { 0.0 } else { limit };
    }

    pub fn get_sharp_limit(&self) -> f32 {
        self.sharp_limit
    }

    /// `set_round_precision`: at least 1.
    pub fn set_round_precision(&mut self, precision: i32) {
        self.round_precision = precision.max(1);
    }

    pub fn get_round_precision(&self) -> i32 {
        self.round_precision
    }

    pub fn set_antialiased(&mut self, antialiased: bool) {
        self.antialiased = antialiased;
    }

    pub fn get_antialiased(&self) -> bool {
        self.antialiased
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = Line2DData::new();
        assert_eq!(l.get_point_count(), 0);
        assert!(!l.is_closed());
        assert_eq!(l.get_width(), 0.0);
        assert_eq!(l.get_joint_mode(), LineJointMode::Sharp);
        assert_eq!(l.get_begin_cap_mode(), LineCapMode::None);
        assert_eq!(l.get_end_cap_mode(), LineCapMode::None);
        assert_eq!(l.get_round_precision(), 8);
        assert_eq!(l.get_default_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(l.get_texture_mode(), LineTextureMode::None);
        assert!(!l.get_antialiased());
    }

    #[test]
    fn point_list_operations() {
        let mut l = Line2DData::new();
        l.add_point(Vector2::ZERO, -1);
        l.add_point(Vector2::new(1.0, 0.0), -1);
        l.add_point(Vector2::new(0.5, 1.0), 1);
        assert_eq!(l.get_point_count(), 3);
        assert_eq!(l.get_point_position(1), Some(Vector2::new(0.5, 1.0)));

        assert!(l.set_point_position(0, Vector2::new(-1.0, 0.0)));
        assert_eq!(l.get_point_position(0), Some(Vector2::new(-1.0, 0.0)));
        assert!(!l.set_point_position(9, Vector2::ZERO));

        assert!(l.remove_point(1));
        assert_eq!(l.get_point_count(), 2);
        assert!(!l.remove_point(5));

        l.clear_points();
        assert_eq!(l.get_point_count(), 0);
    }

    #[test]
    fn width_and_curve_scaling() {
        let mut l = Line2DData::new();
        l.set_width(-3.0);
        assert_eq!(l.get_width(), 0.0);
        l.set_width(10.0);
        assert_eq!(l.get_width(), 10.0);

        // Curve value 0.25 at offset 0 scales the width to 2.5.
        let mut c = Curve::new();
        c.add_point(Vector2::new(0.0, 0.25));
        c.add_point(Vector2::new(1.0, 1.0));
        l.set_curve(Some(c));
        assert!((l.get_width_at(0.0) - 2.5).abs() < 1e-4);
        // Endpoint of the curve is 1.0 -> 10.
        assert!((l.get_width_at(1.0) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn setters_clamp_like_the_engine() {
        let mut l = Line2DData::new();
        l.set_sharp_limit(-2.0);
        assert_eq!(l.get_sharp_limit(), 0.0);
        l.set_round_precision(0);
        assert_eq!(l.get_round_precision(), 1);
        l.set_round_precision(16);
        assert_eq!(l.get_round_precision(), 16);

        l.set_closed(true);
        assert!(l.is_closed());
        l.set_joint_mode(LineJointMode::Round);
        assert_eq!(l.get_joint_mode(), LineJointMode::Round);
        l.set_begin_cap_mode(LineCapMode::Box);
        assert_eq!(l.get_begin_cap_mode(), LineCapMode::Box);
    }
}
