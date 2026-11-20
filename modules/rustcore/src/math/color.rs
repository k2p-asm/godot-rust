//! Port of `core/math/color.h` + `color.cpp` (numeric and parsing parts).
//!
//! Pending: OK-HSL/OK-HSV conversions (`thirdparty/misc/ok_color.h`, needs a
//! port of the Oklab implementation) and the named-color table
//! (`color_names.inc`).

use crate::math::funcs;
use crate::templates::hashfuncs;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }
}

impl Color {
    pub const ZERO: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const ONE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[inline]
    pub fn index(&self, i: usize) -> f32 {
        match i {
            0 => self.r,
            1 => self.g,
            2 => self.b,
            _ => self.a,
        }
    }

    #[inline]
    pub fn to_rgba32(&self) -> u32 {
        let mut c = (self.r * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.g * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.b * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.a * 255.0).round() as u8 as u32;
        c
    }

    #[inline]
    pub fn to_argb32(&self) -> u32 {
        let mut c = (self.a * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.r * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.g * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.b * 255.0).round() as u8 as u32;
        c
    }

    #[inline]
    pub fn to_abgr32(&self) -> u32 {
        let mut c = (self.a * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.b * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.g * 255.0).round() as u8 as u32;
        c <<= 8;
        c |= (self.r * 255.0).round() as u8 as u32;
        c
    }

    #[inline]
    pub fn to_rgba64(&self) -> u64 {
        let mut c = (self.r * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.g * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.b * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.a * 65535.0).round() as u16 as u64;
        c
    }

    #[inline]
    pub fn to_argb64(&self) -> u64 {
        let mut c = (self.a * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.r * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.g * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.b * 65535.0).round() as u16 as u64;
        c
    }

    #[inline]
    pub fn to_abgr64(&self) -> u64 {
        let mut c = (self.a * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.b * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.g * 65535.0).round() as u16 as u64;
        c <<= 16;
        c |= (self.r * 65535.0).round() as u16 as u64;
        c
    }

    /// `#rrggbb` or `#rrggbbaa`, lowercase, like the engine's `to_html`.
    pub fn to_html(&self, alpha: bool) -> String {
        fn hex(v: f32) -> String {
            let mut v = (v * 255.0).round() as i32;
            v = v.clamp(0, 255);
            format!("{v:02x}")
        }
        let mut s = String::new();
        s.push('#');
        s.push_str(&hex(self.r));
        s.push_str(&hex(self.g));
        s.push_str(&hex(self.b));
        if alpha {
            s.push_str(&hex(self.a));
        }
        s
    }

    #[inline]
    pub fn get_h(&self) -> f32 {
        let min = self.r.min(self.g).min(self.b);
        let max = self.r.max(self.g).max(self.b);
        let delta = max - min;
        if delta == 0.0 {
            return 0.0;
        }
        let mut h = if self.r == max {
            (self.g - self.b) / delta
        } else if self.g == max {
            2.0 + (self.b - self.r) / delta
        } else {
            4.0 + (self.r - self.g) / delta
        };
        h /= 6.0;
        if h < 0.0 {
            h += 1.0;
        }
        h
    }

    #[inline]
    pub fn get_s(&self) -> f32 {
        let min = self.r.min(self.g).min(self.b);
        let max = self.r.max(self.g).max(self.b);
        let delta = max - min;
        if max != 0.0 {
            delta / max
        } else {
            0.0
        }
    }

