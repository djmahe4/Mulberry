//! Oscillator module providing common waveform generators.
//!
//! Supports sine, square, sawtooth, triangle, and white-noise waveforms via a
//! phase-accumulator design that keeps the oscillator state in a single `f32`
//! ranging from `0.0` to `1.0`.

use mulberry_core::Sample;
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Supported waveform shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    WhiteNoise,
}

/// A single-voice oscillator that produces audio samples for a given
/// [`Waveform`] at a specified frequency.
///
// CONTEXT7 REVIEW: Phase-accumulator design
// We store phase as a normalised `f32` in `[0, 1)` and advance it by
// `frequency / sample_rate` each sample. This avoids precision loss that
// would occur if we accumulated radians (which grow without bound) and
// keeps waveform calculations branch-free except for the waveform match.
pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    /// Normalised phase in `[0.0, 1.0)`.
    phase: f32,
    sample_rate: f32,
    amplitude: f32,
    /// Internal PRNG state for white-noise generation (simple LCG).
    noise_state: u32,
}

impl Oscillator {
    /// Creates a new oscillator.
    ///
    /// * `waveform`    – the shape of the wave to generate.
    /// * `frequency`   – oscillation frequency in Hz.
    /// * `sample_rate` – output sample rate in Hz.
    pub fn new(waveform: Waveform, frequency: f32, sample_rate: f32) -> Self {
        Self {
            waveform,
            frequency,
            phase: 0.0,
            sample_rate,
            amplitude: 1.0,
            noise_state: 12345,
        }
    }

    /// Sets the oscillation frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    /// Sets the waveform shape.
    pub fn set_waveform(&mut self, wf: Waveform) {
        self.waveform = wf;
    }

    /// Sets the output amplitude (clamped to `0.0..=1.0`).
    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp.clamp(0.0, 1.0);
    }

    /// Generates the next sample and advances the phase accumulator.
    pub fn next_sample(&mut self) -> Sample {
        let raw = match self.waveform {
            Waveform::Sine => (self.phase * TAU).sin(),
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => 2.0 * self.phase - 1.0,
            Waveform::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            Waveform::WhiteNoise => {
                // Minimal LCG PRNG – no external rand crate needed.
                self.noise_state = self
                    .noise_state
                    .wrapping_mul(1664525)
                    .wrapping_add(1013904223);
                // Map u32 to [-1.0, 1.0]
                (self.noise_state as f32 / u32::MAX as f32) * 2.0 - 1.0
            }
        };

        // Advance phase accumulator.
        self.phase += self.frequency / self.sample_rate;
        self.phase %= 1.0;

        raw * self.amplitude
    }

    /// Fills `buffer` with consecutive samples.
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Resets the phase accumulator to zero.
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_starts_at_zero() {
        let mut osc = Oscillator::new(Waveform::Sine, 440.0, 44100.0);
        let s = osc.next_sample();
        assert!(s.abs() < 1e-6, "sine should start near 0, got {s}");
    }

    #[test]
    fn square_first_half_positive() {
        let mut osc = Oscillator::new(Waveform::Square, 1.0, 4.0);
        assert_eq!(osc.next_sample(), 1.0);
        assert_eq!(osc.next_sample(), 1.0);
        assert_eq!(osc.next_sample(), -1.0);
    }

    #[test]
    fn fill_buffer_length() {
        let mut osc = Oscillator::new(Waveform::Sawtooth, 440.0, 44100.0);
        let mut buf = vec![0.0; 256];
        osc.fill_buffer(&mut buf);
        assert!(buf.iter().all(|&s| (-1.0..=1.0).contains(&s)));
    }

    #[test]
    fn amplitude_scaling() {
        let mut osc = Oscillator::new(Waveform::Square, 1.0, 4.0);
        osc.set_amplitude(0.5);
        assert!((osc.next_sample() - 0.5).abs() < 1e-6);
    }
}
