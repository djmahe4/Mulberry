//! Drum machine event types.

use serde::{Deserialize, Serialize};

/// A drum hit event emitted by the drum machine scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumHit {
    /// Track index (0-7)
    pub track: usize,
    /// Step index (0-15)
    pub step: usize,
    /// Velocity (0.0-1.0)
    pub velocity: f32,
    /// Whether this is an accented hit
    pub accent: bool,
}

/// Drum machine state for serialization / UI sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumMachineState {
    pub bpm: f32,
    pub current_step: usize,
    pub playing: bool,
    pub pattern_name: String,
}
