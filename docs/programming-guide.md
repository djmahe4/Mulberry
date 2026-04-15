# Mulberry Programming Guide

## Architecture Overview

Mulberry is a multi-crate Rust workspace implementing a next-generation DAW + live-coding environment. It combines real-time audio processing, AI-powered orchestration, and a Strudel/TidalCycles-inspired pattern language into a single cohesive platform.

### Crate Map

| Crate | Purpose |
|-------|---------|
| `mulberry-core` | Core types, events, config, error handling |
| `mulberry-dsp` | DSP primitives: oscillators, filters, envelopes |
| `mulberry-audio` | Real-time audio engine (cpal integration) |
| `mulberry-transport` | DAW transport: play/stop/seek/BPM/timeline |
| `mulberry-livecode` | Strudel/TidalCycles-style pattern engine |
| `mulberry-agent` | AI agent orchestration |
| `mulberry-tts` | Text-to-speech vocal synthesis |
| `mulberry-plugin-sdk` | Plugin SDK with dynamic loading |
| `mulberry-app` | Main application binary |

Each crate is designed to be as independent as possible. `mulberry-core` provides the shared foundation (event types, configuration, error types) that all other crates depend on. The `mulberry-app` crate ties everything together into the final binary.

### Data Flow

```
User Input (live-code / GUI)
       │
       ▼
   EventBus (crossbeam broadcast)
       │
   ┌───┴───┬──────────┬──────────┐
   ▼       ▼          ▼          ▼
Transport  LiveCode   Agent      Plugin
   │       Parser     Orchestrator Loader
   │       │          │          │
   ▼       ▼          ▼          ▼
   TickClock  Pattern   JoinSet    Host API
   │       Scheduler  (tokio)    │
   │       │          │          │
   └───┬───┘──────────┘──────────┘
       ▼
   Audio Engine (cpal callback)
       │
   ┌───┴───┐
   ▼       ▼
  Voices  Audio Graph
   │       │
   └───┬───┘
       ▼
   DAC (speakers)
```

**How it works:**

1. **User input** arrives from either the live-coding REPL or a GUI interaction.
2. The input is dispatched onto the **EventBus**, a crossbeam broadcast channel that fans out events to all subscribers.
3. Subsystems — Transport, LiveCode Parser, Agent Orchestrator, Plugin Loader — each receive relevant events and process them independently.
4. All subsystems ultimately produce audio commands or sample data that feed into the **Audio Engine**.
5. The Audio Engine runs inside a **cpal callback** on a dedicated real-time thread, mixing voices and the audio graph before sending samples to the DAC.

---

## Async Concurrency Patterns

Mulberry uses four distinct concurrency patterns, each chosen for a specific use case. Picking the right pattern is critical for correctness and performance.

### 1. `FuturesUnordered` / `StreamExt::for_each_concurrent`

**Use for:** Real-time event processing where you need bounded concurrency over a stream of incoming events.

```rust
use futures::stream::{self, StreamExt};

let event_stream = event_bus.subscribe();

event_stream
    .for_each_concurrent(/* limit */ 8, |event| async move {
        match event {
            Event::NoteOn(note) => handle_note_on(note).await,
            Event::NoteOff(note) => handle_note_off(note).await,
            _ => {}
        }
    })
    .await;
```

**When to use:** When you have a stream of events and want to process up to N concurrently without spawning unbounded tasks. This is ideal for event handlers that do async I/O but must not overwhelm the system.

**Why:** It provides natural backpressure — if all N slots are busy, the stream pauses until one completes.

### 2. `tokio::task::JoinSet`

**Use for:** Agent tasks (LLM calls, TTS generation, background processing) where you spawn a dynamic set of tasks and need to collect their results.

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

// Spawn multiple agent tasks
set.spawn(async { llm_query("Generate a melody").await });
set.spawn(async { tts_synthesize("Hello world").await });
set.spawn(async { analyze_audio(buffer).await });

// Collect results as they complete
while let Some(result) = set.join_next().await {
    match result {
        Ok(output) => process_output(output),
        Err(e) => log::error!("Agent task failed: {e}"),
    }
}
```

**When to use:** When you need to manage a dynamic, bounded set of background tasks whose results you care about. JoinSet automatically cleans up tasks when dropped.

**Why:** Unlike raw `tokio::spawn`, JoinSet gives you structured concurrency — you can await all tasks, cancel them on drop, and handle errors in one place.

### 3. `LocalSet` + `spawn_local`

**Use for:** Local non-Send state, such as audio graph nodes that contain raw pointers or non-thread-safe FFI resources.

```rust
use tokio::task::LocalSet;

let local = LocalSet::new();

