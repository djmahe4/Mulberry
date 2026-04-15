//! Stereo-style delay effect with feedback.

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Simple delay line with feedback.
pub struct Delay {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    wet: f32,
    dry: f32,
    sample_rate: f32,
}

impl Delay {
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (sample_rate * 2.0) as usize;
        let delay_samples = (sample_rate * 0.5) as usize; // 500 ms default
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            delay_samples,
            feedback: 0.3,
            wet: 0.3,
            dry: 0.7,
            sample_rate,
        }
    }
}

impl Plugin for Delay {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks
        let buf_len = self.buffer.len();
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let read_pos = (self.write_pos + buf_len - self.delay_samples) % buf_len;
            let delayed = self.buffer[read_pos];
            self.buffer[self.write_pos] = in_s + delayed * self.feedback;
            self.write_pos = (self.write_pos + 1) % buf_len;
            *out_s = in_s * self.dry + delayed * self.wet;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.delay",
            name: "Delay",
            description: "Simple delay line with feedback",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        let delay_ms = self.delay_samples as f32 / self.sample_rate * 1000.0;
        vec![
            PluginParameter {
                id: "delay_ms".into(),
                name: "Delay (ms)".into(),
                value: delay_ms,
                min: 1.0,
                max: 2000.0,
                default: 500.0,
            },
            PluginParameter {
                id: "feedback".into(),
                name: "Feedback".into(),
                value: self.feedback,
                min: 0.0,
                max: 0.99,
                default: 0.3,
            },
            PluginParameter {
                id: "wet".into(),
                name: "Wet".into(),
                value: self.wet,
                min: 0.0,
                max: 1.0,
                default: 0.3,
            },
            PluginParameter {
                id: "dry".into(),
                name: "Dry".into(),
                value: self.dry,
                min: 0.0,
                max: 1.0,
                default: 0.7,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "delay_ms" => {
                let samples = (value * self.sample_rate / 1000.0) as usize;
                self.delay_samples = samples.clamp(1, self.buffer.len() - 1);
            }
            "feedback" => self.feedback = value.clamp(0.0, 0.99),
            "wet" => self.wet = value.clamp(0.0, 1.0),
            "dry" => self.dry = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "delay_ms" => Some(self.delay_samples as f32 / self.sample_rate * 1000.0),
            "feedback" => Some(self.feedback),
            "wet" => Some(self.wet),
            "dry" => Some(self.dry),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let delay_ms = self.delay_samples as f32 / self.sample_rate * 1000.0;
        self.sample_rate = sample_rate;
        let max_samples = (sample_rate * 2.0) as usize;
        self.buffer = vec![0.0; max_samples];
        self.write_pos = 0;
        self.delay_samples = ((delay_ms * sample_rate / 1000.0) as usize)
            .clamp(1, max_samples - 1);
    }

    fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|s| *s = 0.0);
        self.write_pos = 0;
    }
}