    #[inline]
    pub fn get_v(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    pub fn set_hsv(&mut self, mut h: f32, s: f32, v: f32, alpha: f32) {
        self.a = alpha;
        if s == 0.0 {
            self.r = v;
            self.g = v;
            self.b = v;
            return;
        }

        h *= 6.0;
        h = h % 6.0;
        let i = h.floor();
        let f = h - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match i as i32 {
            0 => {
                self.r = v;
                self.g = t;
                self.b = p;
            }
            1 => {
                self.r = q;
                self.g = v;
                self.b = p;
            }
            2 => {
                self.r = p;
                self.g = v;
                self.b = t;
            }
            3 => {
                self.r = p;
                self.g = q;
                self.b = v;
            }
            4 => {
                self.r = t;
                self.g = p;
                self.b = v;
            }
            _ => {
                self.r = v;
                self.g = p;
                self.b = q;
            }
        }
    }

    pub fn from_hsv(h: f32, s: f32, v: f32, alpha: f32) -> Self {
        let mut c = Color::default();
        c.set_hsv(h, s, v, alpha);
        c
    }

    #[inline]
    pub fn set_h(&mut self, h: f32) {
        self.set_hsv(h, self.get_s(), self.get_v(), self.a);
    }

    #[inline]
    pub fn set_s(&mut self, s: f32) {
        self.set_hsv(self.get_h(), s, self.get_v(), self.a);
    }

    #[inline]
    pub fn set_v(&mut self, v: f32) {
        self.set_hsv(self.get_h(), self.get_s(), v, self.a);
    }

    /// Okhsl -> sRGB (`ok_color::okhsl_to_srgb` followed by the engine's clamp).
    pub fn set_ok_hsl(&mut self, h: f32, s: f32, l: f32, alpha: f32) {
        let rgb = crate::math::ok_color::okhsl_to_srgb(crate::math::ok_color::Hsl { h, s, l });
        let col = Color::new(rgb.r, rgb.g, rgb.b, alpha).clamp(Color::ZERO, Color::ONE);
        *self = col;
    }

    pub fn from_ok_hsl(h: f32, s: f32, l: f32, alpha: f32) -> Self {
        let mut c = Color::default();
        c.set_ok_hsl(h, s, l, alpha);
        c
    }

    /// Okhsv -> sRGB.
    pub fn set_ok_hsv(&mut self, h: f32, s: f32, v: f32, alpha: f32) {
        let rgb = crate::math::ok_color::okhsv_to_srgb(crate::math::ok_color::Hsv { h, s, v });
        *self = Color::new(rgb.r, rgb.g, rgb.b, alpha).clamp(Color::ZERO, Color::ONE);
    }

    pub fn from_ok_hsv(h: f32, s: f32, v: f32, alpha: f32) -> Self {
        let mut c = Color::default();
        c.set_ok_hsv(h, s, v, alpha);
        c
    }
    #[inline]
    pub fn get_luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    pub fn lerp(&self, to: &Color, weight: f32) -> Self {
        Self {
            r: funcs::lerp(self.r, to.r, weight),
            g: funcs::lerp(self.g, to.g, weight),
            b: funcs::lerp(self.b, to.b, weight),
            a: funcs::lerp(self.a, to.a, weight),
        }
    }

    pub fn darkened(&self, amount: f32) -> Self {
        Self {
            r: self.r * (1.0 - amount),
            g: self.g * (1.0 - amount),
            b: self.b * (1.0 - amount),
            a: self.a,
        }
    }

    pub fn lightened(&self, amount: f32) -> Self {
        Self {
            r: self.r + (1.0 - self.r) * amount,
            g: self.g + (1.0 - self.g) * amount,
            b: self.b + (1.0 - self.b) * amount,
            a: self.a,
        }
    }

    pub fn blend(&self, over: &Color) -> Self {
        let sa = 1.0 - over.a;
        let a = self.a * sa + over.a;
        if a == 0.0 {
            return Color::new(0.0, 0.0, 0.0, 0.0);
        }
        Self {
            r: (self.r * self.a * sa + over.r * over.a) / a,
            g: (self.g * self.a * sa + over.g * over.a) / a,
            b: (self.b * self.a * sa + over.b * over.a) / a,
            a,
        }
    }

    pub fn srgb_to_linear(&self) -> Self {
        fn conv(c: f32) -> f32 {
            if c < 0.04045 {
                c * (1.0 / 12.92)
            } else {
                ((f64::from(c + 0.055) * (1.0 / (1.0 + 0.055))).powf(2.4)) as f32
            }
        }
        Self {
            r: conv(self.r),
            g: conv(self.g),
            b: conv(self.b),
            a: self.a,
        }
    }

    pub fn linear_to_srgb(&self) -> Self {
        fn conv(c: f32) -> f32 {
            if c < 0.0031308 {
                12.92 * c
            } else {
                (1.0 + 0.055) * f32::powf(c, 1.0 / 2.4) - 0.055
            }
        }
        Self {
            r: conv(self.r),
            g: conv(self.g),
            b: conv(self.b),
            a: self.a,
        }
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Color) -> bool {
        funcs::is_equal_approx(self.r, other.r)
            && funcs::is_equal_approx(self.g, other.g)
            && funcs::is_equal_approx(self.b, other.b)
            && funcs::is_equal_approx(self.a, other.a)
    }

    #[inline]
    pub fn is_same(&self, other: &Color) -> bool {
        funcs::is_same(self.r, other.r)
            && funcs::is_same(self.g, other.g)
            && funcs::is_same(self.b, other.b)
            && funcs::is_same(self.a, other.a)
    }

    pub fn clamp(&self, min: Color, max: Color) -> Self {
        Self {
            r: funcs::clamp_f32(self.r, min.r, max.r),
            g: funcs::clamp_f32(self.g, min.g, max.g),
            b: funcs::clamp_f32(self.b, min.b, max.b),
            a: funcs::clamp_f32(self.a, min.a, max.a),
        }
    }

    pub fn invert(&mut self) {
        self.r = 1.0 - self.r;
        self.g = 1.0 - self.g;
        self.b = 1.0 - self.b;
    }

    pub fn inverted(&self) -> Self {
        let mut c = *self;
        c.invert();
        c
    }

    /// `0xRRGGBBAA` packed value.
    pub fn hex(hex: u32) -> Self {
        let a = (hex & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        Self::new(r, g, b, a)
    }

    pub fn hex64(hex: u64) -> Self {
        let a = (hex & 0xFFFF) as f32 / 65535.0;
        let b = ((hex >> 16) & 0xFFFF) as f32 / 65535.0;
        let g = ((hex >> 32) & 0xFFFF) as f32 / 65535.0;
        let r = ((hex >> 48) & 0xFFFF) as f32 / 65535.0;
        Self::new(r, g, b, a)
    }

    pub fn from_rgba8(r: i64, g: i64, b: i64, a: i64) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    /// Parses `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` (the `#` is optional).
    /// Returns `None` on invalid input, matching the engine's error fallback.
    pub fn html(code: &str) -> Option<Self> {
        let s = code.strip_prefix('#').unwrap_or(code);
        let mut vals = [0f32; 4];
        vals[3] = 1.0;

        let chars: Vec<char> = s.chars().collect();
        match chars.len() {
            3 | 4 => {
                for (i, c) in chars.iter().enumerate() {
                    vals[i] = c.to_digit(16)? as f32 / 15.0;
                }
            }
            6 | 8 => {
                for i in 0..chars.len() / 2 {
                    let hi = chars[i * 2].to_digit(16)?;
                    let lo = chars[i * 2 + 1].to_digit(16)?;
                    vals[i] = (hi * 16 + lo) as f32 / 255.0;
                }
            }
            _ => return None,
        }

        Some(Self::new(vals[0], vals[1], vals[2], vals[3]))
    }

    /// Normalizes a color name the way the engine does: strips ` -_'.` and
    /// uppercases, then looks it up in the generated table.
    pub fn find_named_color(name: &str) -> Option<usize> {
        // The engine strips ' -_, .' from the input AND from the table keys.
        let normalized: String = name
            .chars()
            .filter(|c| *c != ' ' && *c != '-' && *c != '_' && *c != '\'' && *c != '.')
            .flat_map(|c| c.to_uppercase())
            .collect();
        crate::math::color_names::NAMED_COLORS.iter().position(|(n, _)| {
            n.chars()
                .filter(|c| *c != '_')
                .flat_map(|c| c.to_uppercase())
                .eq(normalized.chars())
        })
    }

    /// `Color::named(name)`; `None` where the engine errors out.
    pub fn named(name: &str) -> Option<Self> {
        let idx = Self::find_named_color(name)?;
        Some(Self::hex(crate::math::color_names::NAMED_COLORS[idx].1))
    }

    /// `Color::named(name, default)`.
    pub fn named_or(name: &str, default: Color) -> Self {
        Self::named(name).unwrap_or(default)
    }

    #[inline]
    pub fn named_count() -> usize {
        crate::math::color_names::NAMED_COLORS.len()
    }

    pub fn named_at(idx: usize) -> Option<(&'static str, Color)> {
        crate::math::color_names::NAMED_COLORS
            .get(idx)
            .map(|(n, v)| (*n, Self::hex(*v)))
    }
    #[inline]
    pub fn hash(&self) -> u32 {
        let mut h = hashfuncs::hash_murmur3_one_float(self.r, hashfuncs::HASH_MURMUR3_SEED);
        h = hashfuncs::hash_murmur3_one_float(self.g, h);
        h = hashfuncs::hash_murmur3_one_float(self.b, h);
        h = hashfuncs::hash_murmur3_one_float(self.a, h);
        hashfuncs::hash_fmix32(h)
    }

    /// Packed RGBE9995, as used by the lightmapper and GI.
    pub fn to_rgbe9995(&self) -> u32 {
        const MAX_VAL: f32 = (0x1FFu32 << 7) as f32;
        const MIN_VAL: f32 = 1.0 / (1 << 16) as f32;

        let r = funcs::clamp_f32(self.r, 0.0, MAX_VAL);
        let g = funcs::clamp_f32(self.g, 0.0, MAX_VAL);
        let b = funcs::clamp_f32(self.b, 0.0, MAX_VAL);

        let max_channel = r.max(g).max(b.max(MIN_VAL));

        let mut e_bits = max_channel.to_bits();
        e_bits = e_bits.wrapping_add(0x07804000);
        e_bits &= 0x7F800000;
        let e = f32::from_bits(e_bits);

        let rf = (r + e).to_bits();
        let gf = (g + e).to_bits();
        let bf = (b + e).to_bits();

        let mut e_shifted = e_bits << 4;
        e_shifted = e_shifted.wrapping_add(0x10000000);

        e_shifted | (bf << 18) | (gf << 9) | (rf & 511)
    }

    pub fn from_rgbe9995(rgbe: u32) -> Self {
        let r = (rgbe & 0x1ff) as f32;
        let g = ((rgbe >> 9) & 0x1ff) as f32;
        let b = ((rgbe >> 18) & 0x1ff) as f32;
        let e = (rgbe >> 27) as f32;
        let m = 2.0f32.powf(e - 15.0 - 9.0);
        Self::new(r * m, g * m, b * m, 1.0)
    }
}
macro_rules! color_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Color {
            type Output = Color;
            #[inline]
            fn $method(self, o: Color) -> Color {
                Color::new(self.r $op o.r, self.g $op o.g, self.b $op o.b, self.a $op o.a)
            }
        }
        impl std::ops::$trait<f32> for Color {
            type Output = Color;
            #[inline]
            fn $method(self, s: f32) -> Color {
                Color::new(self.r $op s, self.g $op s, self.b $op s, self.a $op s)
            }
        }
    };
}

