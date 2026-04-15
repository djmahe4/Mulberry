//! Error types for the Mulberry DAW.
//!
//! [`MulberryError`] is the unified error type returned by all public APIs in
//! the Mulberry ecosystem. Each variant maps to a specific subsystem.

use thiserror::Error;

/// A unified error type covering every Mulberry subsystem.
///
/// Variants that wrap a [`String`] carry a human-readable description of the
/// failure. The `Io` and `SerdeJson` variants provide automatic conversion
/// from the corresponding standard-library / serde errors.
// CONTEXT7 REVIEW: Using `thiserror` keeps the boilerplate minimal while
// giving each variant a clear `Display` implementation. New subsystem errors
// should be added as additional variants.
#[derive(Debug, Error)]
pub enum MulberryError {
    /// An error originating from the audio engine.
    #[error("audio engine error: {0}")]
    AudioEngine(String),

    /// An error originating from the DSP subsystem.
    #[error("dsp error: {0}")]
    Dsp(String),

    /// An error originating from the live-coding evaluator.
    #[error("live code error: {0}")]
    LiveCode(String),

    /// An error originating from the transport subsystem.
    #[error("transport error: {0}")]
    Transport(String),

    /// An error originating from the AI agent subsystem.
    #[error("agent error: {0}")]
    Agent(String),

    /// An error originating from the TTS subsystem.
    #[error("tts error: {0}")]
    Tts(String),

    /// An error originating from the plugin subsystem.
    #[error("plugin error: {0}")]
    Plugin(String),

    /// A configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// An I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization / deserialization error.
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
