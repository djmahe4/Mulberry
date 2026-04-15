//! Freeverb-style reverb using Schroeder comb + allpass filters.

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

/// Single-pole low-pass damped comb filter.
struct CombFilter {
    buffer: Vec<f32>,
    pos: usize,
    damp: f32,
    feedback: f32,
    filter_state: f32,
}

impl CombFilter {
    fn new(delay_len: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: vec![0.0; delay_len],
            pos: 0,
            damp,
            feedback,
            filter_state: 0.0,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.pos];
        self.filter_state = output * (1.0 - self.damp) + self.filter_state * self.damp;
        self.buffer[self.pos] = input + self.filter_state * self.feedback;
        self.pos = (self.pos + 1) % self.buffer.len();
        output
    }

    fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    fn set_damp(&mut self, damp: f32) {
        self.damp = damp;
    }

    fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|s| *s = 0.0);
        self.pos = 0;
        self.filter_state = 0.0;
    }
}

struct AllpassFilter {
    buffer: Vec<f32>,
    pos: usize,
}

impl AllpassFilter {
    fn new(delay_len: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_len],
            pos: 0,
        }
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.pos];
        self.buffer[self.pos] = input + buffered * 0.5;
        self.pos = (self.pos + 1) % self.buffer.len();
        buffered - input
    }

    fn clear(&mut self) {
        self.buffer.iter_mut().for_each(|s| *s = 0.0);
        self.pos = 0;
    }
}

/// Comb delay lengths at 44100 Hz.
const COMB_DELAYS_44100: [usize; 4] = [1116, 1188, 1277, 1356];
/// Allpass delay lengths at 44100 Hz.
const ALLPASS_DELAYS_44100: [usize; 2] = [556, 441];

const DEFAULT_ROOM_SIZE: f32 = 0.5;
const DEFAULT_DAMP: f32 = 0.5;
const DEFAULT_WET: f32 = 0.3;
const DEFAULT_DRY: f32 = 0.7;

fn feedback_from_room_size(room_size: f32) -> f32 {
    0.7 + room_size * 0.28
}

/// Freeverb-style reverb with 4 comb filters and 2 allpass filters.
pub struct Reverb {
    combs: [CombFilter; 4],
    allpasses: [AllpassFilter; 2],
    room_size: f32,
    damp: f32,
    wet: f32,
    dry: f32,
    sample_rate: f32,
}

fn scale_delay(base: usize, sample_rate: f32) -> usize {
    ((base as f32 * sample_rate / 44100.0) as usize).max(1)
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let feedback = feedback_from_room_size(DEFAULT_ROOM_SIZE);
        let combs = [
            CombFilter::new(scale_delay(COMB_DELAYS_44100[0], sample_rate), feedback, DEFAULT_DAMP),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[1], sample_rate), feedback, DEFAULT_DAMP),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[2], sample_rate), feedback, DEFAULT_DAMP),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[3], sample_rate), feedback, DEFAULT_DAMP),
        ];
        let allpasses = [
            AllpassFilter::new(scale_delay(ALLPASS_DELAYS_44100[0], sample_rate)),
            AllpassFilter::new(scale_delay(ALLPASS_DELAYS_44100[1], sample_rate)),
        ];
        Self {
            combs,
            allpasses,
            room_size: DEFAULT_ROOM_SIZE,
            damp: DEFAULT_DAMP,
            wet: DEFAULT_WET,
            dry: DEFAULT_DRY,
            sample_rate,
        }
    }
}

impl Plugin for Reverb {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks (buffers pre-allocated)
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let input_sample = *in_s;
            let reverb = self.combs[0].process(input_sample * 0.015)
                + self.combs[1].process(input_sample * 0.015)
                + self.combs[2].process(input_sample * 0.015)
                + self.combs[3].process(input_sample * 0.015);
            let allpassed = self.allpasses[0].process(reverb);
            let allpassed = self.allpasses[1].process(allpassed);
            *out_s = input_sample * self.dry + allpassed * self.wet;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.reverb",
            name: "Reverb",
            description: "Freeverb-style reverb with 4 comb + 2 allpass filters",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        vec![
            PluginParameter {
                id: "room_size".into(),
                name: "Room Size".into(),
                value: self.room_size,
                min: 0.0,
                max: 1.0,
                default: DEFAULT_ROOM_SIZE,
            },
            PluginParameter {
                id: "damp".into(),
                name: "Damping".into(),
                value: self.damp,
                min: 0.0,
                max: 1.0,
                default: DEFAULT_DAMP,
            },
            PluginParameter {
                id: "wet".into(),
                name: "Wet".into(),
                value: self.wet,
                min: 0.0,
                max: 1.0,
                default: DEFAULT_WET,
            },
        ]
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        match id {
            "room_size" => {
                self.room_size = value.clamp(0.0, 1.0);
                let feedback = feedback_from_room_size(self.room_size);
                for comb in self.combs.iter_mut() {
                    comb.set_feedback(feedback);
                }
            }
            "damp" => {
                self.damp = value.clamp(0.0, 1.0);
                for comb in self.combs.iter_mut() {
                    comb.set_damp(self.damp);
                }
            }
            "wet" => self.wet = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        match id {
            "room_size" => Some(self.room_size),
            "damp" => Some(self.damp),
            "wet" => Some(self.wet),
            _ => None,
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        let feedback = feedback_from_room_size(self.room_size);
        self.combs = [
            CombFilter::new(scale_delay(COMB_DELAYS_44100[0], sample_rate), feedback, self.damp),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[1], sample_rate), feedback, self.damp),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[2], sample_rate), feedback, self.damp),
            CombFilter::new(scale_delay(COMB_DELAYS_44100[3], sample_rate), feedback, self.damp),
        ];
        self.allpasses = [
            AllpassFilter::new(scale_delay(ALLPASS_DELAYS_44100[0], sample_rate)),
            AllpassFilter::new(scale_delay(ALLPASS_DELAYS_44100[1], sample_rate)),
        ];
    }

    fn reset(&mut self) {
        for comb in self.combs.iter_mut() {
            comb.clear();
        }
        for ap in self.allpasses.iter_mut() {
            ap.clear();
        }
    }
}
