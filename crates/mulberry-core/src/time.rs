//! Musical time representations.
//!
//! Types for describing tempo, time signatures, musical positions, and a
//! sample-accurate clock that maps audio samples to musical time.

use serde::{Deserialize, Serialize};

// ── Tempo ───────────────────────────────────────────────────────────────────

/// Represents a musical tempo in beats per minute (BPM).
// CONTEXT7 REVIEW: Tempo is stored as `f64` to support fractional BPM values
// (e.g. 128.5 BPM) which are common in electronic music production.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tempo {
    /// Beats per minute.
    pub bpm: f64,
}

impl Tempo {
    /// Creates a new [`Tempo`] from the given BPM value.
    pub fn new(bpm: f64) -> Self {
        Self { bpm }
    }

    /// Returns the number of audio samples that make up one beat at the given
    /// sample rate.
    pub fn samples_per_beat(&self, sample_rate: u32) -> f64 {
        if self.bpm == 0.0 {
            return 0.0;
        }
        (sample_rate as f64) * 60.0 / self.bpm
    }

    /// Returns the duration of a single beat in seconds.
    pub fn beat_duration_secs(&self) -> f64 {
        if self.bpm == 0.0 {
            return 0.0;
        }
        60.0 / self.bpm
    }
}

impl Default for Tempo {
    fn default() -> Self {
        Self { bpm: 120.0 }
    }
}

// ── TimeSignature ───────────────────────────────────────────────────────────

/// A musical time signature (e.g. 4/4, 3/4, 6/8).
// CONTEXT7 REVIEW: Using `u8` for numerator/denominator is sufficient — real
// time signatures rarely exceed 255 and this keeps the struct compact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    /// Number of beats per bar.
    pub numerator: u8,
    /// Note value that receives one beat (e.g. 4 = quarter note).
    pub denominator: u8,
}

impl TimeSignature {
    /// Creates a new [`TimeSignature`].
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns the number of beats in a single bar.
    #[inline]
    pub fn beats_per_bar(&self) -> u8 {
        self.numerator
    }
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

// ── MusicalPosition ─────────────────────────────────────────────────────────

/// A position in musical time expressed as a bar number and fractional beat
/// within that bar.
// CONTEXT7 REVIEW: `bar` is `u64` to support very long sessions without
// overflow. `beat` is `f64` for sub-beat precision.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MusicalPosition {
    /// Zero-based bar index.
    pub bar: u64,
    /// Fractional beat within the current bar (0.0 ≤ beat < beats_per_bar).
    pub beat: f64,
}

impl MusicalPosition {
    /// Returns the origin position (bar 0, beat 0.0).
    pub fn zero() -> Self {
        Self {
            bar: 0,
            beat: 0.0,
        }
    }

    /// Advances the position by the given number of audio samples.
    pub fn advance_by_samples(
        &mut self,
        num_samples: u64,
        tempo: &Tempo,
        time_sig: &TimeSignature,
        sample_rate: u32,
    ) {
        let samples_per_beat = tempo.samples_per_beat(sample_rate);
        if samples_per_beat == 0.0 {
            return;
        }
        let beats_to_add = num_samples as f64 / samples_per_beat;
        let beats_per_bar = time_sig.beats_per_bar() as f64;

        let mut total_beat = self.beat + beats_to_add;
        let extra_bars = (total_beat / beats_per_bar).floor() as u64;
        total_beat -= extra_bars as f64 * beats_per_bar;

        self.bar += extra_bars;
        self.beat = total_beat;
    }

    /// Returns the total number of beats from the origin, taking the time
    /// signature into account.
    pub fn total_beats(&self, time_sig: &TimeSignature) -> f64 {
        self.bar as f64 * time_sig.beats_per_bar() as f64 + self.beat
    }
}

impl Default for MusicalPosition {
    fn default() -> Self {
        Self::zero()
    }
}

// ── TickClock ───────────────────────────────────────────────────────────────

/// A sample-accurate clock that tracks playback position and converts it to
/// both wall-clock time and musical time.
// CONTEXT7 REVIEW: The clock is intentionally simple and stateless with
// respect to tempo — callers supply tempo/time-sig at query time so the
// clock stays correct across tempo changes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickClock {
    /// Current position in samples since the clock was started / reset.
    pub sample_position: u64,
    /// The sample rate used for time conversions.
    pub sample_rate: u32,
}

impl TickClock {
    /// Creates a new [`TickClock`] at position zero with the given sample rate.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_position: 0,
            sample_rate,
        }
    }

    /// Advances the clock by the given number of samples.
    #[inline]
    pub fn advance(&mut self, num_samples: u64) {
        self.sample_position += num_samples;
    }

    /// Resets the clock back to position zero.
    #[inline]
    pub fn reset(&mut self) {
        self.sample_position = 0;
    }

    /// Returns the current position in seconds.
    pub fn position_secs(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.sample_position as f64 / self.sample_rate as f64
    }

    /// Converts the current sample position into a [`MusicalPosition`].
    pub fn musical_position(&self, tempo: &Tempo, time_sig: &TimeSignature) -> MusicalPosition {
        let mut pos = MusicalPosition::zero();
        pos.advance_by_samples(self.sample_position, tempo, time_sig, self.sample_rate);
        pos
    }
}
