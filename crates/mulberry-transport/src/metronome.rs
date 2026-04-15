//! Metronome — audible click generator.
//!
//! The [`Metronome`] produces short sine-wave bursts at beat boundaries.
//! Downbeats use a higher pitch (1000 Hz) and regular beats use a lower
//! pitch (800 Hz), both with a quick exponential decay.

use std::f32::consts::PI;

// CONTEXT7 REVIEW:
// Issue: Click generation approach
// Resolution: Use a simple decaying sine oscillator rather than stored samples
// Why: Keeps the metronome dependency-free (no WAV files) and allows
//      real-time frequency / gain control. The ~20 ms click duration is
//      short enough to sound like a natural percussive click.

/// Duration of a single click in seconds.
const CLICK_DURATION_SECS: f32 = 0.02;

/// Frequency of a downbeat click in Hz.
const DOWNBEAT_FREQ: f32 = 1000.0;

/// Frequency of a regular beat click in Hz.
const BEAT_FREQ: f32 = 800.0;

/// Exponential decay rate — higher values produce a faster fade-out.
const DECAY_RATE: f32 = 150.0;

/// A metronome that produces audible clicks at beat boundaries.
///
/// Call [`trigger_click`](Metronome::trigger_click) at the start of each beat,
/// then pull samples one at a time via [`next_sample`](Metronome::next_sample).
pub struct Metronome {
    enabled: bool,
    gain: f32,
    sample_rate: u32,
    /// Total length of one click in samples.
    click_length: u32,
    /// Remaining samples in the current click (counts down to zero).
    click_samples_remaining: u32,
    /// Whether the current click is a downbeat.
    is_downbeat: bool,
}

impl Metronome {
    /// Creates a new [`Metronome`] at the given sample rate.
    ///
    /// The metronome starts **enabled** with unity gain.
    pub fn new(sample_rate: u32) -> Self {
        let click_length = (CLICK_DURATION_SECS * sample_rate as f32) as u32;
        Self {
            enabled: true,
            gain: 1.0,
            sample_rate,
            click_length,
            click_samples_remaining: 0,
            is_downbeat: false,
        }
    }

    /// Enables or disables the metronome.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.click_samples_remaining = 0;
        }
    }

    /// Sets the output gain (0.0 = silent, 1.0 = unity).
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    /// Triggers a new click. Pass `true` for a downbeat (higher pitch).
    pub fn trigger_click(&mut self, downbeat: bool) {
        if !self.enabled {
            return;
        }
        self.is_downbeat = downbeat;
        self.click_samples_remaining = self.click_length;
    }

    /// Generates the next audio sample for the metronome.
    ///
    /// Returns `0.0` when no click is active or the metronome is disabled.
    pub fn next_sample(&mut self) -> f32 {
        if self.click_samples_remaining == 0 || !self.enabled {
            return 0.0;
        }

        let elapsed = (self.click_length - self.click_samples_remaining) as f32;
        let t = elapsed / self.sample_rate as f32;

        let freq = if self.is_downbeat {
            DOWNBEAT_FREQ
        } else {
            BEAT_FREQ
        };

        let oscillator = (2.0 * PI * freq * t).sin();
        let envelope = (-DECAY_RATE * t).exp();
        let sample = oscillator * envelope * self.gain;

        self.click_samples_remaining -= 1;
        sample
    }

    /// Returns `true` if the metronome is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
