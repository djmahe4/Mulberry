//! Monophonic synth bass synthesizer.

use mulberry_dsp::{AdsrEnvelope, BiquadFilter, FilterType, Oscillator, Waveform};

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Parameters for the SynthBass module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SynthBassParams {
    pub waveform: Waveform,
    pub cutoff: f32,
    pub resonance: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub distortion: f32,
}

impl Default for SynthBassParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sawtooth,
            cutoff: 800.0,
            resonance: 0.707,
            attack: 0.005,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            distortion: 0.0,
        }
    }
}

/// Monophonic bass synthesizer with oscillator, filter, ADSR, and distortion.
pub struct SynthBass {
    oscillator: Oscillator,
    filter: BiquadFilter,
    envelope: AdsrEnvelope,
    params: SynthBassParams,
    sample_rate: f32,
}

impl SynthBass {
    pub fn new(sample_rate: f32) -> Self {
        let params = SynthBassParams::default();
        Self {
            oscillator: Oscillator::new(params.waveform, 110.0, sample_rate),
            filter: BiquadFilter::new(FilterType::LowPass, params.cutoff, params.resonance, sample_rate),
            envelope: AdsrEnvelope::new(
                params.attack,
                params.decay,
                params.sustain,
                params.release,
                sample_rate,
            ),
            params,
            sample_rate,
        }
    }

    /// Trigger a note-on event. `note` is a MIDI note number (0–127).
    pub fn note_on(&mut self, note: u8, _velocity: f32) {
        let freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        self.oscillator.set_frequency(freq);
        self.envelope.trigger();
    }

    /// Begin the release stage.
    pub fn note_off(&mut self) {
        self.envelope.release_note();
    }

    /// Generate the next output sample.
    pub fn next_sample(&mut self) -> f32 {
        let env = self.envelope.next_sample();
        let osc = self.oscillator.next_sample();
        let raw = self.filter.process_sample(osc * env);
        // Soft-clip distortion via tanh
        raw.tanh() * (1.0 + self.params.distortion * 9.0).tanh()
    }

    /// Returns `true` while the envelope is producing output.
    pub fn is_active(&self) -> bool {
        self.envelope.is_active()
    }

    /// Replace all params and reinitialise DSP components.
    pub fn set_params(&mut self, params: SynthBassParams) {
        self.oscillator.set_waveform(params.waveform);
        self.filter = BiquadFilter::new(
            FilterType::LowPass,
            params.cutoff,
            params.resonance,
            self.sample_rate,
        );
        self.envelope = AdsrEnvelope::new(
            params.attack,
            params.decay,
            params.sustain,
            params.release,
            self.sample_rate,
        );
        self.params = params;
    }
}

impl Plugin for SynthBass {
    fn process_block(&mut self, _input: &[f32], output: &mut [f32]) {
        // Generator: input is ignored.
        for out_s in output.iter_mut() {
            *out_s = self.next_sample();
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.synth_bass",
            name: "Synth Bass",
            description: "Monophonic bass synthesizer with ADSR, filter, and distortion",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![
            PluginParameter {
                id: "cutoff".into(),
                name: "Filter Cutoff (Hz)".into(),
                value: self.params.cutoff,
                min: 20.0,
                max: 20000.0,
                default: 800.0,
            },
            PluginParameter {
                id: "resonance".into(),
                name: "Resonance".into(),
                value: self.params.resonance,
                min: 0.1,
                max: 10.0,
                default: 0.707,
            },
            PluginParameter {
                id: "attack".into(),
                name: "Attack (s)".into(),
                value: self.params.attack,
                min: 0.001,
                max: 4.0,
                default: 0.005,
            },
            PluginParameter {
                id: "decay".into(),
                name: "Decay (s)".into(),
                value: self.params.decay,
                min: 0.001,
                max: 4.0,
                default: 0.1,
            },
            PluginParameter {
                id: "sustain".into(),
                name: "Sustain".into(),
                value: self.params.sustain,
                min: 0.0,
                max: 1.0,
                default: 0.7,
            },
            PluginParameter {
                id: "release".into(),
                name: "Release (s)".into(),
                value: self.params.release,
                min: 0.001,
                max: 10.0,
                default: 0.2,
            },
            PluginParameter {
                id: "distortion".into(),
                name: "Distortion".into(),
                value: self.params.distortion,
                min: 0.0,
                max: 1.0,
                default: 0.0,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "cutoff" => {
                self.params.cutoff = value.clamp(20.0, 20000.0);
                self.filter = BiquadFilter::new(
                    FilterType::LowPass,
                    self.params.cutoff,
                    self.params.resonance,
                    self.sample_rate,
                );
            }
            "resonance" => {
                self.params.resonance = value.max(0.1);
                self.filter = BiquadFilter::new(
                    FilterType::LowPass,
                    self.params.cutoff,
                    self.params.resonance,
                    self.sample_rate,
                );
            }
            "attack" => {
                self.params.attack = value.max(0.001);
                self.envelope = AdsrEnvelope::new(
                    self.params.attack,
                    self.params.decay,
                    self.params.sustain,
                    self.params.release,
                    self.sample_rate,
                );
            }
            "decay" => {
                self.params.decay = value.max(0.001);
                self.envelope = AdsrEnvelope::new(
                    self.params.attack,
                    self.params.decay,
                    self.params.sustain,
                    self.params.release,
                    self.sample_rate,
                );
            }
            "sustain" => {
                self.params.sustain = value.clamp(0.0, 1.0);
                self.envelope = AdsrEnvelope::new(
                    self.params.attack,
                    self.params.decay,
                    self.params.sustain,
                    self.params.release,
                    self.sample_rate,
                );
            }
            "release" => {
                self.params.release = value.max(0.001);
                self.envelope = AdsrEnvelope::new(
                    self.params.attack,
                    self.params.decay,
                    self.params.sustain,
                    self.params.release,
                    self.sample_rate,
                );
            }
            "distortion" => self.params.distortion = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "cutoff" => Some(self.params.cutoff),
            "resonance" => Some(self.params.resonance),
            "attack" => Some(self.params.attack),
            "decay" => Some(self.params.decay),
            "sustain" => Some(self.params.sustain),
            "release" => Some(self.params.release),
            "distortion" => Some(self.params.distortion),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.filter = BiquadFilter::new(
            FilterType::LowPass,
            self.params.cutoff,
            self.params.resonance,
            sample_rate,
        );
        self.envelope = AdsrEnvelope::new(
            self.params.attack,
            self.params.decay,
            self.params.sustain,
            self.params.release,
            sample_rate,
        );
    }

    fn reset(&mut self) {
        self.envelope.reset();
        self.filter.reset();
        self.oscillator.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_off_and_active() {
        let mut bass = SynthBass::new(44100.0);
        assert!(!bass.is_active());
        bass.note_on(36, 1.0); // C2
        assert!(bass.is_active());

        // Should produce non-zero samples while active
        let s = bass.next_sample();
        // After note_on the envelope is in attack; sample should be near 0 initially
        // but definitely should not panic
        let _ = s;

        bass.note_off();
        // Still active during release
        assert!(bass.is_active());
    }

    #[test]
    fn next_sample_non_zero_in_sustain() {
        let mut bass = SynthBass::new(44100.0);
        bass.note_on(36, 1.0);
        // Burn through attack + decay (attack=5ms, decay=100ms at 44100 → ~4631 samples)
        for _ in 0..5000 {
            bass.next_sample();
        }
        // Now in sustain; sample should be non-zero
        let s = bass.next_sample();
        assert!(s.abs() > 1e-4, "expected non-zero output in sustain, got {s}");
    }
}
