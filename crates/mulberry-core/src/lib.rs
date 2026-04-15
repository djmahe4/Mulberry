//! # Mulberry Core
//!
//! Core types, events, configuration, and shared state for the Mulberry DAW.
//!
//! This crate provides the foundational abstractions used across all Mulberry subsystems:
//! - Audio frame types and sample formats
//! - Event bus for inter-component communication
//! - Configuration types
//! - Error types
//! - Time and musical position representations

pub mod config;
pub mod drums;
pub mod error;
pub mod event;
pub mod sample;
pub mod time;

pub use config::MulberryConfig;
pub use drums::{DrumHit, DrumMachineState};
pub use error::MulberryError;
pub use event::{Event, EventBus};
pub use sample::{AudioBuffer, AudioFrame, Sample};
pub use time::{MusicalPosition, Tempo, TickClock, TimeSignature};
