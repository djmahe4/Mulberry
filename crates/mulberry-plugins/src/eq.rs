//! 8-Band Parametric EQ using biquad filters.

use mulberry_dsp::{BiquadFilter, FilterType};

use crate::plugin::{Plugin, PluginInfo, PluginParameter};

const NUM_BANDS: usize = 8;

/// Default center frequencies for the 8 EQ bands (Hz).
const DEFAULT_FREQUENCIES: [f32; NUM_BANDS] = [
    60.0, 120.0, 250.0, 500.0, 1000.0, 3000.0, 6000.0, 12000.0,
];

const DEFAULT_Q: f32 = 0.707;

/// 8-band parametric EQ. Each band is an independent biquad filter.
pub struct EightBandEq {
    bands: [BiquadFilter; NUM_BANDS],
    gains: [f32; NUM_BANDS],
    enabled: [bool; NUM_BANDS],
    frequencies: [f32; NUM_BANDS],
    qs: [f32; NUM_BANDS],
    sample_rate: f32,
}

impl EightBandEq {
    pub fn new(sample_rate: f32) -> Self {
        let bands = std::array::from_fn(|i| {
            BiquadFilter::new(FilterType::BandPass, DEFAULT_FREQUENCIES[i], DEFAULT_Q, sample_rate)
        });
        Self {
            bands,
            gains: [1.0; NUM_BANDS],
            enabled: [true; NUM_BANDS],
            frequencies: DEFAULT_FREQUENCIES,
            qs: [DEFAULT_Q; NUM_BANDS],
            sample_rate,
        }
    }

    /// Recomputes the filter for the given band.
    pub fn set_band_frequency(&mut self, band: usize, freq: f32) {
        if band >= NUM_BANDS {
            return;
        }
        self.frequencies[band] = freq;
        self.bands[band] = BiquadFilter::new(
            FilterType::BandPass,
            freq,
            self.qs[band],
            self.sample_rate,
        );
    }

    /// Set band gain (in dB, converted to linear).
    pub fn set_band_gain(&mut self, band: usize, gain_db: f32) {
        if band >= NUM_BANDS {
            return;
        }
        self.gains[band] = 10.0_f32.powf(gain_db / 20.0);
    }

    pub fn set_band_enabled(&mut self, band: usize, enabled: bool) {
        if band < NUM_BANDS {
            self.enabled[band] = enabled;
        }
    }
}

impl Plugin for EightBandEq {
    fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // REALTIME SAFE: no allocation, no locks
        for (in_s, out_s) in input.iter().zip(output.iter_mut()) {
            let mut sample = *in_s;
            for i in 0..NUM_BANDS {
                if self.enabled[i] {
                    sample = self.bands[i].process_sample(sample) * self.gains[i];
                }
            }
            *out_s = sample;
        }
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "mulberry.eq.8band",
            name: "8-Band EQ",
            description: "8-band parametric equaliser using biquad filters",
        }
    }

    fn parameters(&self) -> Vec<PluginParameter> {
        let mut params = Vec::with_capacity(NUM_BANDS * 4);
        for i in 0..NUM_BANDS {
            params.push(PluginParameter {
                id: format!("band{i}_freq"),
                name: format!("Band {} Frequency", i + 1),
                value: self.frequencies[i],
                min: 20.0,
                max: 20000.0,
                default: DEFAULT_FREQUENCIES[i],
            });
            let gain_db = 20.0 * self.gains[i].log10();
            params.push(PluginParameter {
                id: format!("band{i}_gain"),
                name: format!("Band {} Gain", i + 1),
                value: gain_db,
                min: -24.0,
                max: 24.0,
                default: 0.0,
            });
            params.push(PluginParameter {
                id: format!("band{i}_q"),
                name: format!("Band {} Q", i + 1),
                value: self.qs[i],
                min: 0.1,
                max: 10.0,
                default: DEFAULT_Q,
            });
            params.push(PluginParameter {
                id: format!("band{i}_enabled"),
                name: format!("Band {} Enabled", i + 1),
                value: if self.enabled[i] { 1.0 } else { 0.0 },
                min: 0.0,
                max: 1.0,
                default: 1.0,
            });
        }
        params
    }

    fn set_parameter(&mut self, id: &str, value: f32) {
        for i in 0..NUM_BANDS {
            if id == format!("band{i}_freq") {
                self.frequencies[i] = value;
                self.bands[i] = BiquadFilter::new(
                    FilterType::BandPass,
                    value,
                    self.qs[i],
                    self.sample_rate,
                );
                return;
            }
            if id == format!("band{i}_gain") {
                self.gains[i] = 10.0_f32.powf(value / 20.0);
                return;
            }
            if id == format!("band{i}_q") {
                self.qs[i] = value;
                self.bands[i] = BiquadFilter::new(
                    FilterType::BandPass,
                    self.frequencies[i],
                    value,
                    self.sample_rate,
                );
                return;
            }
            if id == format!("band{i}_enabled") {
                self.enabled[i] = value >= 0.5;
                return;
            }
        }
    }

    fn get_parameter(&self, id: &str) -> Option<f32> {
        for i in 0..NUM_BANDS {
            if id == format!("band{i}_freq") {
                return Some(self.frequencies[i]);
            }
            if id == format!("band{i}_gain") {
                return Some(20.0 * self.gains[i].log10());
            }
            if id == format!("band{i}_q") {
                return Some(self.qs[i]);
            }
            if id == format!("band{i}_enabled") {
                return Some(if self.enabled[i] { 1.0 } else { 0.0 });
            }
        }
        None
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for i in 0..NUM_BANDS {
            self.bands[i] = BiquadFilter::new(
                FilterType::BandPass,
                self.frequencies[i],
                self.qs[i],
                sample_rate,
            );
        }
    }

    fn reset(&mut self) {
        for band in self.bands.iter_mut() {
            band.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_in_silence_out() {
        let mut eq = EightBandEq::new(44100.0);
        let input = vec![0.0_f32; 256];
        let mut output = vec![0.0_f32; 256];
        eq.process_block(&input, &mut output);
        for s in &output {
            assert!(s.abs() < 1e-6, "expected silence, got {s}");
        }
    }
}
