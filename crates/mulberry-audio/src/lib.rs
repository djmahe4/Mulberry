//! # Mulberry Audio
//!
//! Real-time audio engine for the Mulberry DAW.
//!
//! Architecture:
//! - Audio callback runs on a dedicated high-priority thread (via cpal)
//! - Communication with main thread via lock-free ring buffers and crossbeam channels
//! - No heap allocations in the audio callback path
//!
//! # CONTEXT7 REVIEW:
//! Issue: Audio callback must be real-time safe (no locks, no allocations, no syscalls)
//! Resolution: Use ring buffers for audio data, crossbeam channels for commands
//! Why: Lock-free communication ensures glitch-free audio even under load

pub mod engine;
pub mod graph;
pub mod voice;

pub use engine::AudioEngine;
pub use graph::{AudioGraph, AudioNode, NodeId};
pub use voice::Voice;
