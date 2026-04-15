//! Simple mixer utility for summing and panning mono audio channels into
//! stereo [`AudioFrame`]s.

use mulberry_core::AudioFrame;
use std::f32::consts::FRAC_PI_2;

/// A single channel strip in the [`Mixer`].
#[derive(Clone, Debug)]
pub struct MixerChannel {
    /// Linear gain applied to this channel (`0.0..`).
    pub gain: f32,
    /// Stereo pan position (`-1.0` = hard left, `0.0` = centre, `1.0` = hard right).
    pub pan: f32,
    /// Whether this channel is muted.
    pub mute: bool,
    /// Whether this channel is soloed.
    pub solo: bool,
}

impl MixerChannel {
    /// Creates a new channel with unity gain, centre pan, un-muted, un-soloed.
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
        }
    }
}

impl Default for MixerChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple summing mixer that combines mono input buffers into stereo
/// [`AudioFrame`]s with per-channel gain and pan, plus a master gain.
///
// CONTEXT7 REVIEW: Equal-power pan law
// We use equal-power panning (`left = cos(θ)`, `right = sin(θ)` where
// `θ = (pan + 1) / 2 * π/2`).  This keeps perceived loudness constant as a
// source moves across the stereo field — the standard choice in most DAWs.
// Linear panning is simpler but causes a ~3 dB dip at centre.
pub struct Mixer {
    /// Per-channel strips.
    pub channels: Vec<MixerChannel>,
    /// Master output gain.
    pub master_gain: f32,
}

impl Mixer {
    /// Creates a mixer pre-populated with `num_channels` default channel strips.
    pub fn new(num_channels: usize) -> Self {
        Self {
            channels: (0..num_channels).map(|_| MixerChannel::new()).collect(),
            master_gain: 1.0,
        }
    }

    /// Appends a new default channel and returns its index.
    pub fn add_channel(&mut self) -> usize {
        self.channels.push(MixerChannel::new());
        self.channels.len() - 1
    }

    /// Sets the gain for channel `ch`.
    pub fn set_channel_gain(&mut self, ch: usize, gain: f32) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.gain = gain;
        }
    }

    /// Sets the pan for channel `ch` (clamped to `-1.0..=1.0`).
    pub fn set_channel_pan(&mut self, ch: usize, pan: f32) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.pan = pan.clamp(-1.0, 1.0);
        }
    }

    /// Sets the mute state for channel `ch`.
    pub fn set_channel_mute(&mut self, ch: usize, mute: bool) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.mute = mute;
        }
    }

    /// Sets the master output gain.
    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain;
    }

    /// Mixes all input buffers down to a vector of stereo [`AudioFrame`]s.
    ///
    /// Each entry in `inputs` corresponds to the channel at the same index.
    /// If an input is shorter than the longest buffer, missing samples are
    /// treated as silence.  Channels beyond the number of mixer strips are
    /// ignored.
    pub fn mix_down(&self, inputs: &[&[f32]]) -> Vec<AudioFrame> {
        let max_len = inputs.iter().map(|b| b.len()).max().unwrap_or(0);
        if max_len == 0 {
            return Vec::new();
        }

        let any_solo = self.channels.iter().any(|c| c.solo);

        let mut output = vec![AudioFrame::silence(); max_len];

        for (ch_idx, channel) in self.channels.iter().enumerate() {
            let buf = match inputs.get(ch_idx) {
                Some(b) => *b,
                None => continue,
            };

            // Skip muted channels. When solo is active, skip non-soloed ones.
            if channel.mute {
                continue;
            }
            if any_solo && !channel.solo {
                continue;
            }

            // Equal-power pan: θ = (pan + 1) / 2 * π/2
            let theta = (channel.pan + 1.0) / 2.0 * FRAC_PI_2;
            let left_gain = theta.cos() * channel.gain;
            let right_gain = theta.sin() * channel.gain;

            for (i, frame) in output.iter_mut().enumerate() {
                let sample = if i < buf.len() { buf[i] } else { 0.0 };
                frame.left += sample * left_gain;
                frame.right += sample * right_gain;
            }
        }

        // Apply master gain.
        if (self.master_gain - 1.0).abs() > f32::EPSILON {
            for frame in &mut output {
                frame.left *= self.master_gain;
                frame.right *= self.master_gain;
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_pan_equal_power() {
        let mixer = Mixer::new(1);
        let input: Vec<f32> = vec![1.0];
        let frames = mixer.mix_down(&[&input]);
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        // At centre pan (θ = π/4), cos and sin are equal ≈ 0.707.
        assert!((f.left - f.right).abs() < 1e-6);
        assert!((f.left - (FRAC_PI_2 / 2.0).cos()).abs() < 1e-4);
    }

    #[test]
    fn hard_left_pan() {
        let mut mixer = Mixer::new(1);
        mixer.set_channel_pan(0, -1.0);
        let input = vec![1.0];
        let frames = mixer.mix_down(&[&input]);
        let f = &frames[0];
        assert!((f.left - 1.0).abs() < 1e-6);
        assert!(f.right.abs() < 1e-6);
    }

    #[test]
    fn mute_silences_channel() {
        let mut mixer = Mixer::new(1);
        mixer.set_channel_mute(0, true);
        let input = vec![1.0; 4];
        let frames = mixer.mix_down(&[&input]);
        assert!(frames.iter().all(|f| f.left == 0.0 && f.right == 0.0));
    }

    #[test]
    fn master_gain_applied() {
        let mut mixer = Mixer::new(1);
        mixer.set_master_gain(0.5);
        let input = vec![1.0];
        let frames = mixer.mix_down(&[&input]);
        let f = &frames[0];
        let expected = (FRAC_PI_2 / 2.0).cos() * 0.5;
        assert!((f.left - expected).abs() < 1e-4);
    }

    #[test]
    fn solo_isolates_channel() {
        let mut mixer = Mixer::new(2);
        // Solo channel 1 only.
        mixer.channels[1].solo = true;
        let input0 = vec![1.0; 4];
        let input1 = vec![0.5; 4];
        let frames = mixer.mix_down(&[&input0, &input1]);
        // Channel 0 should be silent (not soloed).
        let expected_per_sample = 0.5 * (FRAC_PI_2 / 2.0).cos();
        for f in &frames {
            assert!((f.left - expected_per_sample).abs() < 1e-4);
        }
    }
}
