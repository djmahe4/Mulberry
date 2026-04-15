//! Event bus for inter-component communication.
//!
//! The [`EventBus`] provides a broadcast-style publish/subscribe mechanism so
//! that any Mulberry subsystem can emit or react to [`Event`]s without direct
//! coupling.

use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ── TransportEvent ──────────────────────────────────────────────────────────

/// Events related to transport / playback control.
// CONTEXT7 REVIEW: Transport events are kept separate from the top-level
// `Event` enum so that transport-specific logic can pattern-match without
// caring about unrelated variants.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransportEvent {
    /// Start playback.
    Play,
    /// Stop playback and reset position.
    Stop,
    /// Pause playback without resetting position.
    Pause,
    /// Seek to a position in seconds.
    Seek(f64),
    /// Change the tempo (BPM).
    SetTempo(f64),
    /// Change the time signature (numerator, denominator).
    SetTimeSignature(u8, u8),
}

// ── Event ───────────────────────────────────────────────────────────────────

/// A top-level event that can be published on the [`EventBus`].
///
/// Every subsystem in Mulberry communicates through this unified event type.
// CONTEXT7 REVIEW: A single event enum keeps the event bus generic and avoids
// the need for trait-object erasure. New subsystem events should be added as
// variants here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// A transport / playback control event.
    Transport(TransportEvent),

    /// A MIDI-style note-on event.
    NoteOn {
        /// MIDI channel (0–15).
        channel: u8,
        /// MIDI note number (0–127).
        note: u8,
        /// Velocity in the range `[0.0, 1.0]`.
        velocity: f32,
    },

    /// A MIDI-style note-off event.
    NoteOff {
        /// MIDI channel (0–15).
        channel: u8,
        /// MIDI note number (0–127).
        note: u8,
    },

    /// A parameter change targeting a named component and parameter.
    ParameterChange {
        /// The identifier of the target component.
        target: String,
        /// The name of the parameter to change.
        param: String,
        /// The new parameter value.
        value: f64,
    },

    /// A request to evaluate a live-coding expression.
    LiveCodeEval(String),

    /// A message originating from an AI agent.
    AgentMessage {
        /// The unique identifier of the agent.
        agent_id: String,
        /// The message payload.
        content: String,
    },

    /// An opaque event forwarded to / from a plugin.
    PluginEvent {
        /// The unique identifier of the plugin.
        plugin_id: String,
        /// Arbitrary JSON payload.
        data: serde_json::Value,
    },

    /// A drum hit from the drum machine.
    DrumHit(crate::drums::DrumHit),

    /// Drum machine state update.
    DrumMachineState(crate::drums::DrumMachineState),

    /// A graceful shutdown request.
    Shutdown,
}

// ── EventBus ────────────────────────────────────────────────────────────────

/// A broadcast-style event bus.
///
/// Subscribers each receive their own [`crossbeam_channel::Receiver`]. When an
/// event is published, it is cloned and sent to every active subscriber.
// CONTEXT7 REVIEW: Using `crossbeam_channel` bounded channels + `RwLock`
// provides a lock-free fast-path for reads (most operations) while keeping
// the subscriber list safely mutable.
pub struct EventBus {
    subscribers: RwLock<Vec<Sender<Event>>>,
}

impl EventBus {
    /// Creates a new, empty [`EventBus`].
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Registers a new subscriber and returns a [`Receiver`] that will
    /// receive all future events.
    ///
    /// The internal channel is unbounded so that slow consumers do not block
    /// the publisher.
    pub fn subscribe(&self) -> Receiver<Event> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.subscribers.write().push(tx);
        rx
    }

    /// Publishes an event to **all** subscribers.
    ///
    /// Disconnected subscribers are automatically pruned.
    pub fn publish(&self, event: Event) {
        let mut subs = self.subscribers.write();
        subs.retain(|tx| tx.send(event.clone()).is_ok());
    }

    /// Attempts to publish an event without blocking.
    ///
    /// Because the channels are unbounded this behaves identically to
    /// [`publish`](Self::publish), but the name mirrors the `try_send`
    /// convention for clarity.
    pub fn try_publish(&self, event: Event) {
        let mut subs = self.subscribers.write();
        subs.retain(|tx| tx.try_send(event.clone()).is_ok());
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
