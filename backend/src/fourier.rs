//! Fourier transform computation for contour representation.
//!
//! # Algorithm
//!
//! The algorithm treats the x and y coordinates as the real and imaginary parts
//! of complex numbers, performs a Fast Fourier Transform (FFT), and extracts
//! the frequency components. These components represent circles rotating at
//! different frequencies that, when combined, recreate the original shape.
use rustfft::{FftPlanner, num_complex::Complex64};
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug)]
pub struct Coefficient {
    pub real: f64,
    pub imag: f64,
    pub n: i32,
}

impl Coefficient {
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct EquationData {
    /// sorted by magnitude (largest first, DC component at index 0)
    pub coefficients: Vec<Coefficient>,

    pub period: f64,
}

pub fn compute_coefficients(
    points_x: Vec<f64>,
    points_y: Vec<f64>,
    max_coefficients: Option<usize>,
) -> Vec<Coefficient> {
    let n = points_x.len();
    assert_eq!(n, points_y.len(), "x and y arrays must have same length");

    let mut buffer: Vec<Complex64> = points_x
        .iter()
        .zip(points_y.iter())
        .map(|(&x, &y)| Complex64::new(x, y))
        .collect();

    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer);

    let mut coeffs: Vec<Coefficient> = buffer
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let freq_index = if i <= n / 2 {
                i as i32
            } else {
                i as i32 - n as i32
            };
            Coefficient {
                real: c.re / n as f64,
                imag: c.im / n as f64,
                n: freq_index,
            }
        })
        .collect();

    let a0_index = coeffs.iter().position(|c| c.n == 0).unwrap_or(0);
    let a0 = coeffs.remove(a0_index);

    coeffs.sort_by(|a, b| {
        let mag_a = (a.real * a.real + a.imag * a.imag).sqrt();
        let mag_b = (b.real * b.real + b.imag * b.imag).sqrt();
        mag_b
            .partial_cmp(&mag_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(max_k) = max_coefficients {
        coeffs.truncate((max_k.saturating_sub(1)).min(n - 1));
    }

    coeffs.insert(0, a0);

    coeffs
}
