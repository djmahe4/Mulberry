//! Main application entry point for the Mulberry DAW.

use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info, warn};

use mulberry_audio::AudioEngine;
use mulberry_audio::voice::VoiceParams;
use mulberry_core::event::{Event, EventBus, TransportEvent};
use mulberry_core::MulberryConfig;
use mulberry_dsp::Waveform;
use mulberry_livecode::parser::parse_pattern;
use mulberry_livecode::pattern::Pattern;
use mulberry_livecode::PatternScheduler;
use mulberry_agent::AgentOrchestrator;
use mulberry_plugin_sdk::PluginLoader;
use mulberry_tts::engine::{TtsEngine, TtsRequest};
use mulberry_tts::FormantSynthesizer;
use mulberry_transport::Transport;

// CONTEXT7 REVIEW:
// Issue: Main event loop pattern selection
// Resolution: tokio::select! for multiplexing event sources (bus, signals, timers)
// Why: select! is the correct pattern for the main coordination loop that
//      dispatches between different event sources. No spawning needed here.

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Mulberry DAW starting up");

    // ── Configuration ───────────────────────────────────────────────
    let config = MulberryConfig::default();

    // ── Event Bus ───────────────────────────────────────────────────
    let event_bus = EventBus::new();
    let event_rx = event_bus.subscribe();

    // ── Transport ───────────────────────────────────────────────────
    let mut transport = Transport::new(&config, event_bus);

    // ── Audio Engine (graceful fallback if no device) ───────────────
    let audio_engine = match AudioEngine::new(&config) {
        Ok(engine) => {
            info!("Audio engine initialized successfully");
            Some(engine)
        }
        Err(e) => {
            warn!("Audio engine unavailable (no audio device?): {e}");
            warn!("Continuing without audio output");
            None
        }
    };

    // ── Pattern Scheduler ───────────────────────────────────────────
    let _scheduler = PatternScheduler::new(config.sample_rate);

    // ── Agent Orchestrator ──────────────────────────────────────────
    let event_bus_arc = Arc::new(EventBus::new());
    let _orchestrator = AgentOrchestrator::new(Arc::clone(&event_bus_arc));

    // ── TTS: Formant Synthesizer ────────────────────────────────────
    let tts = FormantSynthesizer::new(config.sample_rate);

    // ── Plugin Loader ───────────────────────────────────────────────
    let _plugin_loader = PluginLoader::new();

    info!(
        sample_rate = config.sample_rate,
        buffer_size = config.buffer_size,
        channels = config.channels,
        tempo = config.initial_tempo,
        "Mulberry subsystems initialized"
    );

    // ── Demo Sequence ───────────────────────────────────────────────

    // (a) Parse a live-code pattern
    let ast = parse_pattern("c4 e4 g4 c5")?;
    info!("Parsed live-code pattern AST: {:?}", ast);

    // (b) Resolve into a Pattern
    let pattern = Pattern::from_ast(&ast)?;

    // (c) Log the pattern events
    for event in &pattern.events {
        info!(
            start = event.start,
            duration = event.duration,
            note = ?event.note,
            name = ?event.note_name,
            "Pattern event"
        );
    }

    // (d) Start transport playing
    transport.play();
    info!("Transport started (playing)");

    // (e) If audio engine is available, add a voice (sine, 440Hz)
    if let Some(ref engine) = audio_engine {
        let voice_params = VoiceParams {
            waveform: Waveform::Sine,
            frequency: 440.0,
            amplitude: 0.5,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
        };
        if let Err(e) = engine.add_voice(voice_params) {
            warn!("Failed to add voice to audio engine: {e}");
        } else {
            info!("Added sine voice at 440Hz to audio engine");
        }
    }

    // (f) Synthesize "Hello Mulberry" using FormantSynthesizer
    let tts_request = TtsRequest {
        text: "Hello Mulberry".to_string(),
        speed: 1.0,
        pitch: 1.0,
        sample_rate: config.sample_rate,
    };
    match tts.synthesize(&tts_request) {
        Ok(result) => {
            // (g) Log TTS result duration
            info!(
                duration_secs = result.duration_secs,
                samples = result.samples.len(),
                "TTS synthesized 'Hello Mulberry'"
            );
        }
        Err(e) => {
            warn!("TTS synthesis failed: {e}");
        }
    }

    // ── Event Processing Loop ───────────────────────────────────────
    info!("Entering main event loop (Ctrl+C to exit)");

    loop {
        tokio::select! {
            // Listen for events from the bus subscriber
            event = tokio::task::spawn_blocking({
                let rx = event_rx.clone();
                move || rx.recv()
            }) => {
                match event {
                    Ok(Ok(evt)) => match evt {
                        Event::Transport(ref te) => {
                            info!("Transport event: {:?}", te);
                            match te {
                                TransportEvent::Stop => {
                                    info!("Transport stopped");
                                }
                                TransportEvent::Play => {
                                    info!("Transport playing");
                                }
                                TransportEvent::Pause => {
                                    info!("Transport paused");
                                }
                                _ => {}
                            }
                        }
                        Event::NoteOn { channel, note, velocity } => {
                            info!(channel, note, velocity, "NoteOn received");
                        }
                        Event::NoteOff { channel, note } => {
                            info!(channel, note, "NoteOff received");
                        }
                        Event::LiveCodeEval(ref expr) => {
                            info!(expr, "LiveCode eval request");
                        }
                        Event::Shutdown => {
                            info!("Shutdown event received");
                            break;
                        }
                        _ => {
                            info!("Event: {:?}", evt);
                        }
                    },
                    Ok(Err(_)) => {
                        info!("Event bus channel closed");
                        break;
                    }
                    Err(e) => {
                        error!("Event receiver task failed: {e}");
                        break;
                    }
                }
            }

            // Listen for Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received, shutting down");
                break;
            }
        }
    }

    // ── Clean Shutdown ──────────────────────────────────────────────
    info!("Shutting down Mulberry");

    transport.stop();

    if let Some(ref engine) = audio_engine {
        if let Err(e) = engine.stop() {
            warn!("Error stopping audio engine: {e}");
        }
    }

    info!("Mulberry shut down successfully");
    Ok(())
}
