//! Mixer panel (right side) state.

use serde::{Deserialize, Serialize};

/// A single mixer channel strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    pub id: u32,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub peak_level: f32,
    pub plugin_chain: Vec<String>,
}

/// State for the right mixer panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerPanelState {
    pub channels: Vec<MixerChannel>,
    pub master_volume: f32,
    pub master_peak: f32,
}

impl Default for MixerPanelState {
    fn default() -> Self {
        let channels = (0..8u32)
            .map(|i| MixerChannel {
                id: i,
                name: format!("Ch {}", i + 1),
                volume: 0.8,
                pan: 0.0,
                muted: false,
                solo: false,
                peak_level: 0.0,
                plugin_chain: Vec::new(),
            })
            .collect();
        Self {
            channels,
            master_volume: 1.0,
            master_peak: 0.0,
        }
    }
}
