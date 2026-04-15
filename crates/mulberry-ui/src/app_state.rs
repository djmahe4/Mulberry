//! Application state model shared between backend and UI.

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};

/// The overall UI state visible to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    /// Current BPM
    pub bpm: f64,
    /// Is transport playing
    pub playing: bool,
    /// Current position in beats
    pub position_beats: f64,
    /// Current live-code pattern string
    pub live_code: String,
    /// Active plugin names on each track
    pub track_plugins: Vec<Vec<String>>,
    /// Drum machine pattern (16 steps × 8 tracks, active flags)
    pub drum_pattern: [[bool; 16]; 8],
    /// Current drum step (for UI highlight)
    pub current_drum_step: usize,
    /// Mixer channel volumes (linear 0.0-1.0)
    pub channel_volumes: Vec<f32>,
    /// Mixer channel pans (-1.0 left to 1.0 right)
    pub channel_pans: Vec<f32>,
    /// Log messages for the agent panel
    pub agent_log: Vec<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            playing: false,
            position_beats: 0.0,
            live_code: String::from(r#""c4 e4 g4 c5""#),
            track_plugins: vec![Vec::new(); 8],
            drum_pattern: [[false; 16]; 8],
            current_drum_step: 0,
            channel_volumes: vec![0.8; 8],
            channel_pans: vec![0.0; 8],
            agent_log: Vec::new(),
        }
    }
}

/// Shared, thread-safe application state.
///
/// # CONTEXT7 REVIEW:
/// Problem: AppState must be Send + Sync for use across tokio tasks and UI thread
/// Decision: Wrap UiState in Arc<RwLock<T>> from std
/// Why this is correct: std::sync::RwLock is Send+Sync and supports many concurrent
///   readers. Arc provides shared ownership across tasks.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<RwLock<UiState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(UiState::default())),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, UiState> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, UiState> {
        self.inner.write().unwrap()
    }

    pub fn snapshot(&self) -> UiState {
        self.inner.read().unwrap().clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
