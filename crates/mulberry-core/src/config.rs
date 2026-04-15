//! Configuration types for the Mulberry DAW.
//!
//! [`MulberryConfig`] aggregates all top-level settings that control how the
//! various subsystems are initialized.

use serde::{Deserialize, Serialize};

/// Top-level configuration for the Mulberry DAW.
///
/// Sensible defaults are provided via the [`Default`] implementation so that
/// the engine can start with zero explicit configuration.
// CONTEXT7 REVIEW: All fields are public and the struct derives
// `Serialize`/`Deserialize` so it can be round-tripped through JSON/TOML
// config files. Adding a new subsystem setting should be a non-breaking
// addition thanks to `#[serde(default)]`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MulberryConfig {
    /// Audio sample rate in Hz.
    pub sample_rate: u32,
    /// Audio buffer size in frames.
    pub buffer_size: usize,
    /// Number of output channels.
    pub channels: u16,
    /// Initial tempo in BPM.
    pub initial_tempo: f64,
    /// Initial time signature as `(numerator, denominator)`.
    pub time_signature: (u8, u8),
    /// Optional HTTP endpoint for the AI agent backend.
    pub agent_endpoint: Option<String>,
    /// Whether text-to-speech narration is enabled.
    pub tts_enabled: bool,
    /// Directories to scan for external plugins.
    pub plugin_dirs: Vec<String>,
}

impl Default for MulberryConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 512,
            channels: 2,
            initial_tempo: 120.0,
            time_signature: (4, 4),
            agent_endpoint: None,
            tts_enabled: false,
            plugin_dirs: Vec::new(),
        }
    }
}
