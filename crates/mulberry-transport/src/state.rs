//! Transport state machine.
//!
//! The [`Transport`] struct is the central timing authority for the DAW.
//! It manages playback state, tempo, time signature, looping, and publishes
//! [`TransportEvent`]s on the [`EventBus`] whenever state changes.

use mulberry_core::event::{Event, EventBus, TransportEvent};
use mulberry_core::time::{MusicalPosition, Tempo, TickClock, TimeSignature};
use mulberry_core::MulberryConfig;
use serde::{Deserialize, Serialize};

// ── TransportState ──────────────────────────────────────────────────────────

/// The current playback state of the transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    /// Playback is stopped and the position is at the origin.
    Stopped,
    /// Playback is active and the clock is advancing.
    Playing,
    /// Playback is suspended but the position is retained.
    Paused,
}

// ── Transport ───────────────────────────────────────────────────────────────

// CONTEXT7 REVIEW:
// Issue: Transport state transitions and event publishing
// Resolution: Transport owns an EventBus clone and publishes events on state changes
// Why: Decouples transport from consumers; any subsystem can subscribe to transport events

/// The DAW transport — owns the canonical playback state, tempo, time
/// signature, clock, and loop points.
///
/// All mutations publish the corresponding [`TransportEvent`] on the
/// [`EventBus`] so that other subsystems can react without direct coupling.
pub struct Transport {
    state: TransportState,
    tempo: Tempo,
    time_signature: TimeSignature,
    clock: TickClock,
    loop_enabled: bool,
    loop_start: MusicalPosition,
    loop_end: MusicalPosition,
    event_bus: EventBus,
}

impl Transport {
    /// Creates a new [`Transport`] from the given configuration and event bus.
    pub fn new(config: &MulberryConfig, event_bus: EventBus) -> Self {
        let (ts_num, ts_den) = config.time_signature;
        Self {
            state: TransportState::Stopped,
            tempo: Tempo::new(config.initial_tempo),
            time_signature: TimeSignature::new(ts_num, ts_den),
            clock: TickClock::new(config.sample_rate),
            loop_enabled: false,
            loop_start: MusicalPosition::zero(),
            loop_end: MusicalPosition::zero(),
            event_bus,
        }
    }

    /// Starts playback.
    pub fn play(&mut self) {
        self.state = TransportState::Playing;
        self.event_bus
            .publish(Event::Transport(TransportEvent::Play));
    }

    /// Stops playback and resets the clock to the origin.
    pub fn stop(&mut self) {
        self.state = TransportState::Stopped;
        self.clock.reset();
        self.event_bus
            .publish(Event::Transport(TransportEvent::Stop));
    }

    /// Pauses playback without resetting the position.
    pub fn pause(&mut self) {
        self.state = TransportState::Paused;
        self.event_bus
            .publish(Event::Transport(TransportEvent::Pause));
    }

    /// Seeks to the given position expressed in total beats from the origin.
    ///
    /// The clock's sample position is recalculated from the beat position and
    /// a [`TransportEvent::Seek`] is published with the equivalent time in
    /// seconds.
    pub fn seek(&mut self, position_beats: f64) {
        let samples_per_beat = self.tempo.samples_per_beat(self.clock.sample_rate);
        self.clock.sample_position = (position_beats * samples_per_beat) as u64;

        let position_secs = position_beats * self.tempo.beat_duration_secs();
        self.event_bus
            .publish(Event::Transport(TransportEvent::Seek(position_secs)));
    }

    /// Updates the tempo (BPM) and publishes a [`TransportEvent::SetTempo`].
    pub fn set_tempo(&mut self, bpm: f64) {
        self.tempo = Tempo::new(bpm);
        self.event_bus
            .publish(Event::Transport(TransportEvent::SetTempo(bpm)));
    }

    /// Updates the time signature and publishes a
    /// [`TransportEvent::SetTimeSignature`].
    pub fn set_time_signature(&mut self, num: u8, den: u8) {
        self.time_signature = TimeSignature::new(num, den);
        self.event_bus
            .publish(Event::Transport(TransportEvent::SetTimeSignature(num, den)));
    }

    /// Configures the loop region.
    pub fn set_loop(&mut self, enabled: bool, start: MusicalPosition, end: MusicalPosition) {
        self.loop_enabled = enabled;
        self.loop_start = start;
        self.loop_end = end;
    }

    /// Advances the clock by `num_samples` audio frames when playing.
    ///
    /// If looping is enabled and the position passes `loop_end`, the clock
    /// wraps back to `loop_start`.
    pub fn advance(&mut self, num_samples: u32) {
        if self.state != TransportState::Playing {
            return;
        }

        self.clock.advance(num_samples as u64);

        if self.loop_enabled {
            let pos = self
                .clock
                .musical_position(&self.tempo, &self.time_signature);
            let end_beats = self.loop_end.total_beats(&self.time_signature);
            let current_beats = pos.total_beats(&self.time_signature);

            if end_beats > 0.0 && current_beats >= end_beats {
                let start_beats = self.loop_start.total_beats(&self.time_signature);
                let overshoot = current_beats - end_beats;
                let target_beats = start_beats + overshoot;
                let samples_per_beat = self.tempo.samples_per_beat(self.clock.sample_rate);
                self.clock.sample_position = (target_beats * samples_per_beat) as u64;
            }
        }
    }

