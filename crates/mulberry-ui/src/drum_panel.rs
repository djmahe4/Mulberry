//! Drum machine panel state (left panel, bottom section).

use serde::{Deserialize, Serialize};

/// Names of the 8 default drum tracks.
pub const DRUM_TRACK_NAMES: [&str; 8] = [
    "Kick", "Snare", "Hi-Hat", "Open HH",
    "Tom 1", "Tom 2", "Clap",  "Perc",
];

/// State for the 16-step drum machine panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPanelState {
    /// 8 tracks × 16 steps active flags
    pub steps: [[bool; 16]; 8],
    /// 8 tracks × 16 steps velocities (0.0..1.0)
    pub velocities: [[f32; 16]; 8],
    /// Currently highlighted step (during playback)
    pub playhead: usize,
    /// Is drum machine playing
    pub playing: bool,
    /// BPM
    pub bpm: f32,
    /// Track mute states
    pub muted: [bool; 8],
    /// Track names
    pub track_names: [String; 8],
}

impl Default for DrumPanelState {
    fn default() -> Self {
        // Default kick pattern: steps 0, 4, 8, 12
        let mut steps = [[false; 16]; 8];
        steps[0][0] = true;
        steps[0][4] = true;
        steps[0][8] = true;
        steps[0][12] = true;
        // Default snare: steps 4, 12
        steps[1][4] = true;
        steps[1][12] = true;
        // Default hi-hat: every 2 steps
        for i in (0..16).step_by(2) {
            steps[2][i] = true;
        }

        Self {
            steps,
            velocities: [[0.8; 16]; 8],
            playhead: 0,
            playing: false,
            bpm: 120.0,
            muted: [false; 8],
            track_names: DRUM_TRACK_NAMES.map(|s| s.to_string()),
        }
    }
}
