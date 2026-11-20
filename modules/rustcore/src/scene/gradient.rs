//! `scene/resources/gradient.*`: color stops with three interpolation modes.
//!
//! Points are sorted by offset on first use. `get_color_at_offset` reproduces
//! the engine's binary search exactly, including the `middle--` correction and
//! the clamp to the first/last color outside the stop range. Linear and cubic
//! modes can interpolate in sRGB, linear sRGB or Oklab.
//!
//! Not ported: `Resource`/`ClassDB` plumbing and `GradientTexture`. The engine
//! has no bake on `Gradient` itself.

use crate::math::color::Color;
use crate::math::funcs;
use crate::math::ok_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterpolationMode {
    Linear,
    Constant,
    Cubic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    LinearSrgb,
    Oklab,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    offset: f32,
    color: Color,
}

pub struct Gradient {
    points: Vec<Point>,
    is_sorted: bool,
    interpolation_mode: InterpolationMode,
    interpolation_color_space: ColorSpace,
}

impl Default for Gradient {
    fn default() -> Self {
        Self::new()
    }
}

impl Gradient {
    /// The engine starts with a black-to-white gradient.
    pub fn new() -> Self {
        Self {
            points: vec![
                Point { offset: 0.0, color: Color::new(0.0, 0.0, 0.0, 1.0) },
                Point { offset: 1.0, color: Color::new(1.0, 1.0, 1.0, 1.0) },
            ],
            is_sorted: true,
            interpolation_mode: InterpolationMode::Linear,
            interpolation_color_space: ColorSpace::Srgb,
        }
    }