    /// Returns the current [`TransportState`].
    pub fn state(&self) -> TransportState {
        self.state
    }

    /// Returns the current musical position.
    pub fn position(&self) -> MusicalPosition {
        self.clock
            .musical_position(&self.tempo, &self.time_signature)
    }

    /// Returns the current position in seconds.
    pub fn position_secs(&self) -> f64 {
        self.clock.position_secs()
    }

    /// Returns a reference to the current [`Tempo`].
    pub fn tempo(&self) -> &Tempo {
        &self.tempo
    }

    /// Returns a reference to the current [`TimeSignature`].
    pub fn time_signature(&self) -> &TimeSignature {
        &self.time_signature
    }

    /// Returns `true` if the transport is currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == TransportState::Playing
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_transport() -> (Transport, crossbeam_channel::Receiver<Event>) {
        let config = MulberryConfig::default();
        let bus = EventBus::new();
        let rx = bus.subscribe();
        let transport = Transport::new(&config, bus);
        (transport, rx)
    }

    #[test]
    fn initial_state_is_stopped() {
        let (transport, _rx) = default_transport();
        assert_eq!(transport.state(), TransportState::Stopped);
        assert!(!transport.is_playing());
    }

    #[test]
    fn play_sets_state_and_publishes_event() {
        let (mut transport, rx) = default_transport();
        transport.play();

        assert_eq!(transport.state(), TransportState::Playing);
        assert!(transport.is_playing());

        let event = rx.try_recv().expect("expected Play event");
        assert!(matches!(event, Event::Transport(TransportEvent::Play)));
    }

    #[test]
    fn stop_resets_clock_and_publishes_event() {
        let (mut transport, rx) = default_transport();
        transport.play();
        transport.advance(44100); // advance 1 second
        transport.stop();

        assert_eq!(transport.state(), TransportState::Stopped);
        assert_eq!(transport.position_secs(), 0.0);

        // Drain Play event, then check Stop
        let _play = rx.try_recv().unwrap();
        let event = rx.try_recv().expect("expected Stop event");
        assert!(matches!(event, Event::Transport(TransportEvent::Stop)));
    }

    #[test]
    fn pause_preserves_position() {
        let (mut transport, rx) = default_transport();
        transport.play();
        transport.advance(44100);
        let pos_before = transport.position_secs();
        transport.pause();

        assert_eq!(transport.state(), TransportState::Paused);
        assert_eq!(transport.position_secs(), pos_before);

        let _play = rx.try_recv().unwrap();
        let event = rx.try_recv().expect("expected Pause event");
        assert!(matches!(event, Event::Transport(TransportEvent::Pause)));
    }

    #[test]
    fn seek_updates_position() {
        let (mut transport, rx) = default_transport();
        // At 120 BPM, 4 beats = 2 seconds
        transport.seek(4.0);

        let pos = transport.position();
        assert_eq!(pos.bar, 1); // 4 beats in 4/4 = 1 full bar
        assert!((pos.beat - 0.0).abs() < 1e-9);

        let event = rx.try_recv().expect("expected Seek event");
        assert!(matches!(event, Event::Transport(TransportEvent::Seek(_))));
    }

    #[test]
    fn set_tempo_publishes_event() {
        let (mut transport, rx) = default_transport();
        transport.set_tempo(140.0);

        assert_eq!(transport.tempo().bpm, 140.0);

        let event = rx.try_recv().expect("expected SetTempo event");
        assert!(matches!(
            event,
            Event::Transport(TransportEvent::SetTempo(bpm)) if (bpm - 140.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn set_time_signature_publishes_event() {
        let (mut transport, rx) = default_transport();
        transport.set_time_signature(3, 4);

        assert_eq!(transport.time_signature().numerator, 3);
        assert_eq!(transport.time_signature().denominator, 4);

        let event = rx.try_recv().expect("expected SetTimeSignature event");
        assert!(matches!(
            event,
            Event::Transport(TransportEvent::SetTimeSignature(3, 4))
        ));
    }

    #[test]
    fn advance_does_not_move_when_stopped() {
        let (mut transport, _rx) = default_transport();
        transport.advance(44100);
        assert_eq!(transport.position_secs(), 0.0);
    }

    #[test]
    fn advance_does_not_move_when_paused() {
        let (mut transport, _rx) = default_transport();
        transport.play();
        transport.advance(44100);
        transport.pause();
        let pos = transport.position_secs();
        transport.advance(44100);
        assert_eq!(transport.position_secs(), pos);
    }

    #[test]
    fn loop_wraps_position() {
        let (mut transport, _rx) = default_transport();

        // Loop between bar 0 and bar 2 (8 beats in 4/4)
        let start = MusicalPosition::zero();
        let end = MusicalPosition { bar: 2, beat: 0.0 };
        transport.set_loop(true, start, end);

        transport.play();
        // At 120 BPM, 44100 SR: 1 beat = 22050 samples, 8 beats = 176400 samples
        // Advance past the loop end
        transport.advance(200_000);

        let pos = transport.position();
        let total = pos.total_beats(&transport.time_signature);
        // Should have wrapped back into [0, 8)
        assert!(total < 8.0, "position should have looped: total={total}");
    }
}
