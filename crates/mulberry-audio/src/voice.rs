use mulberry_dsp::{AdsrEnvelope, Oscillator, Waveform};

/// Parameters for constructing a [`Voice`].
#[derive(Debug, Clone)]
pub struct VoiceParams {
    pub waveform: Waveform,
    pub frequency: f32,
    pub amplitude: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

/// A Voice combines an [`Oscillator`] with an [`AdsrEnvelope`] to produce a
/// single pitched sound that responds to note-on / note-off events.
pub struct Voice {
    oscillator: Oscillator,
    envelope: AdsrEnvelope,
    active: bool,
}

impl Voice {
    /// Create a new voice from the given parameters and sample rate.
    pub fn new(params: &VoiceParams, sample_rate: f32) -> Self {
        let mut oscillator = Oscillator::new(params.waveform, params.frequency, sample_rate);
        oscillator.set_amplitude(params.amplitude);

        let envelope = AdsrEnvelope::new(
            params.attack,
            params.decay,
            params.sustain,
            params.release,
            sample_rate,
        );

        Self {
            oscillator,
            envelope,
            active: false,
        }
    }

    /// Trigger the voice (start the attack phase of the envelope).
    pub fn trigger(&mut self) {
        self.envelope.trigger();
        self.active = true;
    }

    /// Begin the release phase of the envelope.
    pub fn release(&mut self) {
        self.envelope.release_note();
    }

    /// Generate the next sample: oscillator output scaled by the envelope.
    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let osc_sample = self.oscillator.next_sample();
        let env_value = self.envelope.next_sample();

        if !self.envelope.is_active() {
            self.active = false;
        }

        osc_sample * env_value
    }

    /// Returns `true` while the voice is still producing sound.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Change the oscillator frequency.
    pub fn set_frequency(&mut self, freq: f32) {
        self.oscillator.set_frequency(freq);
    }
}