local.run_until(async {
    // This closure can use non-Send types
    let mut graph = AudioGraph::new(); // not Send

    tokio::task::spawn_local(async move {
        graph.process_block(&mut buffer);
    })
    .await
    .unwrap();
}).await;
```

**When to use:** When you have types that are `!Send` (e.g., raw pointers, Rc, RefCell-based state) but still want async processing.

**Why:** Tokio's default executor requires `Send` futures. `LocalSet` runs everything on a single thread, avoiding that requirement while still allowing async/await syntax.

### 4. `Arc<Mutex>` / Crossbeam Channels

**Use for:** Shared state between the async runtime and the real-time audio thread.

```rust
use std::sync::Arc;
use crossbeam_channel::{bounded, Sender, Receiver};

// Command channel: async world → audio thread
let (cmd_tx, cmd_rx): (Sender<AudioCommand>, Receiver<AudioCommand>) = bounded(256);

// In the audio callback (real-time thread):
fn audio_callback(cmd_rx: &Receiver<AudioCommand>, output: &mut [f32]) {
    // Non-blocking receive — never blocks the audio thread
    while let Ok(cmd) = cmd_rx.try_recv() {
        match cmd {
            AudioCommand::SetVolume(v) => { /* ... */ }
            AudioCommand::NoteOn(note) => { /* ... */ }
        }
    }
    // ... fill output buffer ...
}
```

**When to use:** For communication between threads where one side is a real-time audio callback that **must not block**.

**Why:** `crossbeam_channel` is lock-free for bounded channels and supports `try_recv()` / `try_send()`, making it safe for real-time use. `Arc<Mutex>` is acceptable for configuration state accessed with `try_lock()`.

---

## Real-Time Audio Safety Rules

The audio callback runs on a dedicated high-priority thread managed by the OS audio subsystem (via cpal). **Any stall in this callback causes audible glitches (clicks, pops, dropouts).** Follow these rules strictly:

### Rule 1: No Heap Allocations

```rust
// ❌ BAD — allocates on the heap
fn audio_callback(output: &mut [f32]) {
    let temp = vec![0.0f32; output.len()]; // ALLOCATION!
    // ...
}

// ✅ GOOD — use pre-allocated buffers
struct AudioState {
    scratch: Vec<f32>, // allocated once at init
}

fn audio_callback(state: &mut AudioState, output: &mut [f32]) {
    state.scratch.clear();
    state.scratch.extend(std::iter::repeat(0.0).take(output.len()));
    // No new allocation if capacity is sufficient
}
```

**Why:** `malloc`/`free` can take unbounded time due to lock contention and system calls.

### Rule 2: No Locks (Use `try_lock` with Silence Fallback)

```rust
// ❌ BAD — can block indefinitely
fn audio_callback(state: &Arc<Mutex<SharedState>>, output: &mut [f32]) {
    let state = state.lock().unwrap(); // BLOCKS!
}

// ✅ GOOD — try_lock with silence fallback
fn audio_callback(state: &Arc<Mutex<SharedState>>, output: &mut [f32]) {
    match state.try_lock() {
        Ok(s) => render_audio(&s, output),
        Err(_) => {
            // Output silence rather than block
            for sample in output.iter_mut() {
                *sample = 0.0;
            }
        }
    }
}
```

**Why:** A regular `lock()` can block if another thread holds the mutex, causing audio dropout.

### Rule 3: No Syscalls

Avoid any operation that might trigger a syscall:
- No file I/O (`read`, `write`, `open`)
- No `println!` or logging macros
- No `thread::sleep`
- No network operations

### Rule 4: Use Ring Buffers for Data Transfer

```rust
use ringbuf::{HeapRb, Producer, Consumer};

// Producer side (non-real-time thread)
let rb = HeapRb::<f32>::new(4096);
let (mut producer, mut consumer) = rb.split();

// Write samples from the loading thread
producer.push_slice(&loaded_samples);

// Consumer side (audio callback — real-time thread)
fn audio_callback(consumer: &mut Consumer<f32>, output: &mut [f32]) {
    let count = consumer.pop_slice(output);
    // Fill remainder with silence if not enough data
    for sample in &mut output[count..] {
        *sample = 0.0;
    }
}
```

### Rule 5: Use Crossbeam Channels for Commands

```rust
use crossbeam_channel::{bounded, Receiver};

enum AudioCommand {
    NoteOn { note: u8, velocity: f32 },
    NoteOff { note: u8 },
    SetParam { id: u32, value: f32 },
    Stop,
}

