//! Track panel (left side) state.

use serde::{Deserialize, Serialize};

/// A single track in the DAW.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackView {
    pub id: u32,
    pub name: String,
    pub color: String, // CSS hex color
    pub muted: bool,
    pub solo: bool,
    pub armed: bool,
    pub volume: f32,
    pub pan: f32,
    pub kind: TrackKind,
}

/// The type of a track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackKind {
    Audio,
    Midi,
    Drum,
    Bus,
}

/// State for the left track panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPanelState {
    pub tracks: Vec<TrackView>,
    pub selected_track: Option<u32>,
}

impl Default for TrackPanelState {
    fn default() -> Self {
        let tracks = (0..8u32)
            .map(|i| {
                let kind = if i == 7 { TrackKind::Drum } else { TrackKind::Audio };
                let name = if i == 7 {
                    "Drums".to_string()
                } else {
                    format!("Track {}", i + 1)
                };
                let colors = [
                    "#ff6b6b", "#ffd93d", "#6bcb77", "#4d96ff",
                    "#c77dff", "#ff9f1c", "#2ec4b6", "#e76f51",
                ];
                TrackView {
                    id: i,
                    name,
                    color: colors[i as usize % colors.len()].to_string(),
                    muted: false,
                    solo: false,
                    armed: false,
                    volume: 0.8,
                    pan: 0.0,
                    kind,
                }
            })
            .collect();

        Self {
            tracks,
            selected_track: None,
        }
    }
}
