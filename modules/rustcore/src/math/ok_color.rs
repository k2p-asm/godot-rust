//! Port of `thirdparty/misc/ok_color.h` (Björn Ottosson's Okhsl/Okhsv).
//!
//! Kept structurally identical to the original, including the Halley steps and
//! the coefficient tables. `get_ST_mid` is a fitted polynomial and
//! `compute_max_saturation` switches coefficient sets per region: neither may be
//! "cleaned up" without changing the output.

pub const PI: f32 = 3.14159265358979323846264338327950288;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Lc {
    pub l: f32,
    pub c: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct St {
    pub s: f32,
    pub t: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Cs {
    pub c_0: f32,
    pub c_mid: f32,
    pub c_max: f32,
}

#[inline]
fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

/// `(0 < x) - (x < 0)`, i.e. zero maps to 0 here (unlike Godot's `SIGN`).
#[inline]
fn sgn(x: f32) -> f32 {
    (x > 0.0) as i32 as f32 - (x < 0.0) as i32 as f32
}

#[inline]
pub fn srgb_transfer_function(a: f32) -> f32 {
    if 0.0031308 >= a {
        12.92 * a
    } else {
        1.055 * a.powf(0.4166666666666667) - 0.055
    }
}

#[inline]
pub fn srgb_transfer_function_inv(a: f32) -> f32 {
    if 0.04045 < a {
        ((a + 0.055) / 1.055).powf(2.4)
    } else {
        a / 12.92
    }
}

pub fn linear_srgb_to_oklab(c: Rgb) -> Lab {
    let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    Lab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

pub fn oklab_to_linear_srgb(c: Lab) -> Rgb {
    let l_ = c.l + 0.3963377774 * c.a + 0.2158037573 * c.b;
    let m_ = c.l - 0.1055613458 * c.a - 0.0638541728 * c.b;
    let s_ = c.l - 0.0894841775 * c.a - 1.2914855480 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    Rgb {
        r: 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    }
}

/// Max saturation for a hue that still fits in sRGB (`S = C/L`).
/// `a` and `b` must be normalized (`a^2 + b^2 == 1`).
pub fn compute_max_saturation(a: f32, b: f32) -> f32 {
    let (k0, k1, k2, k3, k4, wl, wm, ws);

    if -1.88170328 * a - 0.80936493 * b > 1.0 {
        // Red
        k0 = 1.19086277;
        k1 = 1.76576728;
        k2 = 0.59662641;
        k3 = 0.75515197;
        k4 = 0.56771245;
        wl = 4.0767416621;
        wm = -3.3077115913;
        ws = 0.2309699292;
    } else if 1.81444104 * a - 1.19445276 * b > 1.0 {
        // Green
        k0 = 0.73956515;
        k1 = -0.45954404;
        k2 = 0.08285427;
        k3 = 0.12541070;
        k4 = 0.14503204;
        wl = -1.2684380046;
        wm = 2.6097574011;
        ws = -0.3413193965;
    } else {
        // Blue
        k0 = 1.35733652;
        k1 = -0.00915799;
        k2 = -1.15130210;
        k3 = -0.50559606;
        k4 = 0.00692167;
        wl = -0.0041960863;
        wm = -0.7034186147;
        ws = 1.7076147010;
    }

    let mut s = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

    let k_l = 0.3963377774 * a + 0.2158037573 * b;
    let k_m = -0.1055613458 * a - 0.0638541728 * b;
    let k_s = -0.0894841775 * a - 1.2914855480 * b;

    // One Halley step, as in the reference implementation.
    {
        let l_ = 1.0 + s * k_l;
        let m_ = 1.0 + s * k_m;
        let s_ = 1.0 + s * k_s;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s_c = s_ * s_ * s_;

        let l_d_s = 3.0 * k_l * l_ * l_;
        let m_d_s = 3.0 * k_m * m_ * m_;
        let s_d_s = 3.0 * k_s * s_ * s_;

        let l_d_s2 = 6.0 * k_l * k_l * l_;
        let m_d_s2 = 6.0 * k_m * k_m * m_;
        let s_d_s2 = 6.0 * k_s * k_s * s_;

        let f = wl * l + wm * m + ws * s_c;
        let f1 = wl * l_d_s + wm * m_d_s + ws * s_d_s;
        let f2 = wl * l_d_s2 + wm * m_d_s2 + ws * s_d_s2;

        s -= f * f1 / (f1 * f1 - 0.5 * f * f2);
    }

    s
}

pub fn find_cusp(a: f32, b: f32) -> Lc {
    let s_cusp = compute_max_saturation(a, b);

    let rgb_at_max = oklab_to_linear_srgb(Lab { l: 1.0, a: s_cusp * a, b: s_cusp * b });
    let l_cusp = (1.0 / rgb_at_max.r.max(rgb_at_max.g).max(rgb_at_max.b)).cbrt();
    let c_cusp = l_cusp * s_cusp;

    Lc { l: l_cusp, c: c_cusp }
}

/// Intersection of the line `L = L0*(1-t) + t*L1`, `C = t*C1` with the gamut.
pub fn find_gamut_intersection_cusp(
    a: f32,
    b: f32,
    l1: f32,
    c1: f32,
    l0: f32,
    cusp: Lc,
) -> f32 {
    let mut t;

    if (l1 - l0) * cusp.c - (cusp.l - l0) * c1 <= 0.0 {
        // Lower half.
        t = cusp.c * l0 / (c1 * cusp.l + cusp.c * (l0 - l1));
    } else {
        // Upper half: intersect with the triangle first, then one Halley step.
        t = cusp.c * (l0 - 1.0) / (c1 * (cusp.l - 1.0) + cusp.c * (l0 - l1));

        let d_l = l1 - l0;
        let d_c = c1;

        let k_l = 0.3963377774 * a + 0.2158037573 * b;
        let k_m = -0.1055613458 * a - 0.0638541728 * b;
        let k_s = -0.0894841775 * a - 1.2914855480 * b;

        let l_dt = d_l + d_c * k_l;
        let m_dt = d_l + d_c * k_m;
        let s_dt = d_l + d_c * k_s;

        let l = l0 * (1.0 - t) + t * l1;
        let c = t * c1;

        let l_ = l + c * k_l;
        let m_ = l + c * k_m;
        let s_ = l + c * k_s;

        let l3 = l_ * l_ * l_;
        let m3 = m_ * m_ * m_;
        let s3 = s_ * s_ * s_;

        let ldt = 3.0 * l_dt * l_ * l_;
        let mdt = 3.0 * m_dt * m_ * m_;
        let sdt = 3.0 * s_dt * s_ * s_;

        let ldt2 = 6.0 * l_dt * l_dt * l_;
        let mdt2 = 6.0 * m_dt * m_dt * m_;
        let sdt2 = 6.0 * s_dt * s_dt * s_;

        let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3 - 1.0;
        let r1 = 4.0767416621 * ldt - 3.3077115913 * mdt + 0.2309699292 * sdt;
        let r2 = 4.0767416621 * ldt2 - 3.3077115913 * mdt2 + 0.2309699292 * sdt2;

        let u_r = r1 / (r1 * r1 - 0.5 * r * r2);
        let mut t_r = -r * u_r;

        let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3 - 1.0;
        let g1 = -1.2684380046 * ldt + 2.6097574011 * mdt - 0.3413193965 * sdt;
        let g2 = -1.2684380046 * ldt2 + 2.6097574011 * mdt2 - 0.3413193965 * sdt2;

        let u_g = g1 / (g1 * g1 - 0.5 * g * g2);
        let mut t_g = -g * u_g;

        // The C++ code reuses `b` for the blue channel here; the hue is not used
        // afterwards, so a separate name is equivalent.
        let b_ch = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3 - 1.0;
        let b1 = -0.0041960863 * ldt - 0.7034186147 * mdt + 1.7076147010 * sdt;
        let b2 = -0.0041960863 * ldt2 - 0.7034186147 * mdt2 + 1.7076147010 * sdt2;

        let u_b = b1 / (b1 * b1 - 0.5 * b_ch * b2);
        let mut t_b = -b_ch * u_b;

        t_r = if u_r >= 0.0 { t_r } else { f32::MAX };
        t_g = if u_g >= 0.0 { t_g } else { f32::MAX };
        t_b = if u_b >= 0.0 { t_b } else { f32::MAX };

        t += t_r.min(t_g.min(t_b));
    }

    t
}

#[inline]
pub fn find_gamut_intersection(a: f32, b: f32, l1: f32, c1: f32, l0: f32) -> f32 {
    let cusp = find_cusp(a, b);
    find_gamut_intersection_cusp(a, b, l1, c1, l0, cusp)
}

#[inline]
pub fn toe(x: f32) -> f32 {
    const K1: f32 = 0.206;
    const K2: f32 = 0.03;
    let k3 = (1.0 + K1) / (1.0 + K2);
    0.5 * (k3 * x - K1 + ((k3 * x - K1) * (k3 * x - K1) + 4.0 * K2 * k3 * x).sqrt())
}

#[inline]
pub fn toe_inv(x: f32) -> f32 {
    const K1: f32 = 0.206;
    const K2: f32 = 0.03;
    let k3 = (1.0 + K1) / (1.0 + K2);
    (x * x + K1 * x) / (k3 * (x + K2))
}

#[inline]
pub fn to_st(cusp: Lc) -> St {
    St { s: cusp.c / cusp.l, t: cusp.c / (1.0 - cusp.l) }
}

/// Fitted approximation of the cusp location.
pub fn get_st_mid(a_: f32, b_: f32) -> St {
    let s = 0.11516993
        + 1.0
            / (7.44778970
                + 4.15901240 * b_
                + a_ * (-2.19557347
                    + 1.75198401 * b_
                    + a_ * (-2.13704948 - 10.02301043 * b_
                        + a_ * (-4.24894561 + 5.38770819 * b_ + 4.69891013 * a_))));

    let t = 0.11239642
        + 1.0
            / (1.61320320
                - 0.68124379 * b_
                + a_ * (0.40370612
                    + 0.90148123 * b_
                    + a_ * (-0.27087943 + 0.61223990 * b_
                        + a_ * (0.00299215 - 0.45399568 * b_ - 0.14661872 * a_))));

    St { s, t }
}

pub fn get_cs(l: f32, a_: f32, b_: f32) -> Cs {
    let cusp = find_cusp(a_, b_);

    let c_max = find_gamut_intersection_cusp(a_, b_, l, 1.0, l, cusp);
    let st_max = to_st(cusp);

    let k = c_max / (l * st_max.s).min((1.0 - l) * st_max.t);

    let c_mid;
    {
        let st_mid = get_st_mid(a_, b_);
        let c_a = l * st_mid.s;
        let c_b = (1.0 - l) * st_mid.t;
        c_mid = 0.9
            * k
            * (1.0 / (1.0 / (c_a * c_a * c_a * c_a) + 1.0 / (c_b * c_b * c_b * c_b)))
                .sqrt()
                .sqrt();
    }

    let c_0;
    {
        let c_a = l * 0.4;
        let c_b = (1.0 - l) * 0.8;
        c_0 = (1.0 / (1.0 / (c_a * c_a) + 1.0 / (c_b * c_b))).sqrt();
    }

    Cs { c_0, c_mid, c_max }
}

pub fn okhsl_to_srgb(hsl: Hsl) -> Rgb {
    let (h, s, l) = (hsl.h, hsl.s, hsl.l);

    if l == 1.0 {
        return Rgb { r: 1.0, g: 1.0, b: 1.0 };
    } else if l == 0.0 {
        return Rgb { r: 0.0, g: 0.0, b: 0.0 };
    }

    let a_ = (2.0 * PI * h).cos();
    let b_ = (2.0 * PI * h).sin();
    let l_ok = toe_inv(l);

    let cs = get_cs(l_ok, a_, b_);

    let mid = 0.8;
    let mid_inv = 1.25;

    let c;
    if s < mid {
        let t = mid_inv * s;
        let k_1 = mid * cs.c_0;
        let k_2 = 1.0 - k_1 / cs.c_mid;
        c = t * k_1 / (1.0 - k_2 * t);
    } else {
        let t = (s - mid) / (1.0 - mid);
        let k_0 = cs.c_mid;
        let k_1 = (1.0 - mid) * cs.c_mid * cs.c_mid * mid_inv * mid_inv / cs.c_0;
        let k_2 = 1.0 - k_1 / (cs.c_max - cs.c_mid);
        c = k_0 + t * k_1 / (1.0 - k_2 * t);
    }

    let rgb = oklab_to_linear_srgb(Lab { l: l_ok, a: c * a_, b: c * b_ });
    Rgb {
        r: srgb_transfer_function(rgb.r),
        g: srgb_transfer_function(rgb.g),
        b: srgb_transfer_function(rgb.b),
    }
}

pub fn srgb_to_okhsl(rgb: Rgb) -> Hsl {
    // The engine's patch: pure black has no hue, avoid the singularity.
    if rgb.r == 0.0 && rgb.g == 0.0 && rgb.b == 0.0 {
        return Hsl { h: 0.0, s: 0.0, l: 0.0 };
    }

    let lab = linear_srgb_to_oklab(Rgb {
        r: srgb_transfer_function_inv(rgb.r),
        g: srgb_transfer_function_inv(rgb.g),
        b: srgb_transfer_function_inv(rgb.b),
    });

    let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    let l = lab.l;
    // Hue in [0, 1): the engine divides by 2*pi and wraps negatives.
    let mut h = lab.b.atan2(lab.a) / (2.0 * PI);
    if h < 0.0 {
        h += 1.0;
    }
    let l_ = toe(l);

    let cs = get_cs(l, a_, b_);

    let mid = 0.8;
    let mid_inv = 1.25;

    let s;
    if c < cs.c_mid {
        let k_1 = mid * cs.c_0;
        let k_2 = 1.0 - k_1 / cs.c_mid;
        let t = c / (k_1 + k_2 * c);
        s = t * mid;
    } else {
        let k_0 = cs.c_mid;
        let k_1 = (1.0 - mid) * cs.c_mid * cs.c_mid * mid_inv * mid_inv / cs.c_0;
        let k_2 = 1.0 - k_1 / (cs.c_max - cs.c_mid);
        let t = (c - k_0) / (k_1 + k_2 * (c - k_0));
        s = mid + (1.0 - mid) * t;
    }

    Hsl { h, s, l: l_ }
}

/// Okhsv as well, since `Color::set_ok_hsv` needs it.
pub fn okhsv_to_srgb(hsv: Hsv) -> Rgb {
    let (h, s, v) = (hsv.h, hsv.s, hsv.v);

    let a_ = (2.0 * PI * h).cos();
    let b_ = (2.0 * PI * h).sin();

    let st_max = to_st(find_cusp(a_, b_));

    let s_max = st_max.s;
    let t_max = st_max.t;
    let s_0 = 0.5;
    let k = 1.0 - s_0 / s_max;

    let l_v = 1.0 - s * s_0 / (s_0 + t_max - t_max * k * s);
    let c_v = s * t_max * s_0 / (s_0 + t_max - t_max * k * s);

    let l = v * l_v;
    let c = v * c_v;

    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;

    let l_new = toe_inv(l);
    let c_new = c * l_new / l;
    let l2 = l_new;
    let c2 = c_new;

    let rgb_scale = oklab_to_linear_srgb(Lab { l: l_vt, a: a_ * c_vt, b: b_ * c_vt });
    let scale_l = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b).max(0.0)).cbrt();

    let l3 = l2 * scale_l;
    let c3 = c2 * scale_l;

    let rgb = oklab_to_linear_srgb(Lab { l: l3, a: a_ * c3, b: b_ * c3 });
    Rgb {
        r: srgb_transfer_function(rgb.r),
        g: srgb_transfer_function(rgb.g),
        b: srgb_transfer_function(rgb.b),
    }
}

pub fn srgb_to_okhsv(rgb: Rgb) -> Hsv {
    let lab = linear_srgb_to_oklab(Rgb {
        r: srgb_transfer_function_inv(rgb.r),
        g: srgb_transfer_function_inv(rgb.g),
        b: srgb_transfer_function_inv(rgb.b),
    });

    let mut c = (lab.a * lab.a + lab.b * lab.b).sqrt();
    let a_ = lab.a / c;
    let b_ = lab.b / c;

    let mut l = lab.l;
    // Okhsv's hue is measured on (-a, -b), unlike Okhsl.
    let h = 0.5 + 0.5 * (-lab.b).atan2(-lab.a) / PI;

    let cusp = find_cusp(a_, b_);
    let st_max = to_st(cusp);
    let s_max = st_max.s;
    let t_max = st_max.t;
    let s_0 = 0.5;
    let k = 1.0 - s_0 / s_max;

    // First find L_v and C_v.
    let t = t_max / (c + l * t_max);
    let l_v = t * l;
    let c_v = t * c;

    let l_vt = toe_inv(l_v);
    let c_vt = c_v * l_vt / l_v;

    // Invert the toe and the curved top of the triangle.
    let rgb_scale = oklab_to_linear_srgb(Lab { l: l_vt, a: a_ * c_vt, b: b_ * c_vt });
    let scale_l = (1.0 / rgb_scale.r.max(rgb_scale.g).max(rgb_scale.b.max(0.0))).cbrt();

    l /= scale_l;
    c /= scale_l;

    c = c * toe(l) / l;
    l = toe(l);

    let v = l / l_v;
    let s = (s_0 + t_max) * c_v / ((t_max * s_0) + t_max * k * c_v);

    Hsv { h, s, v }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_okhsl_values() {
        // Pure red in sRGB is hue 0, full saturation and lightness ~0.568 in Okhsl.
        let c = okhsl_to_srgb(Hsl { h: 0.0, s: 1.0, l: 0.5 });
        assert!(c.r > 0.5 && c.g < 0.5 && c.b < 0.5, "{c:?}");
    }

    #[test]
    fn black_and_white_shortcut() {
        let w = okhsl_to_srgb(Hsl { h: 0.3, s: 0.5, l: 1.0 });
        assert_eq!(w, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        let k = okhsl_to_srgb(Hsl { h: 0.3, s: 0.5, l: 0.0 });
        assert_eq!(k, Rgb { r: 0.0, g: 0.0, b: 0.0 });
        assert_eq!(srgb_to_okhsl(Rgb { r: 0.0, g: 0.0, b: 0.0 }), Hsl { h: 0.0, s: 0.0, l: 0.0 });
    }

    #[test]
    fn toe_roundtrip() {
        for x in [0.0f32, 0.1, 0.5, 0.9, 1.0] {
            let back = toe_inv(toe(x));
            assert!((back - x).abs() < 1e-4, "{x} -> {back}");
        }
    }
}
