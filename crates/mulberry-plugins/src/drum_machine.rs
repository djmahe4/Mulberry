//! 16-step drum machine sequencer.

/// Number of steps in the drum machine pattern.
pub const DRUM_STEPS: usize = 16;

/// Number of drum tracks.
pub const DRUM_TRACKS: usize = 8;

/// A single step in the drum pattern.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct DrumStep {
    pub active: bool,
    pub velocity: f32,
    pub accent: bool,
}

impl Default for DrumStep {
    fn default() -> Self {
        Self {
            active: false,
            velocity: 0.8,
            accent: false,
        }
    }
}

/// A full drum pattern (16 steps × 8 tracks).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DrumPattern {
    pub steps: [[DrumStep; DRUM_STEPS]; DRUM_TRACKS],
    pub name: String,
}

impl Default for DrumPattern {
    fn default() -> Self {
        Self {
            steps: [[DrumStep::default(); DRUM_STEPS]; DRUM_TRACKS],
            name: "Default".into(),
        }
    }
}

/// 16-step drum machine sequencer.
pub struct DrumMachine {
    pattern: DrumPattern,
    current_step: usize,
    playing: bool,
    bpm: f32,
    sample_rate: f32,
    samples_per_step: f32,
    sample_accumulator: f32,
}

impl DrumMachine {
    pub fn new(sample_rate: f32) -> Self {
        let bpm = 120.0_f32;
        let samples_per_step = Self::compute_samples_per_step(bpm, sample_rate);
        Self {
            pattern: DrumPattern::default(),
            current_step: 0,
            playing: false,
            bpm,
            sample_rate,
            samples_per_step,
            sample_accumulator: 0.0,
        }
    }

    fn compute_samples_per_step(bpm: f32, sample_rate: f32) -> f32 {
        // 4 steps per beat (16th notes)
        let steps_per_second = bpm / 60.0 * 4.0;
        sample_rate / steps_per_second
    }

    /// Advance the sequencer by `num_samples` audio samples.
    ///
    /// Returns a list of `(track, step, velocity)` tuples for every step that
    /// fired during this block.
    pub fn advance(&mut self, num_samples: u32) -> Vec<(usize, usize, f32)> {
        let mut triggered = Vec::new();
        if !self.playing {
            return triggered;
        }

        self.sample_accumulator += num_samples as f32;
        while self.sample_accumulator >= self.samples_per_step {
            self.sample_accumulator -= self.samples_per_step;
            let step = self.current_step;
            for track in 0..DRUM_TRACKS {
                let s = &self.pattern.steps[track][step];
                if s.active {
                    let vel = if s.accent { (s.velocity * 1.25).min(1.0) } else { s.velocity };
                    triggered.push((track, step, vel));
                }
            }
            self.current_step = (self.current_step + 1) % DRUM_STEPS;
        }
        triggered
    }

    pub fn set_step(&mut self, track: usize, step: usize, active: bool, velocity: f32) {
        if track < DRUM_TRACKS && step < DRUM_STEPS {
            self.pattern.steps[track][step].active = active;
            self.pattern.steps[track][step].velocity = velocity.clamp(0.0, 1.0);
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.max(1.0);
        self.samples_per_step = Self::compute_samples_per_step(self.bpm, self.sample_rate);
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.sample_accumulator = 0.0;
        self.playing = false;
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn pattern(&self) -> &DrumPattern {
        &self.pattern
    }

    pub fn pattern_mut(&mut self) -> &mut DrumPattern {
        &mut self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_advance_and_trigger() {
        let sr = 44100.0_f32;
        let mut dm = DrumMachine::new(sr);
        dm.set_bpm(120.0);
        dm.set_step(0, 0, true, 1.0);
        dm.play();

        // samples_per_step at 120 BPM, 4 steps/beat = 8 steps/sec
        // sr / 8 = 5512.5
        let sps = dm.samples_per_step as u32 + 1;
        let triggered = dm.advance(sps);
        assert!(!triggered.is_empty(), "step 0 should trigger");
        assert_eq!(triggered[0].0, 0); // track 0
        assert_eq!(triggered[0].1, 0); // step 0
        assert_eq!(triggered[0].2, 1.0);
    }

    #[test]
    fn no_triggers_when_stopped() {
        let mut dm = DrumMachine::new(44100.0);
        dm.set_step(0, 0, true, 1.0);
        // NOT calling play()
        let triggered = dm.advance(100_000);
        assert!(triggered.is_empty());
    }
}
