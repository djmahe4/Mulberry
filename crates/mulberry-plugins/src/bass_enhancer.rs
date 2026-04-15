//! Bass enhancer: sub-bass boost + harmonic exciter.

use mulberry_dsp::{BiquadFilter, FilterType};

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Adds harmonic enhancement and sub-bass to bass sounds.
pub struct BassEnhancer {
    sub_filter: BiquadFilter,
    harmonic_filter: BiquadFilter,
    sub_gain: f32,
    harmonic_gain: f32,
    sub_freq: f32,
    sample_rate: f32,
}

impl BassEnhancer {
    pub fn new(sample_rate: f32) -> Self {
        let sub_freq = 80.0_f32;
        Self {
            sub_filter: BiquadFilter::new(FilterType::LowPass, sub_freq, 0.707, sample_rate),
            harmonic_filter: BiquadFilter::new(FilterType::BandPass, sub_freq * 2.0, 0.707, sample_rate),
            sub_gain: 0.3,
            harmonic_gain: 0.2,
            sub_freq,
            sample_rate,
        }
    }
}

impl Plugin for BassEnhancer {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let sub = self.sub_filter.process_sample(*in_s) * self.sub_gain;
            let harmonic_source = self.harmonic_filter.process_sample(*in_s);
            // Second harmonic: soft waveshaping
            let harmonic = harmonic_source * harmonic_source * harmonic_source.signum() * self.harmonic_gain;
            *out_s = in_s + sub + harmonic;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.bass_enhancer",
            name: "Bass Enhancer",
            description: "Sub-bass boost and harmonic exciter for bass sounds",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![
            PluginParameter {
                id: "sub_gain".into(),
                name: "Sub Gain".into(),
                value: self.sub_gain,
                min: 0.0,
                max: 1.0,
                default: 0.3,
            },
            PluginParameter {
                id: "harmonic_gain".into(),
                name: "Harmonic Gain".into(),
                value: self.harmonic_gain,
                min: 0.0,
                max: 1.0,
                default: 0.2,
            },
            PluginParameter {
                id: "sub_freq".into(),
                name: "Sub Frequency (Hz)".into(),
                value: self.sub_freq,
                min: 20.0,
                max: 500.0,
                default: 80.0,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "sub_gain" => self.sub_gain = value.clamp(0.0, 1.0),
            "harmonic_gain" => self.harmonic_gain = value.clamp(0.0, 1.0),
            "sub_freq" => {
                self.sub_freq = value.clamp(20.0, 500.0);
                self.sub_filter =
                    BiquadFilter::new(FilterType::LowPass, self.sub_freq, 0.707, self.sample_rate);
                self.harmonic_filter = BiquadFilter::new(
                    FilterType::BandPass,
                    self.sub_freq * 2.0,
                    0.707,
                    self.sample_rate,
                );
            }
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "sub_gain" => Some(self.sub_gain),
            "harmonic_gain" => Some(self.harmonic_gain),
            "sub_freq" => Some(self.sub_freq),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sub_filter =
            BiquadFilter::new(FilterType::LowPass, self.sub_freq, 0.707, sample_rate);
        self.harmonic_filter = BiquadFilter::new(
            FilterType::BandPass,
            self.sub_freq * 2.0,
            0.707,
            sample_rate,
        );
    }

    fn reset(&mut self) {
        self.sub_filter.reset();
        self.harmonic_filter.reset();
    }
}
