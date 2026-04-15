//! Audio sample types and buffer abstractions.
//!
//! Provides the fundamental audio data types used throughout the Mulberry engine:
//! [`Sample`] (f32), [`AudioFrame`] (stereo pair), and [`AudioBuffer`] (a
//! timestamped sequence of frames).

use serde::{Deserialize, Serialize};

// ── Sample ──────────────────────────────────────────────────────────────────

/// A single audio sample represented as a 32-bit float.
///
/// All internal audio processing in Mulberry uses `f32` samples in the
/// range `[-1.0, 1.0]`.
// CONTEXT7 REVIEW: f32 chosen over f64 for cache-friendly real-time audio
// processing — matches the native format of most audio hardware / APIs.
pub type Sample = f32;

// ── AudioFrame ──────────────────────────────────────────────────────────────

/// A single stereo audio frame containing left and right channel samples.
///
/// This is the atomic unit of audio data flowing through the DSP graph.
// CONTEXT7 REVIEW: Fixed stereo layout keeps the hot path branch-free.
// Multi-channel support can be added later via a separate `MultiFrame` type.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioFrame {
    /// Left channel sample.
    pub left: Sample,
    /// Right channel sample.
    pub right: Sample,
}

impl AudioFrame {
    /// Returns a silent frame (both channels at zero).
    #[inline]
    pub fn silence() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
        }
    }

    /// Returns a mono frame where both channels carry the same value.
    #[inline]
    pub fn mono(val: Sample) -> Self {
        Self {
            left: val,
            right: val,
        }
    }

    /// Mixes another frame into this one by summing the samples per-channel.
    #[inline]
    pub fn mix(self, other: Self) -> Self {
        Self {
            left: self.left + other.left,
            right: self.right + other.right,
        }
    }
}

impl Default for AudioFrame {
    fn default() -> Self {
        Self::silence()
    }
}

// ── AudioBuffer ─────────────────────────────────────────────────────────────

/// A buffer of stereo audio frames at a known sample rate.
///
/// Used to pass variable-length audio data between processing stages.
// CONTEXT7 REVIEW: Storing `sample_rate` alongside the data ensures buffers
// are self-describing, which prevents sample-rate mismatch bugs at API
// boundaries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioBuffer {
    /// The sample rate this buffer was captured / generated at.
    pub sample_rate: u32,
    /// The audio frames stored in this buffer.
    pub frames: Vec<AudioFrame>,
}

impl AudioBuffer {
    /// Creates a new, empty `AudioBuffer` with the given sample rate and
    /// pre-allocated capacity.
    pub fn new(sample_rate: u32, capacity: usize) -> Self {
        Self {
            sample_rate,
            frames: Vec::with_capacity(capacity),
        }
    }

    /// Creates an `AudioBuffer` from an existing vector of frames.
    pub fn from_frames(sample_rate: u32, frames: Vec<AudioFrame>) -> Self {
        Self {
            sample_rate,
            frames,
        }
    }

    /// Returns the duration of this buffer in seconds.
    pub fn duration_secs(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.frames.len() as f64 / self.sample_rate as f64
    }

    /// Returns the number of frames in this buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the buffer contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Removes all frames from the buffer without releasing allocated memory.
    #[inline]
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}
