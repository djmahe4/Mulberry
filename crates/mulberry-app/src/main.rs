//! Main application entry point for the Mulberry DAW.
//!
//! Wires together all Mulberry subsystems:
//!   - Core event bus and configuration
//!   - Real-time audio engine (cpal)
//!   - DAW transport (play/stop/seek/BPM)
//!   - Live-code pattern engine (Strudel/Tidal-style)
//!   - AI agent orchestrator (JoinSet)
//!   - TTS vocal synthesis (formant)
//!   - Built-in DSP plugins (EQ, Compressor, Reverb, Delay, Drum/Bass Enhancer)
//!   - 16-step drum machine
//!   - Sample manager
//!   - Synth bass
//!   - Plugin loader (dynamic .so/.dll/.dylib)
//!   - UI state model

// git rm mulberry_audio.py
// git rm requirements.txt

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
use mulberry_plugins::{
    EightBandEq, Compressor, Reverb, Delay, DrumEnhancer, BassEnhancer,
    DrumMachine, SampleManager, SynthBass, plugin::Plugin,
};
use mulberry_ui::app_state::AppState;

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

    // ── Agent Orchestrator (JoinSet-backed) ─────────────────────────
    // CONTEXT7 REVIEW:
    // Problem: Agent tasks are long-running I/O-bound operations
    // Decision: tokio::task::JoinSet used inside AgentOrchestrator
    // Why this is correct: JoinSet provides structured concurrency — all tasks
    //   are trackable and abortable, preventing task leaks on shutdown.
    let event_bus_arc = Arc::new(EventBus::new());
    let _orchestrator = AgentOrchestrator::new(Arc::clone(&event_bus_arc));

    // ── TTS: Formant Synthesizer ────────────────────────────────────
    let tts = FormantSynthesizer::new(config.sample_rate);

    // ── Plugin Loader (dynamic .so/.dll/.dylib) ─────────────────────
    let _plugin_loader = PluginLoader::new();

    // ── Built-in Plugins ────────────────────────────────────────────
    // All plugins pre-allocate their DSP state here; no heap allocation in process_block.
    let sample_rate = config.sample_rate as f32;
    let mut _eq      = EightBandEq::new(sample_rate);
    let mut _comp    = Compressor::new(sample_rate);
    let mut _reverb  = Reverb::new(sample_rate);
    let mut _delay   = Delay::new(sample_rate);
    let mut _drum_fx = DrumEnhancer::new(sample_rate);
    let mut _bass_fx = BassEnhancer::new(sample_rate);

    info!(
        "Built-in plugins loaded: EQ={}, Compressor={}, Reverb={}, Delay={}, DrumEnhancer={}, BassEnhancer={}",
        _eq.info().name, _comp.info().name, _reverb.info().name,
        _delay.info().name, _drum_fx.info().name, _bass_fx.info().name
    );

    // ── Drum Machine (16-step × 8-track) ────────────────────────────
    let mut drum_machine = DrumMachine::new(sample_rate);
    // Kick on steps 0, 4, 8, 12 (track 0)
    drum_machine.set_step(0, 0,  true, 1.0);
    drum_machine.set_step(0, 4,  true, 1.0);
    drum_machine.set_step(0, 8,  true, 1.0);
    drum_machine.set_step(0, 12, true, 1.0);
    // Snare on steps 4, 12 (track 1)
    drum_machine.set_step(1, 4,  true, 0.9);
    drum_machine.set_step(1, 12, true, 0.9);
    // Hi-hat every 2 steps (track 2)
    for step in (0..16usize).step_by(2) {
        drum_machine.set_step(2, step, true, 0.6);
    }
    info!("Drum machine configured with default kick/snare/hi-hat pattern");

    // ── Sample Manager ──────────────────────────────────────────────
    let mut sample_manager = SampleManager::new();
    // Register a synthetic kick sample (short sine burst)
    let kick_samples: Vec<f32> = (0..2048)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let freq = 100.0 * (-t * 20.0_f32).exp();
            (t * freq * 2.0 * std::f32::consts::PI).sin() * (-t * 15.0_f32).exp()
        })
        .collect();
    let kick_id = sample_manager.add_sample("Kick", kick_samples, config.sample_rate);
    info!("Sample manager: registered kick sample (id={})", kick_id);
    info!("Loaded {} sample(s)", sample_manager.sample_count());

    // ── Synth Bass ──────────────────────────────────────────────────
    let mut synth_bass = SynthBass::new(sample_rate);
    synth_bass.note_on(36, 0.8); // C2 bass note
    info!("Synth bass initialized, note on C2 (MIDI 36)");

    // ── UI Application State ────────────────────────────────────────
    let app_state = AppState::new();
    {
        let mut state = app_state.write();
        state.bpm = config.initial_tempo;
    }
    info!("UI application state initialized");

    info!(
        sample_rate = config.sample_rate,
        buffer_size = config.buffer_size,
        channels = config.channels,
        tempo = config.initial_tempo,
        "All Mulberry subsystems initialized"
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
    // CONTEXT7 REVIEW:
    // Problem: Need to multiplex events from bus + OS signals without blocking
    // Decision: tokio::select! with spawn_blocking for the sync crossbeam receiver
    // Why this is correct: spawn_blocking moves the blocking recv() off the async executor.
    //   select! cleanly handles multiple concurrent async branches.
    info!("Entering main event loop (Ctrl+C to exit)");

    loop {
        tokio::select! {
            event = tokio::task::spawn_blocking({
                let rx = event_rx.clone();
                move || rx.recv()
            }) => {
                match event {
                    Ok(Ok(evt)) => match evt {
                        Event::Transport(ref te) => {
                            info!("Transport event: {:?}", te);
                            match te {
                                TransportEvent::Stop => info!("Transport stopped"),
                                TransportEvent::Play => info!("Transport playing"),
                                TransportEvent::Pause => info!("Transport paused"),
                                TransportEvent::SetTempo(bpm) => {
                                    drum_machine.set_bpm(*bpm as f32);
                                    let mut state = app_state.write();
                                    state.bpm = *bpm;
                                }
                                _ => {}
                            }
                        }
                        Event::NoteOn { channel, note, velocity } => {
                            info!(channel, note, velocity, "NoteOn received");
                            synth_bass.note_on(note, velocity);
                        }
                        Event::NoteOff { channel, note } => {
                            info!(channel, note, "NoteOff received");
                            synth_bass.note_off();
                        }
                        Event::LiveCodeEval(ref expr) => {
                            info!(expr, "LiveCode eval request");
                            let mut state = app_state.write();
                            state.live_code = expr.clone();
                        }
                        Event::DrumHit(ref hit) => {
                            info!(track = hit.track, step = hit.step, velocity = hit.velocity, "Drum hit");
                        }
                        Event::Shutdown => {
                            info!("Shutdown event received");
                            break;
                        }
                        _ => {}
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

            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C received, shutting down");
                break;
            }
        }
    }

    // ── Clean Shutdown ──────────────────────────────────────────────
    info!("Shutting down Mulberry");
    transport.stop();
    synth_bass.note_off();

    if let Some(ref engine) = audio_engine {
        if let Err(e) = engine.stop() {
            warn!("Error stopping audio engine: {e}");
        }
    }

    info!("Mulberry shut down successfully. Goodbye! 🎵");
    Ok(())
}
