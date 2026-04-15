//! # Mulberry UI
//!
//! Desktop UI layer for the Mulberry DAW.
//!
//! Provides the application state model and message types that the frontend
//! (Tauri/web view) communicates with.
//!
//! # Layout specification
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  TOP BAR  ─ Transport controls + Agent panel               │
//! ├──────────┬────────────────────────────────┬────────────────┤
//! │  LEFT    │   CENTER                        │  RIGHT         │
//! │  Tracks  │   Code editor (Monaco/CodeMirror)│  Mixer         │
//! │  Drums   │                                 │  Plugins       │
//! ├──────────┴────────────────────────────────┴────────────────┤
//! │  BOTTOM  ─ Piano roll + Waveform view                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # CONTEXT7 REVIEW:
//! Problem: UI state must be shareable between async backend and frontend
//! Decision: Use Arc<RwLock<AppState>> for shared state; frontend receives diffs via channels
//! Why this is correct: RwLock allows many concurrent readers (UI polling) with
//!   exclusive write access for audio engine updates. Arc enables Send across async tasks.

pub mod app_state;
pub mod code_editor;
pub mod commands;
pub mod drum_panel;
pub mod mixer_panel;
pub mod plugin_panel;
pub mod track_panel;
pub mod transport_panel;

pub use app_state::{AppState, UiState};
pub use commands::{UiCommand, UiResponse};
