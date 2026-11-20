//! Port of `core/math/math_fieldwise.cpp` (`fieldwise_assign`): copies one
//! named field from `source` to `target` when both have the same type.
//!
//! The engine's version only exists under `DEBUG_ENABLED`, so it cannot be
//! exercised by the parity harness (which builds with `-O2` and no DEBUG flags).
//! These tests are therefore **non-differential**: they pin the field tables
//! transcribed from `math_fieldwise.cpp`.
//!
//! Behaviour mirrored exactly:
//! - different types -> `target` unchanged;
//! - unknown field -> `target` unchanged;
//! - unsupported types (String, Array, ...) -> `target` unchanged.

use crate::variant::{Variant, VariantType};

/// `fieldwise_assign(target, source, field)`.
pub fn fieldwise_assign(target: &Variant, source: &Variant, field: &str) -> Variant {
    if target.vtype() != source.vtype() {
        return target.clone();
    }

    match source {
        Variant::Vector2(s) => {
            let mut t = target.clone();
            if let Variant::Vector2(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    _ => {}
                }
            }
            t
        }
        Variant::Vector2i(s) => {
            let mut t = target.clone();
            if let Variant::Vector2i(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    _ => {}
                }
            }
            t
        }
        Variant::Vector3(s) => {
            let mut t = target.clone();
            if let Variant::Vector3(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    "z" => v.z = s.z,
                    _ => {}
                }
            }
            t
        }
        Variant::Vector3i(s) => {
            let mut t = target.clone();
            if let Variant::Vector3i(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    "z" => v.z = s.z,
                    _ => {}
                }
            }
            t
        }
        Variant::Vector4(s) => {
            let mut t = target.clone();
            if let Variant::Vector4(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    "z" => v.z = s.z,
                    "w" => v.w = s.w,
                    _ => {}
                }
            }
            t
        }
        Variant::Vector4i(s) => {
            let mut t = target.clone();
            if let Variant::Vector4i(v) = &mut t {
                match field {
                    "x" => v.x = s.x,
                    "y" => v.y = s.y,
                    "z" => v.z = s.z,
                    "w" => v.w = s.w,
                    _ => {}
                }
            }
            t
        }
        Variant::Rect2(s) => {
            let mut t = target.clone();
            if let Variant::Rect2(r) = &mut t {
                match field {
                    "x" => r.position.x = s.position.x,
                    "y" => r.position.y = s.position.y,
                    "w" => r.size.x = s.size.x,
                    "h" => r.size.y = s.size.y,
                    _ => {}
                }
            }
            t
        }
        Variant::Rect2i(s) => {
            let mut t = target.clone();
            if let Variant::Rect2i(r) = &mut t {
                match field {
                    "x" => r.position.x = s.position.x,
                    "y" => r.position.y = s.position.y,
                    "w" => r.size.x = s.size.x,
                    "h" => r.size.y = s.size.y,
                    _ => {}
                }
            }
            t
        }
        Variant::Plane(s) => {
            let mut t = target.clone();
            if let Variant::Plane(p) = &mut t {
                match field {
                    "x" => p.normal.x = s.normal.x,
                    "y" => p.normal.y = s.normal.y,
                    "z" => p.normal.z = s.normal.z,
                    "d" => p.d = s.d,
                    _ => {}
                }
            }
            t
        }
        Variant::Quaternion(s) => {
            let mut t = target.clone();
            if let Variant::Quaternion(q) = &mut t {
                match field {
                    "x" => q.x = s.x,
                    "y" => q.y = s.y,
                    "z" => q.z = s.z,
                    "w" => q.w = s.w,
                    _ => {}
                }
            }
            t
        }
        Variant::Aabb(s) => {
            let mut t = target.clone();
            if let Variant::Aabb(a) = &mut t {
                match field {
                    "px" => a.position.x = s.position.x,
                    "py" => a.position.y = s.position.y,
                    "pz" => a.position.z = s.position.z,
                    "sx" => a.size.x = s.size.x,
                    "sy" => a.size.y = s.size.y,
                    "sz" => a.size.z = s.size.z,
                    _ => {}
                }
            }
            t
        }
        Variant::Basis(s) => {
            let mut t = target.clone();
            if let Variant::Basis(b) = &mut t {
                if let Some((r, c)) = parse_basis_field(field) {
                    b.rows[r][c] = s.rows[r][c];
                }
            }
            t
        }
        Variant::Transform2D(s) => {
            let mut t = target.clone();
            if let Variant::Transform2D(x) = &mut t {
                match field {
                    "xx" => x.columns[0].x = s.columns[0].x,
                    "xy" => x.columns[0].y = s.columns[0].y,
                    "yx" => x.columns[1].x = s.columns[1].x,
                    "yy" => x.columns[1].y = s.columns[1].y,
                    "ox" => x.columns[2].x = s.columns[2].x,
                    "oy" => x.columns[2].y = s.columns[2].y,
                    _ => {}
                }
            }
            t
        }
        Variant::Transform3D(s) => {
            let mut t = target.clone();
            if let Variant::Transform3D(x) = &mut t {
                if let Some((r, c)) = parse_basis_field(field) {
                    x.basis.rows[r][c] = s.basis.rows[r][c];
                } else {
                    match field {
                        "xo" => x.origin.x = s.origin.x,
                        "yo" => x.origin.y = s.origin.y,
                        "zo" => x.origin.z = s.origin.z,
                        _ => {}
                    }
                }
            }
            t
        }
        _ => target.clone(),
    }
}

