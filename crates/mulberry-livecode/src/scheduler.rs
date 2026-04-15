use mulberry_core::time::Tempo;

use crate::pattern::{Pattern, PatternEvent};

/// Schedules pattern events against real time.
///
/// Tracks a cycle position (`0.0..1.0`) and, on each call to
/// [`advance`](Self::advance), moves forward by the appropriate amount
/// (derived from tempo + sample count) and returns any events that fall
/// within the window.
pub struct PatternScheduler {
    /// Current pattern being played
    pattern: Option<Pattern>,
    /// Current position within the cycle (0.0 to 1.0)
    cycle_position: f64,
    /// Sample rate
    sample_rate: u32,
}

impl PatternScheduler {
    pub fn new(sample_rate: u32) -> Self {
        PatternScheduler {
            pattern: None,
            cycle_position: 0.0,
            sample_rate,
        }
    }

    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.pattern = Some(pattern);
    }

    pub fn clear(&mut self) {
        self.pattern = None;
    }

    /// Advance the cycle position and return events that trigger in this window.
    ///
    /// The cycle increment is derived from `tempo` and `num_samples`:
    ///
    /// ```text
    /// base_cps       = bpm / 240            (one cycle = 4 beats)
    /// effective_cps  = base_cps × pattern.cps
    /// Δcycle         = num_samples / sample_rate × effective_cps
    /// ```
    pub fn advance(&mut self, num_samples: u32, tempo: &Tempo) -> Vec<PatternEvent> {
        let pattern = match &self.pattern {
            Some(p) => p,
            None => return Vec::new(),
        };

        // One cycle = one bar of 4/4 time (4 beats)
        let base_cps = tempo.bpm / 240.0;
        let effective_cps = base_cps * pattern.cps;

        let cycle_increment = (num_samples as f64 / self.sample_rate as f64) * effective_cps;

        let old_pos = self.cycle_position;
        let new_pos = old_pos + cycle_increment;

        let events = if new_pos <= 1.0 {
            pattern
                .events_in_range(old_pos, new_pos)
                .into_iter()
                .cloned()
                .collect()
        } else {
            // Wrap around: collect from [old_pos, 1.0) then [0.0, remainder)
            let mut evts: Vec<PatternEvent> = pattern
                .events_in_range(old_pos, 1.0)
                .into_iter()
                .cloned()
                .collect();
            evts.extend(
                pattern
                    .events_in_range(0.0, new_pos - 1.0)
                    .into_iter()
                    .cloned(),
            );
            evts
        };

        self.cycle_position = new_pos % 1.0;
        events
    }

    pub fn cycle_position(&self) -> f64 {
        self.cycle_position
    }

    /// Reset the cycle position to the beginning.
    pub fn reset(&mut self) {
        self.cycle_position = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_pattern;

    fn make_pattern(src: &str) -> Pattern {
        let ast = parse_pattern(src).unwrap();
        Pattern::from_ast(&ast).unwrap()
    }

    #[test]
    fn test_scheduler_no_pattern() {
        let mut sched = PatternScheduler::new(44100);
        let events = sched.advance(1024, &Tempo::new(120.0));
        assert!(events.is_empty());
    }

    #[test]
    fn test_scheduler_advance_collects_events() {
        let mut sched = PatternScheduler::new(44100);
        sched.set_pattern(make_pattern("c4 e4 g4 a4"));

        // At 120 BPM, base_cps = 0.5.  pattern.cps = 1.0 → effective 0.5 cps.
        // One full cycle = 2 seconds = 88200 samples.
        // Advance the whole cycle at once to collect all four events.
        let events = sched.advance(88200, &Tempo::new(120.0));
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_scheduler_cycle_wraps() {
        let mut sched = PatternScheduler::new(44100);
        sched.set_pattern(make_pattern("c4 e4"));

        // Advance a full cycle
        let _ = sched.advance(88200, &Tempo::new(120.0));
        // Position should wrap back near 0.0
        assert!(sched.cycle_position() < 0.01);
    }

    #[test]
    fn test_scheduler_reset() {
        let mut sched = PatternScheduler::new(44100);
        sched.set_pattern(make_pattern("c4"));
        let _ = sched.advance(22050, &Tempo::new(120.0));
        assert!(sched.cycle_position() > 0.0);
        sched.reset();
        assert!((sched.cycle_position()).abs() < f64::EPSILON);
    }
}
