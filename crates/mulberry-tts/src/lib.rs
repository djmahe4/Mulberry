//! # Mulberry TTS
//!
//! Text-to-speech vocal synthesis pipeline for the Mulberry DAW.
//!
//! Provides multiple TTS backends:
//! - Built-in formant synthesizer (basic, no external dependencies)
//! - External process backend (wraps espeak-ng, festival, etc.)
//!
//! # CONTEXT7 REVIEW:
//! Issue: TTS must be free/open-source and work without network
//! Resolution: Provide a trait-based interface with pluggable backends.
//!   Built-in formant synth for zero-dependency fallback,
//!   subprocess wrapper for high-quality external engines.
//! Why: Avoids licensing issues, allows users to choose their preferred engine,
//!      and works offline by default.

pub mod engine;
pub mod error;
pub mod formant;
pub mod phoneme;
pub mod subprocess;

pub use engine::{TtsEngine, TtsRequest, TtsResult};
pub use error::TtsError;
pub use formant::FormantSynthesizer;
pub use subprocess::SubprocessTtsEngine;