color_binop!(Add, add, +);
color_binop!(Sub, sub, -);
color_binop!(Mul, mul, *);
color_binop!(Div, div, /);

impl std::ops::Neg for Color {
    type Output = Color;
    #[inline]
    fn neg(self) -> Color {
        Color::new(-self.r, -self.g, -self.b, -self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packing() {
        let c = Color::new(1.0, 0.5, 0.0, 1.0);
        assert_eq!(c.to_rgba32(), 0xff_80_00_ff);
        assert_eq!(c.to_argb32(), 0xff_ff_80_00);
        assert_eq!(Color::hex(0x00ff00ff), Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(c.to_html(true), "#ff8000ff");
        assert_eq!(c.to_html(false), "#ff8000");
    }

    #[test]
    fn hsv_roundtrip() {
        let c = Color::from_hsv(0.5, 1.0, 1.0, 1.0);
        assert!(c.is_equal_approx(&Color::new(0.0, 1.0, 1.0, 1.0)), "{c:?}");
        let c2 = Color::from_hsv(0.0, 0.0, 0.42, 1.0);
        assert!(c2.is_equal_approx(&Color::new(0.42, 0.42, 0.42, 1.0)));
        assert!((c.get_h() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn html_parsing() {
        assert_eq!(Color::html("#ff0000"), Some(Color::new(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(Color::html("0f0"), Some(Color::new(0.0, 1.0, 0.0, 1.0)));
        assert_eq!(Color::html("#0000ff80").unwrap().a, 128.0 / 255.0);
        assert_eq!(Color::html("nope"), None);
    }

    #[test]
    fn blending_and_luminance() {
        let base = Color::new(0.0, 0.0, 0.0, 1.0);
        let over = Color::new(1.0, 1.0, 1.0, 0.5);
        let b = base.blend(&over);
        assert!((b.r - 0.5).abs() < 1e-6);
        assert!((b.a - 1.0).abs() < 1e-6);
        assert!((Color::new(1.0, 1.0, 1.0, 1.0).get_luminance() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn linear_srgb_roundtrip() {
        for v in [0.0f32, 0.02, 0.25, 0.5, 0.75, 1.0] {
            let c = Color::rgb(v, v, v);
            let back = c.srgb_to_linear().linear_to_srgb();
            assert!((back.r - v).abs() < 1e-5, "{v} -> {}", back.r);
        }
    }
}
