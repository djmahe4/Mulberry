//! # Mulberry Plugins
//!
//! Built-in DSP plugins for the Mulberry DAW.
//!
//! All plugins implement the [`Plugin`] trait with block-based processing.
//! No heap allocation occurs inside `process_block`.
//!
//! # CONTEXT7 REVIEW:
//! Problem: Plugin audio callbacks must be real-time safe
//! Decision: process_block takes borrowed slices, no allocation, stateful DSP uses pre-allocated state
//! Why this is correct: Block-based processing with pre-allocated state is the standard
//!   approach for real-time DSP. All state is initialized at plugin creation.

pub mod bass_enhancer;
pub mod compressor;
pub mod delay;
pub mod drum_enhancer;
pub mod drum_machine;
pub mod eq;
pub mod plugin;
pub mod reverb;
pub mod sample_manager;
pub mod synth_bass;

pub use bass_enhancer::BassEnhancer;
pub use compressor::Compressor;
pub use delay::Delay;
pub use drum_enhancer::DrumEnhancer;
pub use drum_machine::{DrumMachine, DrumPattern, DrumStep, DRUM_STEPS};
pub use eq::EightBandEq;
pub use plugin::{Plugin, PluginInfo, PluginParameter};
pub use reverb::Reverb;
pub use sample_manager::{SampleEntry, SampleManager};
pub use synth_bass::{SynthBass, SynthBassParams};
