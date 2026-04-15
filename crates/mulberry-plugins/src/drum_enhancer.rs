//! Drum enhancer: transient shaper + sub-bass synthesiser.

use std::f32::consts::PI;

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Enhances transients and adds sub-bass punch to drum sounds.
pub struct DrumEnhancer {
    transient_gain: f32,
    sub_freq: f32,
    sub_gain: f32,
    sub_osc_phase: f32,
    envelope: f32,
    sample_rate: f32,
}

impl DrumEnhancer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            transient_gain: 1.0,
            sub_freq: 60.0,
            sub_gain: 0.3,
            sub_osc_phase: 0.0,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl Plugin for DrumEnhancer {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let current_env = in_s.abs();
            let transient = (current_env - self.envelope).max(0.0) * self.transient_gain;
            self.envelope = 0.99 * self.envelope + 0.01 * current_env;

            let sub_sample = (self.sub_osc_phase * 2.0 * PI).sin() * self.sub_gain * current_env;
            self.sub_osc_phase += self.sub_freq / self.sample_rate;
            self.sub_osc_phase %= 1.0;

            *out_s = in_s * (1.0 + transient) + sub_sample;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.drum_enhancer",
            name: "Drum Enhancer",
            description: "Transient shaper and sub-bass synthesiser for drums",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![
            PluginParameter {
                id: "transient_gain".into(),
                name: "Transient Gain".into(),
                value: self.transient_gain,
                min: 0.0,
                max: 10.0,
                default: 1.0,
            },
            PluginParameter {
                id: "sub_freq".into(),
                name: "Sub Frequency (Hz)".into(),
                value: self.sub_freq,
                min: 20.0,
                max: 200.0,
                default: 60.0,
            },
            PluginParameter {
                id: "sub_gain".into(),
                name: "Sub Gain".into(),
                value: self.sub_gain,
                min: 0.0,
                max: 1.0,
                default: 0.3,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "transient_gain" => self.transient_gain = value.max(0.0),
            "sub_freq" => self.sub_freq = value.clamp(20.0, 20000.0),
            "sub_gain" => self.sub_gain = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "transient_gain" => Some(self.transient_gain),
            "sub_freq" => Some(self.sub_freq),
            "sub_gain" => Some(self.sub_gain),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.sub_osc_phase = 0.0;
    }
}
