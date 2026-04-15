use serde::{Deserialize, Serialize};

/// Request for text-to-speech synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    /// Speed multiplier: 0.5 = half speed, 1.0 = normal, 2.0 = double.
    pub speed: f32,
    /// Pitch multiplier on base pitch.
    pub pitch: f32,
    pub sample_rate: u32,
}

/// Result of TTS synthesis.
#[derive(Debug, Clone)]
pub struct TtsResult {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_secs: f32,
}

/// Trait for TTS engines.
///
/// # CONTEXT7 REVIEW:
/// Issue: TTS synthesis can be CPU-intensive
/// Resolution: Use sync trait methods; callers wrap in `tokio::task::spawn_blocking`
/// Why: Prevents blocking the audio thread or UI thread during synthesis
pub trait TtsEngine: Send + Sync {
    /// Synthesize speech from text.
    fn synthesize(&self, request: &TtsRequest) -> Result<TtsResult, crate::error::TtsError>;

    /// Get the name of this TTS engine.
    fn name(&self) -> &str;

    /// Check if the engine is available/initialized.
    fn is_available(&self) -> bool;
}
