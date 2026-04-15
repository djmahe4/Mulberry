//! # Mulberry DSP
//!
//! Digital Signal Processing primitives for the Mulberry DAW.
//!
//! Provides:
//! - Oscillators (sine, square, sawtooth, triangle, noise)
//! - ADSR envelope generator
//! - Biquad filters (low-pass, high-pass, band-pass, notch)
//! - Gain/mixer utilities

pub mod envelope;
pub mod filter;
pub mod mixer;
pub mod oscillator;

pub use envelope::AdsrEnvelope;
pub use filter::{BiquadFilter, FilterType};
pub use mixer::Mixer;
pub use oscillator::{Oscillator, Waveform};