    fn update_sorting(&mut self) {
        if !self.is_sorted {
            self.points.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));
            self.is_sorted = true;
        }
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn get_offsets(&self) -> Vec<f32> {
        self.points.iter().map(|p| p.offset).collect()
    }

    pub fn get_colors(&self) -> Vec<Color> {
        self.points.iter().map(|p| p.color).collect()
    }

    pub fn set_offsets(&mut self, offsets: &[f32]) {
        self.points.resize(
            offsets.len(),
            Point { offset: 0.0, color: Color::new(0.0, 0.0, 0.0, 1.0) },
        );
        for (p, o) in self.points.iter_mut().zip(offsets) {
            p.offset = *o;
        }
        self.is_sorted = false;
    }

    pub fn set_colors(&mut self, colors: &[Color]) {
        if self.points.len() < colors.len() {
            self.is_sorted = false;
        }
        self.points.resize(
            colors.len(),
            Point { offset: 0.0, color: Color::new(0.0, 0.0, 0.0, 1.0) },
        );
        for (p, c) in self.points.iter_mut().zip(colors) {
            p.color = *c;
        }
    }

    pub fn add_point(&mut self, offset: f32, color: Color) {
        self.points.push(Point { offset, color });
        self.is_sorted = false;
    }

    /// Refuses to remove the last point, like the engine.
    pub fn remove_point(&mut self, index: usize) -> bool {
        if index >= self.points.len() || self.points.len() <= 1 {
            return false;
        }
        self.points.remove(index);
        true
    }

    pub fn reverse(&mut self) {
        for p in &mut self.points {
            p.offset = 1.0 - p.offset;
        }
        self.is_sorted = false;
        self.update_sorting();
    }

    pub fn set_offset(&mut self, index: usize, offset: f32) {
        if index >= self.points.len() {
            return;
        }
        self.update_sorting();
        if let Some(p) = self.points.get_mut(index) {
            p.offset = offset;
        }
        self.is_sorted = false;
    }

    pub fn get_offset(&mut self, index: usize) -> f32 {
        if index >= self.points.len() {
            return 0.0;
        }
        self.update_sorting();
        self.points[index].offset
    }

    pub fn set_color(&mut self, index: usize, color: Color) {
        if index >= self.points.len() {
            return;
        }
        self.update_sorting();
        if let Some(p) = self.points.get_mut(index) {
            p.color = color;
        }
    }

    pub fn get_color(&mut self, index: usize) -> Color {
        if index >= self.points.len() {
            return Color::new(0.0, 0.0, 0.0, 1.0);
        }
        self.update_sorting();
        self.points[index].color
    }

    pub fn set_interpolation_mode(&mut self, mode: InterpolationMode) {
        self.interpolation_mode = mode;
    }

    pub fn get_interpolation_mode(&self) -> InterpolationMode {
        self.interpolation_mode
    }

    pub fn set_interpolation_color_space(&mut self, space: ColorSpace) {
        self.interpolation_color_space = space;
    }

    pub fn get_interpolation_color_space(&self) -> ColorSpace {
        self.interpolation_color_space
    }

    fn transform_color_space(&self, color: Color) -> Color {
        match self.interpolation_color_space {
            ColorSpace::Srgb => color,
            ColorSpace::LinearSrgb => color.srgb_to_linear(),
            ColorSpace::Oklab => {
                let linear = color.srgb_to_linear();
                let lab = ok_color::linear_srgb_to_oklab(ok_color::Rgb {
                    r: linear.r,
                    g: linear.g,
                    b: linear.b,
                });
                Color::new(lab.l, lab.a, lab.b, color.a)
            }
        }
    }

    fn inv_transform_color_space(&self, color: Color) -> Color {
        match self.interpolation_color_space {
            ColorSpace::Srgb => color,
            ColorSpace::LinearSrgb => color.linear_to_srgb(),
            ColorSpace::Oklab => {
                let rgb = ok_color::oklab_to_linear_srgb(ok_color::Lab {
                    l: color.r,
                    a: color.g,
                    b: color.b,
                });
                Color::new(rgb.r, rgb.g, rgb.b, color.a).linear_to_srgb()
            }
        }
    }

    /// `get_color_at_offset`: binary search, then constant/linear/cubic.
    pub fn get_color_at_offset(&mut self, offset: f32) -> Color {
        if self.points.is_empty() {
            return Color::new(0.0, 0.0, 0.0, 1.0);
        }
        self.update_sorting();

        let n = self.points.len() as i32;
        let mut low = 0i32;
        let mut high = n - 1;
        let mut middle = 0i32;

        while low <= high {
            middle = (low + high) / 2;
            let point = self.points[middle as usize];
            if point.offset > offset {
                high = middle - 1;
            } else if point.offset < offset {
                low = middle + 1;
            } else {
                return point.color;
            }
        }

        if self.points[middle as usize].offset > offset {
            middle -= 1;
        }
        let first = middle;
        let second = middle + 1;
        if second >= n {
            return self.points[n as usize - 1].color;
        }
        if first < 0 {
            return self.points[0].color;
        }

        let point1 = self.points[first as usize];
        let point2 = self.points[second as usize];
        let weight = (offset - point1.offset) / (point2.offset - point1.offset);

        match self.interpolation_mode {
            InterpolationMode::Constant => point1.color,
            InterpolationMode::Linear => {
                let c1 = self.transform_color_space(point1.color);
                let c2 = self.transform_color_space(point2.color);
                self.inv_transform_color_space(c1.lerp(&c2, weight))
            }
            InterpolationMode::Cubic => {
                let mut p0 = first - 1;
                let mut p3 = second + 1;
                if p3 >= n {
                    p3 = second;
                }
                if p0 < 0 {
                    p0 = first;
                }
                let c0 = self.transform_color_space(self.points[p0 as usize].color);
                let c1 = self.transform_color_space(point1.color);
                let c2 = self.transform_color_space(point2.color);
                let c3 = self.transform_color_space(self.points[p3 as usize].color);

                let mut interpolated = Color::new(0.0, 0.0, 0.0, 0.0);
                let ch = |a: Color, i: usize| [a.r, a.g, a.b, a.a][i];
                let mut out = [0.0f32; 4];
                for i in 0..4 {
                    out[i] = funcs::cubic_interpolate(ch(c1, i), ch(c2, i), ch(c0, i), ch(c3, i), weight);
                }
                interpolated.r = out[0];
                interpolated.g = out[1];
                interpolated.b = out[2];
                interpolated.a = out[3];
                self.inv_transform_color_space(interpolated)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Color, b: Color) -> bool {
        (a.r - b.r).abs() < 1e-5
            && (a.g - b.g).abs() < 1e-5
            && (a.b - b.b).abs() < 1e-5
            && (a.a - b.a).abs() < 1e-5
    }

    #[test]
    fn defaults_are_black_to_white() {
        let mut g = Gradient::new();
        assert_eq!(g.get_point_count(), 2);
        assert!(close(g.get_color_at_offset(0.0), Color::new(0.0, 0.0, 0.0, 1.0)));
        assert!(close(g.get_color_at_offset(1.0), Color::new(1.0, 1.0, 1.0, 1.0)));
        assert!(close(g.get_color_at_offset(0.5), Color::new(0.5, 0.5, 0.5, 1.0)));
    }

    #[test]
    fn outside_the_range_clamps_to_the_ends() {
        let mut g = Gradient::new();
        assert!(close(g.get_color_at_offset(-1.0), Color::new(0.0, 0.0, 0.0, 1.0)));
        assert!(close(g.get_color_at_offset(2.0), Color::new(1.0, 1.0, 1.0, 1.0)));
    }

    #[test]
    fn points_are_sorted_on_first_sample() {
        let mut g = Gradient::new();
        g.add_point(0.25, Color::new(1.0, 0.0, 0.0, 1.0));
        g.add_point(0.75, Color::new(0.0, 0.0, 1.0, 1.0));

        assert_eq!(g.get_offsets(), vec![0.0, 1.0, 0.25, 0.75]);
        // Sampling triggers the sort; red at 0.25 exactly.
        assert!(close(g.get_color_at_offset(0.25), Color::new(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(g.get_offsets(), vec![0.0, 0.25, 0.75, 1.0]);
    }

    #[test]
    fn constant_mode_holds_the_left_color() {
        let mut g = Gradient::new();
        g.set_interpolation_mode(InterpolationMode::Constant);
        // Anywhere in [0,1) the answer is the color of the left stop (black).
        assert!(close(g.get_color_at_offset(0.1), Color::new(0.0, 0.0, 0.0, 1.0)));
        assert!(close(g.get_color_at_offset(0.9), Color::new(0.0, 0.0, 0.0, 1.0)));
        // Exactly at the right stop, the search returns its color.
        assert!(close(g.get_color_at_offset(1.0), Color::new(1.0, 1.0, 1.0, 1.0)));
    }

    #[test]
    fn cubic_mode_hits_the_stops() {
        let mut g = Gradient::new();
        g.set_interpolation_mode(InterpolationMode::Cubic);
        assert!(close(g.get_color_at_offset(0.0), Color::new(0.0, 0.0, 0.0, 1.0)));
        assert!(close(g.get_color_at_offset(1.0), Color::new(1.0, 1.0, 1.0, 1.0)));
        let mid = g.get_color_at_offset(0.5);
        assert!(mid.r.is_finite() && (0.0..=1.0).contains(&mid.r), "{mid:?}");
    }

    #[test]
    fn reverse_mirrors_the_offsets() {
        let mut g = Gradient::new();
        g.reverse();
        assert_eq!(g.get_offsets(), vec![0.0, 1.0]);
        // Colors travel with their points, so black is now at 1.0.
        assert!(close(g.get_color_at_offset(0.0), Color::new(1.0, 1.0, 1.0, 1.0)));
        assert!(close(g.get_color_at_offset(1.0), Color::new(0.0, 0.0, 0.0, 1.0)));
    }

    #[test]
    fn set_offset_resorts_on_next_read() {
        let mut g = Gradient::new();
        g.add_point(0.5, Color::new(0.0, 1.0, 0.0, 1.0));
        g.get_color_at_offset(0.0); // sort
        g.set_offset(1, 0.9); // green stop moves to 0.9
        assert_eq!(g.get_offset(1), 0.9);
        // After the re-sort the white stop is the last one.
        assert_eq!(g.get_offset(2), 1.0);
        assert!(close(g.get_color_at_offset(0.9), Color::new(0.0, 1.0, 0.0, 1.0)));
    }

    #[test]
    fn cannot_remove_the_last_point() {
        let mut g = Gradient::new();
        assert!(g.remove_point(0));
        assert_eq!(g.get_point_count(), 1);
        assert!(!g.remove_point(0));
        assert_eq!(g.get_point_count(), 1);
    }

    #[test]
    fn oklab_midpoint_differs_from_srgb_but_keeps_the_ends() {
        let mut g = Gradient::new();
        let srgb_mid = g.get_color_at_offset(0.5);

        g.set_interpolation_color_space(ColorSpace::Oklab);
        let oklab_mid = g.get_color_at_offset(0.5);
        assert!(close(g.get_color_at_offset(0.0), Color::new(0.0, 0.0, 0.0, 1.0)));
        assert!(close(g.get_color_at_offset(1.0), Color::new(1.0, 1.0, 1.0, 1.0)));
        // Oklab's L = 0.5 is darker in sRGB than a plain 50% lerp (0.389 vs
        // 0.5), which is exactly why interpolating there matters.
        assert!(
            (oklab_mid.r - srgb_mid.r).abs() > 0.05,
            "{oklab_mid:?} vs {srgb_mid:?}"
        );
    }

    #[test]
    fn offsets_and_colors_round_trip() {
        let mut g = Gradient::new();
        g.set_offsets(&[0.0, 0.5, 1.0]);
        g.set_colors(&[
            Color::new(1.0, 0.0, 0.0, 1.0),
            Color::new(0.0, 1.0, 0.0, 1.0),
            Color::new(0.0, 0.0, 1.0, 1.0),
        ]);
        assert_eq!(g.get_point_count(), 3);
        assert_eq!(g.get_offsets(), vec![0.0, 0.5, 1.0]);
        assert_eq!(g.get_colors()[1], Color::new(0.0, 1.0, 0.0, 1.0));
    }
}
