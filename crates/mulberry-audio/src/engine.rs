use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use tracing::{error, info};

use mulberry_core::{MulberryConfig, MulberryError, Sample};

use crate::voice::{Voice, VoiceParams};

/// Commands sent to the audio engine from the control thread.
#[derive(Debug, Clone)]
pub enum EngineCommand {
    /// Add a voice to the engine.
    AddVoice(VoiceParams),
    /// Remove all voices.
    ClearVoices,
    /// Set master gain (linear).
    SetMasterGain(f32),
    /// Shutdown the engine.
    Shutdown,
}

/// Shared mutable state accessed by the audio callback.
struct EngineState {
    voices: Vec<Voice>,
    master_gain: f32,
}

/// Real-time audio engine backed by `cpal`.
///
/// The engine owns a cpal output stream and processes audio in a high-priority
/// callback thread. Control commands are delivered through a bounded
/// `crossbeam_channel`.
pub struct AudioEngine {
    _stream: cpal::Stream,
    cmd_tx: Sender<EngineCommand>,
    config: MulberryConfig,
}

impl AudioEngine {
    /// Create and start a new audio engine.
    ///
    /// Returns an error if the audio device cannot be opened or the stream
    /// fails to start.
    pub fn new(config: &MulberryConfig) -> Result<Self, MulberryError> {
        let host = cpal::default_host();

        let device = host.default_output_device().ok_or_else(|| {
            MulberryError::AudioEngine("no default output device available".into())
        })?;

        info!(
            "Using audio device: {}",
            device
                .name()
                .unwrap_or_else(|_| "<unknown>".to_string())
        );

        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        let (cmd_tx, cmd_rx): (Sender<EngineCommand>, Receiver<EngineCommand>) = bounded(256);

        let state = Arc::new(Mutex::new(EngineState {
            voices: Vec::new(),
            master_gain: 1.0,
        }));

        let sample_rate = config.sample_rate as f32;
        let channels = config.channels as usize;
        let callback_state = Arc::clone(&state);

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [Sample], _: &cpal::OutputCallbackInfo| {
                    audio_callback(data, &cmd_rx, &callback_state, sample_rate, channels);
                },
                move |err| {
                    error!("cpal stream error: {err}");
                },
                None,
            )
            .map_err(|e| MulberryError::AudioEngine(e.to_string()))?;

        stream
            .play()
            .map_err(|e| MulberryError::AudioEngine(e.to_string()))?;

        info!("Audio engine started ({}Hz, {} ch)", config.sample_rate, config.channels);

        Ok(Self {
            _stream: stream,
            cmd_tx,
            config: config.clone(),
        })
    }

    /// Send an arbitrary command to the engine.
    pub fn send_command(&self, cmd: EngineCommand) -> Result<(), MulberryError> {
        self.cmd_tx
            .try_send(cmd)
            .map_err(|e| MulberryError::AudioEngine(format!("failed to send command: {e}")))
    }

    /// Convenience: add a voice that is immediately triggered.
    pub fn add_voice(&self, params: VoiceParams) -> Result<(), MulberryError> {
        self.send_command(EngineCommand::AddVoice(params))
    }

    /// Convenience: set the master output gain.
    pub fn set_master_gain(&self, gain: f32) -> Result<(), MulberryError> {
        self.send_command(EngineCommand::SetMasterGain(gain))
    }

    /// Convenience: request engine shutdown.
    pub fn stop(&self) -> Result<(), MulberryError> {
        self.send_command(EngineCommand::Shutdown)
    }

    /// Return a reference to the config used to create this engine.
    pub fn config(&self) -> &MulberryConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Audio callback – runs on the cpal audio thread
// ---------------------------------------------------------------------------

// CONTEXT7 REVIEW:
// Issue: Using Mutex in audio callback
// Resolution: parking_lot::Mutex with try_lock() in the callback - if lock fails, output silence
// Why: Prevents priority inversion. Audio thread never blocks waiting for lock.

fn audio_callback(
    data: &mut [Sample],
    cmd_rx: &Receiver<EngineCommand>,
    state: &Arc<Mutex<EngineState>>,
    sample_rate: f32,
    channels: usize,
) {
    // Try to acquire the lock without blocking.
    let Some(mut state) = state.try_lock() else {
        // Lock contended – output silence to avoid glitches.
        data.iter_mut().for_each(|s| *s = 0.0);
        return;
    };

    // Drain pending commands (non-blocking).
    while let Ok(cmd) = cmd_rx.try_recv() {
        match cmd {
            EngineCommand::AddVoice(params) => {
                let mut voice = Voice::new(&params, sample_rate);
                voice.trigger();
                state.voices.push(voice);
            }
            EngineCommand::ClearVoices => {
                state.voices.clear();
            }
            EngineCommand::SetMasterGain(g) => {
                state.master_gain = g;
            }
            EngineCommand::Shutdown => {
                state.voices.clear();
                data.iter_mut().for_each(|s| *s = 0.0);
                return;
            }
        }
    }

    let master_gain = state.master_gain;

    // Generate audio: iterate over frames (one frame = `channels` samples).
    let frame_count = data.len() / channels.max(1);
    for frame_idx in 0..frame_count {
        let mut mix: f32 = 0.0;

        for voice in state.voices.iter_mut() {
            if voice.is_active() {
                mix += voice.next_sample();
            }
        }

        mix *= master_gain;

        // Clamp to [-1, 1] to avoid hard clipping downstream.
        mix = mix.clamp(-1.0, 1.0);

        // Write the same mono mix to every channel.
        let base = frame_idx * channels;
        for ch in 0..channels {
            if let Some(sample) = data.get_mut(base + ch) {
                *sample = mix;
            }
        }
    }

    // Remove finished voices to avoid unbounded growth.
    state.voices.retain(|v| v.is_active());
}
