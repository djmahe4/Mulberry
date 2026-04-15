//! # Mulberry Transport
//!
//! DAW transport system: play/stop/pause/seek, BPM management, timeline tracking,
//! and metronome generation.
//!
//! The transport is the central timing authority for the entire DAW. It listens
//! for `TransportEvent`s on the `EventBus` and maintains the canonical position
//! and playback state.

pub mod state;
pub mod timeline;
pub mod metronome;

pub use state::{Transport, TransportState};
pub use timeline::Timeline;
pub use metronome::Metronome;
