//! Dynamic range compressor.

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Dynamic range compressor with attack/release envelope follower.
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack_coef: f32,
    release_coef: f32,
    makeup_gain: f32,
    envelope: f32,
    sample_rate: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        let attack_ms = 10.0_f32;
        let release_ms = 100.0_f32;
        Self {
            threshold: 0.5,
            ratio: 4.0,
            attack_coef: Self::coef(attack_ms, sample_rate),
            release_coef: Self::coef(release_ms, sample_rate),
            makeup_gain: 1.0,
            envelope: 0.0,
            sample_rate,
        }
    }

    fn coef(ms: f32, sample_rate: f32) -> f32 {
        (-1.0_f32 / (ms * sample_rate / 1000.0)).exp()
    }
}

impl Plugin for Compressor {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let level = in_s.abs();

            if level > self.envelope {
                self.envelope = self.attack_coef * (self.envelope - level) + level;
            } else {
                self.envelope = self.release_coef * (self.envelope - level) + level;
            }

            let gain_reduction = if self.envelope > self.threshold {
                let gain = self.threshold + (self.envelope - self.threshold) / self.ratio;
                if self.envelope > 1e-10 {
                    gain / self.envelope
                } else {
                    1.0
                }
            } else {
                1.0
            };

            *out_s = in_s * gain_reduction * self.makeup_gain;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.compressor",
            name: "Compressor",
            description: "Dynamic range compressor with attack/release envelope follower",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![
            PluginParameter {
                id: "threshold".into(),
                name: "Threshold".into(),
                value: self.threshold,
                min: 0.0,
                max: 1.0,
                default: 0.5,
            },
            PluginParameter {
                id: "ratio".into(),
                name: "Ratio".into(),
                value: self.ratio,
                min: 1.0,
                max: 20.0,
                default: 4.0,
            },
            PluginParameter {
                id: "attack_ms".into(),
                name: "Attack (ms)".into(),
                value: {
                    // recover attack_ms from coef: coef = exp(-1/(ms*sr/1000))
                    // ms = -1000 / (ln(coef) * sr)
                    -1000.0 / (self.attack_coef.ln() * self.sample_rate)
                },
                min: 0.1,
                max: 200.0,
                default: 10.0,
            },
            PluginParameter {
                id: "release_ms".into(),
                name: "Release (ms)".into(),
                value: -1000.0 / (self.release_coef.ln() * self.sample_rate),
                min: 1.0,
                max: 2000.0,
                default: 100.0,
            },
            PluginParameter {
                id: "makeup_gain".into(),
                name: "Makeup Gain".into(),
                value: self.makeup_gain,
                min: 0.0,
                max: 4.0,
                default: 1.0,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "threshold" => self.threshold = value.clamp(0.0, 1.0),
            "ratio" => self.ratio = value.max(1.0),
            "attack_ms" => self.attack_coef = Self::coef(value.max(0.01), self.sample_rate),
            "release_ms" => self.release_coef = Self::coef(value.max(0.01), self.sample_rate),
            "makeup_gain" => self.makeup_gain = value.max(0.0),
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "threshold" => Some(self.threshold),
            "ratio" => Some(self.ratio),
            "attack_ms" => Some(-1000.0 / (self.attack_coef.ln() * self.sample_rate)),
            "release_ms" => Some(-1000.0 / (self.release_coef.ln() * self.sample_rate)),
            "makeup_gain" => Some(self.makeup_gain),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let attack_ms = -1000.0 / (self.attack_coef.ln() * self.sample_rate);
        let release_ms = -1000.0 / (self.release_coef.ln() * self.sample_rate);
        self.sample_rate = sample_rate;
        self.attack_coef = Self::coef(attack_ms, sample_rate);
        self.release_coef = Self::coef(release_ms, sample_rate);
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_compression_below_threshold() {
        let mut comp = Compressor::new(44100.0);
        // threshold = 0.5; use a very small signal well below threshold
        let input: Vec<f32> = (0..256).map(|i| 0.1 * (i as f32 * 0.1).sin()).collect();
        let mut output = vec![0.0_f32; 256];
        comp.process_block(&input, &mut output);
        // makeup_gain=1.0; below threshold => gain_reduction ~1.0
        // Ratio test: output should closely match input (allow small envelope ramp)
        for (i, o) in input.iter().zip(output.iter()) {
            assert!(
                (i - o).abs() < 0.01,
                "expected pass-through below threshold, input={i}, output={o}"
            );
        }
    }
}
