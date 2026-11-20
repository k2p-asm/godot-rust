//! `smbFft` from `servers/audio/effects/audio_effect_spectrum_analyzer.cpp`.
//!
//! Bernsee's in-place radix-2 FFT over an interleaved complex buffer
//! (`[re0, im0, re1, im1, ...]`, `2 * frame_size` floats). `sign = -1` is the
//! forward transform, `+1` the inverse. The bit-reversal loop and the butterfly
//! are reproduced statement for statement, so the output matches the engine bit
//! for bit given the same input.
//!
//! The frame size must be a power of two; non-powers are processed with the
//! floor of the log2 (as the engine's `(long)(log(N)/log(2) + 0.5)` does).

/// `smbFft`: in-place transform of `buffer[0..2 * frame_size]`.
pub fn smb_fft(buffer: &mut [f32], frame_size: usize, sign: i32) {
    let n = frame_size;
    if n == 0 || buffer.len() < 2 * n {
        return;
    }
    let log_n = ((n as f64).ln() / 2f64.ln() + 0.5) as i64;

    // Bit reversal.
    let mut i = 2usize;
    while i < 2 * n - 2 {
        let mut j = 0usize;
        let mut bitm = 2usize;
        while bitm < 2 * n {
            if i & bitm != 0 {
                j += 1;
            }
            j <<= 1;
            bitm <<= 1;
        }
        if i < j {
            buffer.swap(i, j);
            buffer.swap(i + 1, j + 1);
        }
        i += 2;
    }

    let mut le = 2usize;
    let mut k = 0i64;
    while k < log_n {
        le <<= 1;
        let le2 = le >> 1;
        let mut ur = 1.0f32;
        let mut ui = 0.0f32;
        let arg = std::f32::consts::PI / ((le2 >> 1) as f32);
        let wr = arg.cos();
        let wi = sign as f32 * arg.sin();
        let mut j = 0usize;
        while j < le2 {
            let mut p1r = j;
            let mut p1i = j + 1;
            let mut p2r = j + le2;
            let mut p2i = j + le2 + 1;
            let mut i = j;
            while i < 2 * n {
                let tr = buffer[p2r] * ur - buffer[p2i] * ui;
                let ti = buffer[p2r] * ui + buffer[p2i] * ur;
                buffer[p2r] = buffer[p1r] - tr;
                buffer[p2i] = buffer[p1i] - ti;
                buffer[p1r] += tr;
                buffer[p1i] += ti;
                p1r += le;
                p1i += le;
                p2r += le;
                p2i += le;
                i += le;
            }
            let tr = ur * wr - ui * wi;
            ui = ur * wi + ui * wr;
            ur = tr;
            j += 2;
        }
        k += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive DFT with the `e^{-2i pi k n / N}` convention.
    fn naive_dft(input: &[f32]) -> Vec<(f64, f64)> {
        let n = input.len();
        (0..n)
            .map(|k| {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                for (index, value) in input.iter().enumerate() {
                    let angle = -2.0 * std::f64::consts::PI * k as f64 * index as f64
                        / n as f64;
                    re += *value as f64 * angle.cos();
                    im += *value as f64 * angle.sin();
                }
                (re, im)
            })
            .collect()
    }

    #[test]
    fn forward_matches_the_naive_dft() {
        for size in [2usize, 4, 8, 16, 32] {
            let input: Vec<f32> = (0..size)
                .map(|index| {
                    ((index as f32) * 0.7).sin() + 0.25 * (index as f32 * 1.3).cos()
                })
                .collect();
            let expected = naive_dft(&input);
            let mut buffer = vec![0.0f32; size * 2];
            for (index, value) in input.iter().enumerate() {
                buffer[index * 2] = *value;
                buffer[index * 2 + 1] = 0.0;
            }
            smb_fft(&mut buffer, size, -1);
            for k in 0..size {
                let re = buffer[k * 2] as f64;
                let im = buffer[k * 2 + 1] as f64;
                assert!(
                    (re - expected[k].0).abs() < 1e-2,
                    "size {size} bin {k} re {re} vs {}",
                    expected[k].0
                );
                assert!(
                    (im - expected[k].1).abs() < 1e-2,
                    "size {size} bin {k} im {im} vs {}",
                    expected[k].1
                );
            }
        }
    }

    #[test]
    fn impulse_and_dc_are_exact() {
        // An impulse at sample 0 transforms to all-ones.
        let mut buffer = vec![0.0f32; 16];
        buffer[0] = 1.0;
        smb_fft(&mut buffer, 8, -1);
        for index in 0..8 {
            assert!((buffer[index * 2] - 1.0).abs() < 1e-5);
            assert!(buffer[index * 2 + 1].abs() < 1e-5);
        }

        // A constant signal transforms to N at bin 0 and zero elsewhere.
        let mut buffer = vec![0.0f32; 16];
        for index in 0..8 {
            buffer[index * 2] = 1.0;
        }
        smb_fft(&mut buffer, 8, -1);
        assert!((buffer[0] - 8.0).abs() < 1e-5);
        assert!(buffer[1].abs() < 1e-5);
        for index in 1..8 {
            assert!(buffer[index * 2].abs() < 1e-5);
            assert!(buffer[index * 2 + 1].abs() < 1e-5);
        }
    }

    #[test]
    fn inverse_undoes_the_forward_transform() {
        let size = 32usize;
        let input: Vec<f32> = (0..size).map(|index| (index as f32 * 0.37).sin()).collect();
        let mut buffer = vec![0.0f32; size * 2];
        for (index, value) in input.iter().enumerate() {
            buffer[index * 2] = *value;
        }
        smb_fft(&mut buffer, size, -1);
        smb_fft(&mut buffer, size, 1);
        for (index, value) in input.iter().enumerate() {
            let restored = buffer[index * 2] / size as f32;
            assert!(
                (restored - value).abs() < 1e-4,
                "sample {index}: {restored} vs {value}"
            );
        }
    }
}