/// Maps "xx"/"xy"/... to a `(row, column)` pair; `None` for other names.
fn parse_basis_field(field: &str) -> Option<(usize, usize)> {
    let bytes = field.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let row = match bytes[0] {
        b'x' => 0,
        b'y' => 1,
        b'z' => 2,
        _ => return None,
    };
    let col = match bytes[1] {
        b'x' => 0,
        b'y' => 1,
        b'z' => 2,
        _ => return None,
    };
    Some((row, col))
}

/// True when `fieldwise_assign` has a table entry for this variant type.
pub fn supports_fieldwise(vtype: VariantType) -> bool {
    matches!(
        vtype,
        VariantType::Vector2
            | VariantType::Vector2i
            | VariantType::Rect2
            | VariantType::Rect2i
            | VariantType::Vector3
            | VariantType::Vector3i
            | VariantType::Vector4
            | VariantType::Vector4i
            | VariantType::Plane
            | VariantType::Quaternion
            | VariantType::Aabb
            | VariantType::Basis
            | VariantType::Transform2D
            | VariantType::Transform3D
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::aabb::Aabb;
    use crate::math::basis::Basis;
    use crate::math::plane::Plane;
    use crate::math::quaternion::Quaternion;
    use crate::math::rect2::Rect2;
    use crate::math::transform_2d::Transform2D;
    use crate::math::transform_3d::Transform3D;
    use crate::math::vector2::Vector2;
    use crate::math::vector3::Vector3;
    use crate::math::vector4::Vector4;

    #[test]
    fn copies_only_the_named_field() {
        let target = Variant::Vector3(Vector3::new(1.0, 2.0, 3.0));
        let source = Variant::Vector3(Vector3::new(9.0, 9.0, 9.0));

        assert_eq!(
            fieldwise_assign(&target, &source, "y"),
            Variant::Vector3(Vector3::new(1.0, 9.0, 3.0))
        );
        // Unknown field leaves the target untouched.
        assert_eq!(fieldwise_assign(&target, &source, "nope"), target);
    }

    #[test]
    fn type_mismatch_is_a_no_op() {
        let target = Variant::Vector3(Vector3::ZERO);
        let source = Variant::Vector2(Vector2::ONE);
        assert_eq!(fieldwise_assign(&target, &source, "x"), target);
    }

    #[test]
    fn rect_uses_x_y_w_h() {
        let target = Variant::Rect2(Rect2::new_xywh(0.0, 0.0, 1.0, 1.0));
        let source = Variant::Rect2(Rect2::new_xywh(5.0, 6.0, 7.0, 8.0));
        assert_eq!(
            fieldwise_assign(&target, &source, "w"),
            Variant::Rect2(Rect2::new_xywh(0.0, 0.0, 7.0, 1.0))
        );
    }

    #[test]
    fn matrix_fields_and_origins() {
        let target = Variant::Transform3D(Transform3D::IDENTITY);
        let source = Variant::Transform3D(Transform3D::new(
            Basis::from_scale(Vector3::new(2.0, 3.0, 4.0)),
            Vector3::new(10.0, 20.0, 30.0),
        ));

        let after_zz = fieldwise_assign(&target, &source, "zz");
        assert_eq!(
            after_zz,
            Variant::Transform3D(Transform3D::new(
                Basis::from_scale(Vector3::new(1.0, 1.0, 4.0)),
                Vector3::ZERO
            ))
        );

        let after_yo = fieldwise_assign(&target, &source, "yo");
        assert_eq!(
            after_yo,
            Variant::Transform3D(Transform3D::new(Basis::IDENTITY, Vector3::new(0.0, 20.0, 0.0)))
        );
    }

    #[test]
    fn transform2d_columns() {
        let target = Variant::Transform2D(Transform2D::IDENTITY);
        let source = Variant::Transform2D(Transform2D::new([
            Vector2::new(2.0, 3.0),
            Vector2::new(4.0, 5.0),
            Vector2::new(6.0, 4.0),
        ]));
        let out = fieldwise_assign(&target, &source, "oy");
        assert_eq!(
            out,
            Variant::Transform2D(Transform2D::new([
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 4.0),
            ]))
        );
    }

    #[test]
    fn aabb_plane_quaternion_fields() {
        let a_target = Variant::Aabb(Aabb::new(Vector3::ZERO, Vector3::ONE));
        let a_source = Variant::Aabb(Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::splat(9.0)));
        assert_eq!(
            fieldwise_assign(&a_target, &a_source, "pz"),
            Variant::Aabb(Aabb::new(Vector3::new(0.0, 0.0, 3.0), Vector3::ONE))
        );

        let p_target = Variant::Plane(Plane::new(0.0, 1.0, 0.0, 2.0));
        let p_source = Variant::Plane(Plane::new(5.0, 5.0, 5.0, 9.0));
        assert_eq!(
            fieldwise_assign(&p_target, &p_source, "d"),
            Variant::Plane(Plane::new(0.0, 1.0, 0.0, 9.0))
        );

        let q_target = Variant::Quaternion(Quaternion::IDENTITY);
        let q_source = Variant::Quaternion(Quaternion::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(
            fieldwise_assign(&q_target, &q_source, "w"),
            Variant::Quaternion(Quaternion::new(0.0, 0.0, 0.0, 4.0))
        );
    }

    #[test]
    fn vector4_and_unsupported_types() {
        let target = Variant::Vector4(Vector4::ZERO);
        let source = Variant::Vector4(Vector4::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(
            fieldwise_assign(&target, &source, "z"),
            Variant::Vector4(Vector4::new(0.0, 0.0, 3.0, 0.0))
        );

        // Types without a table keep the target.
        let s_target = Variant::Int(1);
        let s_source = Variant::Int(2);
        assert_eq!(fieldwise_assign(&s_target, &s_source, "x"), s_target);
        assert!(!supports_fieldwise(VariantType::Int));
        assert!(supports_fieldwise(VariantType::Aabb));
    }
}