fn audio_callback(cmd_rx: &Receiver<AudioCommand>, voices: &mut VoicePool, output: &mut [f32]) {
    // Drain all pending commands (non-blocking)
    while let Ok(cmd) = cmd_rx.try_recv() {
        match cmd {
            AudioCommand::NoteOn { note, velocity } => voices.note_on(note, velocity),
            AudioCommand::NoteOff { note } => voices.note_off(note),
            AudioCommand::SetParam { id, value } => voices.set_param(id, value),
            AudioCommand::Stop => voices.all_notes_off(),
        }
    }

    // Render audio
    voices.render(output);
}
```

---

## Event System

### EventBus Overview

Mulberry uses a centralized **EventBus** built on crossbeam's broadcast channel. Every subsystem subscribes to the bus and receives all events, filtering for the ones it cares about.

```rust
use crossbeam_channel::{bounded, Sender, Receiver};

pub struct EventBus {
    sender: Sender<Event>,
    receivers: Vec<Receiver<Event>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = bounded(capacity);
        Self {
            sender,
            receivers: Vec::new(),
        }
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        // Each subscriber gets its own receiver
        self.sender.clone().into() // simplified
    }

    pub fn publish(&self, event: Event) {
        let _ = self.sender.try_send(event);
    }
}
```

### Event Types

```rust
#[derive(Debug, Clone)]
pub enum Event {
    // Transport events
    Play,
    Stop,
    Seek(f64),             // position in beats
    SetBpm(f64),

    // Note events
    NoteOn { note: u8, velocity: f32, channel: u8 },
    NoteOff { note: u8, channel: u8 },

    // LiveCode events
    CodeEval(String),       // user submitted code
    PatternUpdate(PatternId, Pattern),

    // Agent events
    AgentRequest(AgentTask),
    AgentResponse(AgentResult),

    // Plugin events
    PluginLoaded(PluginId),
    PluginUnloaded(PluginId),
    PluginParam { plugin: PluginId, param: u32, value: f32 },

    // System events
    Error(String),
    Shutdown,
}
```

### Subscribing to Events

```rust
// In a subsystem's initialization:
let rx = event_bus.subscribe();

tokio::spawn(async move {
    loop {
        match rx.recv() {
            Ok(Event::NoteOn { note, velocity, channel }) => {
                // Handle note on
            }
            Ok(Event::Stop) => break,
            Ok(_) => {} // Ignore events we don't care about
            Err(_) => break, // Channel closed
        }
    }
});
```

---

## Plugin Development

### The `MulberryPlugin` Trait

Every plugin implements the `MulberryPlugin` trait:

```rust
pub trait MulberryPlugin: Send + Sync {
    /// Plugin metadata
    fn info(&self) -> PluginInfo;

    /// Called once when the plugin is loaded
    fn init(&mut self, host: &dyn HostApi) -> Result<(), PluginError>;

    /// Process a block of audio samples (in-place)
    fn process(&mut self, buffer: &mut [f32], num_channels: usize);

    /// Handle an incoming event
    fn on_event(&mut self, event: &Event);

    /// Called when the plugin is unloaded
    fn cleanup(&mut self);
}

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub parameters: Vec<ParamInfo>,
}
```

### The `HostApi` Interface

Plugins interact with the host through the `HostApi`:

```rust
pub trait HostApi {
    /// Get the current sample rate
    fn sample_rate(&self) -> u32;

    /// Get the current buffer size
    fn buffer_size(&self) -> usize;

    /// Get the current BPM
    fn bpm(&self) -> f64;

    /// Get the current transport position (in beats)
    fn position(&self) -> f64;

    /// Send an event to the host
    fn send_event(&self, event: Event);

    /// Log a message
    fn log(&self, level: LogLevel, message: &str);
}
```

### Exporting a Plugin

Use the `export_plugin!` macro to make your plugin loadable:

```rust
use mulberry_plugin_sdk::prelude::*;

pub struct MyGainPlugin {
    gain: f32,
}

impl MulberryPlugin for MyGainPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "My Gain".into(),
            version: "1.0.0".into(),
            author: "Your Name".into(),
            description: "A simple gain plugin".into(),
            num_inputs: 2,
            num_outputs: 2,
            parameters: vec![
                ParamInfo {
                    name: "Gain".into(),
                    default: 1.0,
                    min: 0.0,
                    max: 2.0,
                },
            ],
        }
    }

    fn init(&mut self, host: &dyn HostApi) -> Result<(), PluginError> {
        host.log(LogLevel::Info, "MyGainPlugin initialized");
        Ok(())
    }

    fn process(&mut self, buffer: &mut [f32], _num_channels: usize) {
        for sample in buffer.iter_mut() {
            *sample *= self.gain;
        }
    }

    fn on_event(&mut self, event: &Event) {
        if let Event::PluginParam { param: 0, value, .. } = event {
            self.gain = *value;
        }
    }

    fn cleanup(&mut self) {}
}

