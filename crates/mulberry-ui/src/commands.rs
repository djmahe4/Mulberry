//! UI commands (frontend → backend) and responses (backend → frontend).

use serde::{Deserialize, Serialize};

/// Commands that the UI frontend can send to the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiCommand {
    // Transport
    Play,
    Stop,
    Pause,
    SetBpm(f64),
    Seek(f64),

    // Live coding
    EvalCode(String),

    // Drum machine
    SetDrumStep { track: usize, step: usize, active: bool, velocity: f32 },
    SetDrumBpm(f32),
    PlayDrums,
    StopDrums,

    // Mixer
    SetChannelVolume { channel: usize, volume: f32 },
    SetChannelPan { channel: usize, pan: f32 },
    SetMasterVolume(f32),

    // Agent
    SendAgentMessage { agent_id: String, message: String },

    // Plugin
    SetPluginParameter { plugin_id: String, param_id: String, value: f32 },

    // Note input
    NoteOn { channel: u8, note: u8, velocity: f32 },
    NoteOff { channel: u8, note: u8 },

    /// Request a full state snapshot.
    GetState,

    /// Shutdown.
    Quit,
}

/// Responses that the backend sends to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiResponse {
    /// Full state snapshot.
    StateSnapshot(crate::app_state::UiState),
    /// A single field update.
    BpmChanged(f64),
    PlaybackStarted,
    PlaybackStopped,
    PositionUpdate(f64),
    DrumStepAdvanced(usize),
    AgentResponse { agent_id: String, content: String },
    Error(String),
    Ok,
}
