# 🎵 Mulberry — Next-Gen Native DAW

Mulberry is a **production-grade, native desktop DAW + live-coding environment** built entirely in Rust.

> **Legacy notice:** The original Python/Streamlit web app (`mulberry_audio.py`) has been superseded.  
> See [`docs/programming-guide.md`](docs/programming-guide.md) for the migration guide.

---

## ✨ Features

| Category | Feature |
|---|---|
| 🎧 **Audio Engine** | Real-time audio via [cpal](https://github.com/RustAudio/cpal); lock-free callback |
| 🎼 **Live Coding** | Strudel/TidalCycles-style mini-notation: `"c4 [e4 g4] ~ c5"` |
| 🥁 **Drum Machine** | 16-step × 8-track sequencer with velocity and accent |
| 🎸 **Synth Bass** | Monophonic bass synth (osc + ADSR + filter + tanh distortion) |
| 🔌 **Plugins** | EQ (8-band), Compressor, Reverb, Delay, Drum Enhancer, Bass Enhancer |
| 🤖 **AI Agents** | Generator, Editor, Harmony, Debug agents via `tokio::task::JoinSet` |
| 🗣️ **TTS** | Formant synthesizer (zero-dependency) + espeak-ng subprocess backend |
| 🖥️ **UI** | Tauri-compatible state model; tracks/drums/code editor/piano roll/mixer |
| 🔧 **Plugin SDK** | Dynamic library loading for third-party plugins |

---

## 📦 Workspace Structure

```
Mulberry/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── mulberry-core/          # Core types, EventBus, time, config, errors
│   ├── mulberry-audio/         # cpal real-time audio engine + voices + graph
│   ├── mulberry-plugins/       # Built-in DSP plugins + drum machine + synth bass
│   ├── mulberry-ui/            # UI state model (Tauri-compatible)
│   ├── mulberry-dsp/           # Oscillators, filters, ADSR envelopes, mixer
│   ├── mulberry-transport/     # DAW transport, timeline, metronome
│   ├── mulberry-livecode/      # Live-code lexer → parser → pattern engine
│   ├── mulberry-agent/         # AI agent orchestration (JoinSet)
│   ├── mulberry-tts/           # TTS synthesis (formant + subprocess)
│   ├── mulberry-plugin-sdk/    # Dynamic plugin loader + HostApi trait
│   └── mulberry-app/           # Binary entry point
└── docs/
    ├── programming-guide.md    # Architecture, async patterns, plugin dev
    └── music-theory-guide.md   # Fundamentals through production
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# Linux (ALSA)
sudo apt install libasound2-dev pkg-config

# macOS (CoreAudio — no extra steps needed)

# Windows (WASAPI — no extra steps needed)
```

### Build

```bash
git clone https://github.com/djmahe4/Mulberry
cd Mulberry
cargo build --release
```

### Run

```bash
cargo run --release --bin mulberry
```

---

## 🎹 Live-Coding Pattern Syntax

```text
"c4 e4 g4 c5"        -- sequence (4 notes per cycle)
"c4 [e4 g4]"         -- subdivision (e4+g4 share one slot)
"c4 ~ e4 ~"          -- rests (~ = silence)
"c4*3 e4"            -- repeat c4 three times
"c4 e4 g4" | fast 2  -- double speed
"c4 e4 g4" | slow 2  -- half speed
"c4 e4 g4" | rev     -- reverse pattern
```

---

## 🧵 Async Concurrency Model

| Context | Pattern | Reason |
|---|---|---|
| Main event loop | `tokio::select!` | Multiplex bus + signals |
| Agent LLM calls | `JoinSet` | Structured, abortable tasks |
| Real-time event streams | `FuturesUnordered` | No `'static` requirement |
| Audio callback | Lock-free channels + `try_lock` | Real-time safety |
| UI shared state | `Arc<RwLock<T>>` | Many readers, safe writes |

---

## 🔌 Plugin API

```rust
pub trait Plugin: Send {
    /// Process a mono block in-place.
    /// REALTIME SAFE: no allocation, no locks
    fn process_block(&mut self, input: &[f32], output: &mut [f32]);

    fn info(&self) -> PluginInfo;
    fn parameters(&self) -> Vec<PluginParameter>;
    fn set_parameter(&mut self, id: &str, value: f32);
    fn get_parameter(&self, id: &str) -> Option<f32>;
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn reset(&mut self);
}
```

Built-in plugins: **EQ** (8-band) · **Compressor** · **Reverb** · **Delay** · **Drum Enhancer** · **Bass Enhancer**

---

## 📚 Documentation

- [`docs/programming-guide.md`](docs/programming-guide.md) — Architecture, DSP safety, plugin dev
- [`docs/music-theory-guide.md`](docs/music-theory-guide.md) — Music theory from basics to production

---

## 🏗️ Architecture Overview

```
                         EventBus (crossbeam broadcast)
                                │
          ┌─────────────────────┼──────────────────────┐
          ▼                     ▼                       ▼
    Transport             LiveCode Engine         Agent Orchestrator
    (play/stop/BPM)       (pattern parser)        (JoinSet, LLM calls)
          │                     │                       │
          └──────────┬──────────┘                       │
                     ▼                                   │
              Audio Engine (cpal)                        │
              ┌──────┴──────┐                           │
             Voices     Plugin Chain                     │
              │         (EQ→Comp→Reverb…)               │
              └──────┬──────┘                           │
                     ▼                                   ▼
                DAC (speakers)                    UI State (Arc<RwLock>)
```

---

## License

See [LICENSE](LICENSE) for details.
