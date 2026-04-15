//! ADSR (Attack-Decay-Sustain-Release) envelope generator.
//!
//! Produces a gain envelope that can be multiplied with an audio signal to
//! shape its amplitude over time.

use mulberry_core::Sample;

/// The current stage of an ADSR envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// A linear ADSR envelope generator.
///
// CONTEXT7 REVIEW: Linear ramps vs exponential
// This initial implementation (v0.1) uses linear ramps for simplicity and
// deterministic behaviour.  Exponential attack/decay/release curves sound
// more natural for most instruments and are planned for a future version
// behind a configurable `CurveType` parameter.
pub struct AdsrEnvelope {
    /// Attack time in seconds.
    attack: f32,
    /// Decay time in seconds.
    decay: f32,
    /// Sustain level (`0.0..=1.0`).
    sustain: f32,
    /// Release time in seconds.
    release: f32,
    /// Output sample rate in Hz.
    sample_rate: f32,

    // ── internal state ──
    stage: EnvelopeStage,
    /// Current envelope level.
    level: f32,
    /// Number of samples elapsed in the current stage.
    stage_samples: u64,
    /// Level captured at the start of the Release stage.
    release_start_level: f32,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope.
    ///
    /// * `attack`      – attack time in seconds.
    /// * `decay`       – decay time in seconds.
    /// * `sustain`     – sustain level (`0.0..=1.0`).
    /// * `release`     – release time in seconds.
    /// * `sample_rate` – output sample rate in Hz.
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
        Self {
            attack,
            decay,
            sustain: sustain.clamp(0.0, 1.0),
            release,
            sample_rate,
            stage: EnvelopeStage::Idle,
            level: 0.0,
            stage_samples: 0,
            release_start_level: 0.0,
        }
    }

    /// Triggers the envelope, starting (or restarting) from the Attack stage.
    pub fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
        self.level = 0.0;
        self.stage_samples = 0;
    }

    /// Begins the Release stage from the current level.
    pub fn release_note(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.release_start_level = self.level;
            self.stage = EnvelopeStage::Release;
            self.stage_samples = 0;
        }
    }

    /// Returns the next envelope value and advances the internal state.
    pub fn next_sample(&mut self) -> Sample {
        match self.stage {
            EnvelopeStage::Idle => 0.0,

            EnvelopeStage::Attack => {
                let total = (self.attack * self.sample_rate) as u64;
                if total == 0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                    self.stage_samples = 0;
                } else {
                    self.level = self.stage_samples as f32 / total as f32;
                    self.stage_samples += 1;
                    if self.stage_samples >= total {
                        self.level = 1.0;
                        self.stage = EnvelopeStage::Decay;
                        self.stage_samples = 0;
                    }
                }
                self.level
            }

            EnvelopeStage::Decay => {
                let total = (self.decay * self.sample_rate) as u64;
                if total == 0 {
                    self.level = self.sustain;
                    self.stage = EnvelopeStage::Sustain;
                    self.stage_samples = 0;
                } else {
                    let t = self.stage_samples as f32 / total as f32;
                    self.level = 1.0 + (self.sustain - 1.0) * t;
                    self.stage_samples += 1;
                    if self.stage_samples >= total {
                        self.level = self.sustain;
                        self.stage = EnvelopeStage::Sustain;
                        self.stage_samples = 0;
                    }
                }
                self.level
            }

            EnvelopeStage::Sustain => {
                self.level = self.sustain;
                self.level
            }

            EnvelopeStage::Release => {
                let total = (self.release * self.sample_rate) as u64;
                if total == 0 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                    self.stage_samples = 0;
                } else {
                    let t = self.stage_samples as f32 / total as f32;
                    self.level = self.release_start_level * (1.0 - t);
                    self.stage_samples += 1;
                    if self.stage_samples >= total {
                        self.level = 0.0;
                        self.stage = EnvelopeStage::Idle;
                        self.stage_samples = 0;
                    }
                }
                self.level
            }
        }
    }

    /// Returns `true` if the envelope is producing a non-zero signal.
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    /// Returns the current [`EnvelopeStage`].
    pub fn stage(&self) -> EnvelopeStage {
        self.stage
    }

    /// Resets the envelope to [`EnvelopeStage::Idle`].
    pub fn reset(&mut self) {
        self.stage = EnvelopeStage::Idle;
        self.level = 0.0;
        self.stage_samples = 0;
        self.release_start_level = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_returns_zero() {
        let mut env = AdsrEnvelope::new(0.01, 0.01, 0.5, 0.01, 44100.0);
        assert_eq!(env.next_sample(), 0.0);
        assert!(!env.is_active());
    }

    #[test]
    fn trigger_starts_attack() {
        let mut env = AdsrEnvelope::new(0.01, 0.01, 0.5, 0.01, 44100.0);
        env.trigger();
        assert_eq!(env.stage(), EnvelopeStage::Attack);
        assert!(env.is_active());
    }

    #[test]
    fn full_cycle() {
        // Very short envelope so we can iterate through all stages.
        let sr = 100.0;
        let mut env = AdsrEnvelope::new(0.01, 0.01, 0.5, 0.01, sr);
        env.trigger();

        // Consume attack (1 sample at sr=100, attack=0.01s)
        let mut last = 0.0_f32;
        while env.stage() == EnvelopeStage::Attack {
            last = env.next_sample();
        }
        assert!(last > 0.0);

        // Consume decay
        while env.stage() == EnvelopeStage::Decay {
            last = env.next_sample();
        }
        assert!((last - 0.5).abs() < 0.2);

        // Sustain holds
        assert_eq!(env.stage(), EnvelopeStage::Sustain);
        let s = env.next_sample();
        assert!((s - 0.5).abs() < 1e-6);

        // Release
        env.release_note();
        while env.is_active() {
            last = env.next_sample();
        }
        assert_eq!(last, 0.0);
    }
}
