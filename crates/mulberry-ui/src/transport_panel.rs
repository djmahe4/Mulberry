//! Transport panel state and logic.

use serde::{Deserialize, Serialize};

/// State for the transport bar at the top of the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportPanelState {
    pub bpm: f64,
    pub time_signature_num: u8,
    pub time_signature_den: u8,
    pub playing: bool,
    pub recording: bool,
    pub loop_enabled: bool,
    pub loop_start: f64,
    pub loop_end: f64,
    pub position_bars: u64,
    pub position_beats: f64,
}

impl Default for TransportPanelState {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            time_signature_num: 4,
            time_signature_den: 4,
            playing: false,
            recording: false,
            loop_enabled: false,
            loop_start: 0.0,
            loop_end: 4.0,
            position_bars: 1,
            position_beats: 0.0,
        }
    }
}
