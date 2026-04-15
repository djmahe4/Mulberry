use crate::engine::{TtsEngine, TtsRequest, TtsResult};
use crate::error::TtsError;
use crate::phoneme::text_to_phonemes;

/// A simple formant synthesizer that generates vowel-like sounds by mixing
/// two sine oscillators at formant frequencies (F1, F2) with an amplitude
/// envelope.
pub struct FormantSynthesizer {
    #[allow(dead_code)]
    sample_rate: u32,
}

impl FormantSynthesizer {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
}

impl TtsEngine for FormantSynthesizer {
    fn synthesize(&self, request: &TtsRequest) -> Result<TtsResult, TtsError> {
        if request.text.is_empty() {
            return Err(TtsError::InvalidInput("empty text".to_string()));
        }

        let sample_rate = request.sample_rate;
        let phonemes = text_to_phonemes(&request.text);

        // Each phoneme lasts ~0.15 seconds, adjusted by speed
        let phoneme_duration = 0.15 / request.speed;
        let samples_per_phoneme = (phoneme_duration * sample_rate as f32) as usize;
        let total_samples = samples_per_phoneme * phonemes.len();

        let mut samples = Vec::with_capacity(total_samples);
        let base_pitch = 150.0 * request.pitch; // Base fundamental ~150 Hz

        for phoneme in &phonemes {
            let (f1, f2) = phoneme.formants();

            for i in 0..samples_per_phoneme {
                let t = i as f32 / sample_rate as f32;

                if f1 == 0.0 && f2 == 0.0 {
                    // Silence
                    samples.push(0.0);
                    continue;
                }

                // Simple amplitude envelope (attack-sustain-release)
                let env_pos = i as f32 / samples_per_phoneme as f32;
                let envelope = if env_pos < 0.1 {
                    env_pos / 0.1 // attack
                } else if env_pos > 0.8 {
                    (1.0 - env_pos) / 0.2 // release
                } else {
                    1.0 // sustain
                };

                // Mix fundamental with two formants
                let fundamental =
                    (2.0 * std::f32::consts::PI * base_pitch * t).sin() * 0.3;
                let formant1 = (2.0 * std::f32::consts::PI * f1 * t).sin() * 0.4;
                let formant2 = (2.0 * std::f32::consts::PI * f2 * t).sin() * 0.3;

                let sample = (fundamental + formant1 + formant2) * envelope * 0.5;
                samples.push(sample.clamp(-1.0, 1.0));
            }
        }

        let duration_secs = samples.len() as f32 / sample_rate as f32;

        Ok(TtsResult {
            samples,
            sample_rate,
            duration_secs,
        })
    }

    fn name(&self) -> &str {
        "Mulberry Formant Synth"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formant_synth_basic() {
        let synth = FormantSynthesizer::new(44100);
        assert!(synth.is_available());
        assert_eq!(synth.name(), "Mulberry Formant Synth");
    }

    #[test]
    fn test_formant_synth_produces_audio() {
        let synth = FormantSynthesizer::new(44100);
        let request = TtsRequest {
            text: "hello".to_string(),
            speed: 1.0,
            pitch: 1.0,
            sample_rate: 44100,
        };
        let result = synth.synthesize(&request).unwrap();
        assert!(!result.samples.is_empty());
        assert_eq!(result.sample_rate, 44100);
        assert!(result.duration_secs > 0.0);
    }

    #[test]
    fn test_formant_synth_empty_text() {
        let synth = FormantSynthesizer::new(44100);
        let request = TtsRequest {
            text: String::new(),
            speed: 1.0,
            pitch: 1.0,
            sample_rate: 44100,
        };
        assert!(synth.synthesize(&request).is_err());
    }

    #[test]
    fn test_formant_synth_samples_in_range() {
        let synth = FormantSynthesizer::new(44100);
        let request = TtsRequest {
            text: "aeiou".to_string(),
            speed: 1.0,
            pitch: 1.0,
            sample_rate: 44100,
        };
        let result = synth.synthesize(&request).unwrap();
        for sample in &result.samples {
            assert!(
                *sample >= -1.0 && *sample <= 1.0,
                "sample out of range: {}",
                sample
            );
        }
    }

    #[test]
    fn test_formant_synth_speed_changes_duration() {
        let synth = FormantSynthesizer::new(44100);
        let normal = synth
            .synthesize(&TtsRequest {
                text: "aeiou".to_string(),
                speed: 1.0,
                pitch: 1.0,
                sample_rate: 44100,
            })
            .unwrap();
        let fast = synth
            .synthesize(&TtsRequest {
                text: "aeiou".to_string(),
                speed: 2.0,
                pitch: 1.0,
                sample_rate: 44100,
            })
            .unwrap();
        assert!(fast.duration_secs < normal.duration_secs);
    }
}
