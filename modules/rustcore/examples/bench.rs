//! Same three loops as `parity/bench.cpp`, in the Rust port.
//! Run with: `cargo run --release --example bench`

use godot_core_rs::math::basis::{Basis, EulerOrder};
use godot_core_rs::math::transform_3d::Transform3D;
use godot_core_rs::math::vector3::Vector3;
use std::time::Instant;

const N: usize = 2 * 1000 * 1000;

fn main() {
    let mut pts = Vec::with_capacity(N);
    let mut state: u32 = 0x1234_5678;
    let mut next = move || -> f32 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        (state & 0xFFFF) as f32 / 65535.0 - 0.5
    };
    for _ in 0..N {
        pts.push(Vector3::new(next() * 100.0, next() * 100.0, next() * 100.0));
    }

    let basis = Basis::from_euler(Vector3::new(0.3, -0.7, 1.1), EulerOrder::YXZ);
    let xform = Transform3D::new(basis, Vector3::new(1.0, 2.0, 3.0));

    let t0 = Instant::now();
    let mut acc_dot = 0.0f32;
    for i in 0..N {
        acc_dot += pts[i].dot(pts[(i + 1) % N]);
    }
    let d_dot = t0.elapsed().as_secs_f64() * 1e3;

    let t2 = Instant::now();
    let (mut acc_x, mut acc_y, mut acc_z) = (0.0f32, 0.0f32, 0.0f32);
    for p in pts.iter() {
        let v = xform.xform(*p);
        acc_x += v.x;
        acc_y += v.y;
        acc_z += v.z;
    }
    let d_xform = t2.elapsed().as_secs_f64() * 1e3;

    let t4 = Instant::now();
    let mut acc_len = 0.0f32;
    for p in pts.iter() {
        acc_len += p.normalized().length();
    }
    let d_norm = t4.elapsed().as_secs_f64() * 1e3;

    println!("rust dot        {:8.2} ms  ({:.3} ns/op)  checksum {:.6}", d_dot, d_dot * 1e6 / N as f64, acc_dot);
    println!("rust xform      {:8.2} ms  ({:.3} ns/op)  checksum {:.6}", d_xform, d_xform * 1e6 / N as f64, acc_x + acc_y + acc_z);
    println!("rust normalize  {:8.2} ms  ({:.3} ns/op)  checksum {:.6}", d_norm, d_norm * 1e6 / N as f64, acc_len);
}