// This macro generates the C-compatible FFI functions for dynamic loading
export_plugin!(MyGainPlugin, MyGainPlugin { gain: 1.0 });
```

Build your plugin as a dynamic library:

```toml
# Cargo.toml for your plugin
[lib]
crate-type = ["cdylib"]

[dependencies]
mulberry-plugin-sdk = { path = "../mulberry-plugin-sdk" }
```

---

## Live Coding Syntax

Mulberry's live-coding engine uses a pattern language inspired by TidalCycles and Strudel. Patterns describe sequences of events distributed over a **cycle** (one musical loop).

### Simple Sequences

A space-separated list of notes divides the cycle evenly:

```
"c4 e4 g4 c5"
```

This plays four notes, each taking ¼ of the cycle. At 120 BPM with a 1-bar cycle, each note lasts one beat.

### Subdivisions

Square brackets subdivide a single slot:

```
"c4 [e4 g4] b4 c5"
```

Here, `c4`, `b4`, and `c5` each take ¼ of the cycle. `e4` and `g4` share the second quarter, each getting ⅛.

Subdivisions can nest:

```
"c4 [e4 [g4 b4]] c5"
```

### Rests

Use `~` for silence:

```
"c4 ~ e4 ~"
```

Plays a note, silence, a note, silence — creating a rhythmic pattern.

### Repeats

Use `*N` to repeat a note N times within its slot:

```
"c4*3 e4"
```

`c4` plays three times in the first half, `e4` once in the second half.

### Transforms

Pipe a pattern into a transform with `|`:

```
"c4 e4 g4 c5" | fast 2
```

The pattern now plays twice per cycle (double speed).

```
"c4 e4 g4 c5" | slow 2
```

The pattern stretches over two cycles (half speed).

```
"c4 e4 g4 c5" | rev
```

Reverses the sequence: `"c5 g4 e4 c4"`.

### Chaining Transforms

Transforms can be chained:

```
"c4 e4 g4 c5" | fast 2 | rev
```

### Additional Transform Examples

```
"c4 e4 g4" | every 3 (fast 2)     -- every 3rd cycle, double speed
"c4 e4 g4" | jux rev              -- reverse in one stereo channel
"c4 e4 g4" | degrade 0.5          -- randomly drop 50% of events
```

### Polyphony

Use `<` and `>` to layer patterns (play simultaneously):

```
< "c4 e4 g4" , "e3 g3 b3" >
```

Both patterns play at the same time, creating harmony.

### Assigning Patterns to Tracks

```
d1 $ "c4 e4 g4 c5" | sound "piano"
d2 $ "bd ~ sd ~" | sound "drums"
d3 $ "c3 g3" | sound "bass" | slow 2
```

Each `d1`, `d2`, `d3` is an output track. `$` binds a pattern to a track. The `sound` transform selects the instrument.

### Practical Examples

**Simple beat:**
```
d1 $ "bd bd sd bd" | sound "drums"
```

**Arpeggiated chord with subdivision:**
```
d1 $ "c4 [e4 g4] c5 [g4 e4]" | sound "synth"
```

**Layered pattern with transform:**
```
d1 $ < "c4 e4 g4 c5" , "c3 ~ g3 ~" > | sound "piano" | fast 2
```

**Evolving pattern:**
```
d1 $ "c4 e4 g4 b4" | every 4 (fast 2) | every 3 (rev) | sound "keys"
```

---

## Error Handling

Mulberry uses a layered error strategy:

```rust
// Core error type
#[derive(Debug, thiserror::Error)]
pub enum MulberryError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("DSP error: {0}")]
    Dsp(#[from] DspError),

    #[error("Pattern parse error: {0}")]
    Pattern(#[from] PatternError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
}
```

- **In the audio callback:** Errors are never propagated. Log them via a lock-free ring buffer and output silence.
- **In async code:** Use `Result<T, MulberryError>` and the `?` operator.
- **In plugins:** Return `Result<(), PluginError>` from `init()`. Runtime errors in `process()` should be handled internally.
- **In live-code parsing:** Return user-friendly parse errors via the event bus so the REPL can display them.

---

## Configuration

Mulberry uses a TOML configuration file:

```toml
[audio]
sample_rate = 44100
buffer_size = 256
device = "default"

[transport]
bpm = 120.0
time_signature = [4, 4]

[livecode]
cycle_length = 1.0  # in bars

[agent]
model = "gpt-4"
max_concurrent = 4

[plugins]
search_paths = ["./plugins", "~/.mulberry/plugins"]
```

Configuration is loaded at startup and accessible through the `Config` struct in `mulberry-core`.
