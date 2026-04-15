//! Biquad filter implementations based on the Robert Bristow-Johnson cookbook.
//!
//! Supports low-pass, high-pass, band-pass, and notch filter types using a
//! Direct Form II transposed biquad structure.

use mulberry_core::Sample;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Supported biquad filter types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

/// A second-order IIR (biquad) filter.
///
/// Coefficients are computed from the [Audio EQ Cookbook][cookbook] by Robert
/// Bristow-Johnson and applied via Direct Form II.
///
/// [cookbook]: https://www.w3.org/2011/audio/audio-eq-cookbook.html
pub struct BiquadFilter {
    filter_type: FilterType,
    sample_rate: f32,
    frequency: f32,
    q: f32,

    // Normalised coefficients (a0 already divided out).
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,

    // Filter state (previous input/output samples).
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    /// Creates a new biquad filter and computes its coefficients.
    ///
    /// * `filter_type` – the filter shape.
    /// * `frequency`   – cutoff or centre frequency in Hz.
    /// * `q`           – quality factor (`0.707` ≈ Butterworth is a common choice).
    /// * `sample_rate` – output sample rate in Hz.
    pub fn new(filter_type: FilterType, frequency: f32, q: f32, sample_rate: f32) -> Self {
        let mut f = Self {
            filter_type,
            sample_rate,
            frequency,
            q,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        f.compute_coefficients();
        f
    }

    /// Updates the cutoff / centre frequency and recomputes coefficients.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
        self.compute_coefficients();
    }

    /// Updates the quality factor and recomputes coefficients.
    pub fn set_q(&mut self, q: f32) {
        self.q = q;
        self.compute_coefficients();
    }

    /// Processes a single input sample and returns the filtered output.
    #[inline]
    pub fn process_sample(&mut self, input: Sample) -> Sample {
        let output =
            self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                - self.a1 * self.y1
                - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    /// Filters the buffer in-place.
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    /// Clears the internal delay-line state.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Computes biquad coefficients using the Bristow-Johnson cookbook formulas.
    fn compute_coefficients(&mut self) {
        let omega = 2.0 * PI * self.frequency / self.sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * self.q);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            FilterType::LowPass => {
                let b0 = (1.0 - cos_omega) / 2.0;
                let b1 = 1.0 - cos_omega;
                let b2 = (1.0 - cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass => {
                let b0 = (1.0 + cos_omega) / 2.0;
                let b1 = -(1.0 + cos_omega);
                let b2 = (1.0 + cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalise by a0.
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowpass_attenuates_high_freq() {
        // 100 Hz LowPass should barely attenuate a 50 Hz sine but strongly
        // attenuate a 10 kHz sine.
        let sr = 44100.0;
        let mut lp = BiquadFilter::new(FilterType::LowPass, 100.0, 0.707, sr);

        // Feed a few hundred samples of a high-frequency sine.
        let mut energy = 0.0_f32;
        for i in 0..1024 {
            let input = (2.0 * PI * 10_000.0 * i as f32 / sr).sin();
            let out = lp.process_sample(input);
            energy += out * out;
        }
        let rms_high = (energy / 1024.0).sqrt();

        lp.reset();
        energy = 0.0;
        for i in 0..1024 {
            let input = (2.0 * PI * 50.0 * i as f32 / sr).sin();
            let out = lp.process_sample(input);
            energy += out * out;
        }
        let rms_low = (energy / 1024.0).sqrt();

        assert!(
            rms_low > rms_high * 5.0,
            "low-freq RMS ({rms_low}) should be much larger than high-freq RMS ({rms_high})"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut f = BiquadFilter::new(FilterType::HighPass, 1000.0, 0.707, 44100.0);
        f.process_sample(1.0);
        f.reset();
        // After reset with zero input the output should be zero.
        assert_eq!(f.process_sample(0.0), 0.0);
    }
}
