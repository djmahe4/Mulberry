use serde::{Deserialize, Serialize};

use crate::ast::PatternNode;
use crate::error::LiveCodeError;

/// A single event in a resolved pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEvent {
    /// Start time within the cycle (0.0 to 1.0)
    pub start: f64,
    /// Duration within the cycle (fraction of cycle)
    pub duration: f64,
    /// MIDI note number (if it's a note)
    pub note: Option<u8>,
    /// Original note name string
    pub note_name: Option<String>,
}

/// A resolved pattern: a list of events over one cycle.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Events in this pattern, sorted by start time
    pub events: Vec<PatternEvent>,
    /// Playback speed multiplier (1.0 = follows tempo exactly).
    ///
    /// Modified by `fast` / `slow` transforms and [`set_cps`](Self::set_cps).
    /// The scheduler combines this with the current tempo to derive the
    /// effective cycles-per-second.
    pub cps: f64,
}

impl Pattern {
    /// Flatten an AST into a time-resolved list of events over one cycle.
    pub fn from_ast(node: &PatternNode) -> Result<Self, LiveCodeError> {
        let mut cps = 1.0;
        let events = resolve_node(node, 0.0, 1.0, &mut cps)?;
        Ok(Pattern { events, cps })
    }

    /// Override the cycles-per-second value.
    pub fn set_cps(&mut self, cps: f64) {
        self.cps = cps;
    }

    /// Return references to events whose start falls within `[start, end)`.
    ///
    /// Handles the wrap-around case where `start > end` (end of one cycle into
    /// the beginning of the next).
    pub fn events_in_range(&self, start: f64, end: f64) -> Vec<&PatternEvent> {
        if start <= end {
            self.events
                .iter()
                .filter(|e| e.start >= start && e.start < end)
                .collect()
        } else {
            // Wrap-around: [start, 1.0) ∪ [0.0, end)
            self.events
                .iter()
                .filter(|e| e.start >= start || e.start < end)
                .collect()
        }
    }
}

// ── AST → events ────────────────────────────────────────────────────

fn resolve_node(
    node: &PatternNode,
    start: f64,
    duration: f64,
    cps: &mut f64,
) -> Result<Vec<PatternEvent>, LiveCodeError> {
    match node {
        PatternNode::Note(name) => {
            let midi = note_name_to_midi(name)?;
            Ok(vec![PatternEvent {
                start,
                duration,
                note: Some(midi),
                note_name: Some(name.clone()),
            }])
        }

        PatternNode::Rest => Ok(vec![]),

        PatternNode::Sequence(elements) => {
            let n = elements.len() as f64;
            let sub_dur = duration / n;
            let mut events = Vec::new();
            for (i, elem) in elements.iter().enumerate() {
                let sub_start = start + sub_dur * i as f64;
                events.extend(resolve_node(elem, sub_start, sub_dur, cps)?);
            }
            Ok(events)
        }

        PatternNode::Subdivision(elements) => {
            let n = elements.len() as f64;
            let sub_dur = duration / n;
            let mut events = Vec::new();
            for (i, elem) in elements.iter().enumerate() {
                let sub_start = start + sub_dur * i as f64;
                events.extend(resolve_node(elem, sub_start, sub_dur, cps)?);
            }
            Ok(events)
        }

        PatternNode::Repeat(inner, count) => {
            let n = *count as f64;
            let sub_dur = duration / n;
            let mut events = Vec::new();
            for i in 0..*count {
                let sub_start = start + sub_dur * i as f64;
                events.extend(resolve_node(inner, sub_start, sub_dur, cps)?);
            }
            Ok(events)
        }

        PatternNode::Transform { pattern, name, arg } => {
            let mut events = resolve_node(pattern, start, duration, cps)?;

            match name.as_str() {
                "fast" => {
                    let n = arg.unwrap_or(2.0);
                    *cps *= n;
                }
                "slow" => {
                    let n = arg.unwrap_or(2.0);
                    *cps /= n;
                }
                "rev" => {
                    for ev in &mut events {
                        ev.start = start + duration - (ev.start - start) - ev.duration;
                    }
                    events.sort_by(|a, b| {
                        a.start
                            .partial_cmp(&b.start)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                other => {
                    return Err(LiveCodeError::EvalError(format!(
                        "Unknown transform: {}",
                        other
                    )));
                }
            }

            Ok(events)
        }
    }
}

/// Convert a note name like "c4" to a MIDI note number.
///
/// Supports optional sharp (`#`) or flat (`b`) accidentals.
/// ```text
/// c4  → 60
/// a4  → 69
/// c#4 → 61
/// eb4 → 63
/// ```
pub fn note_name_to_midi(name: &str) -> Result<u8, LiveCodeError> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return Err(LiveCodeError::UnknownNote(name.to_string()));
    }

    let mut pos = 0;

    // Note letter → semitone offset from C
    let semitone: i16 = match chars[pos].to_ascii_lowercase() {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => return Err(LiveCodeError::UnknownNote(name.to_string())),
    };
    pos += 1;

    // Optional accidental
    let accidental: i16 = if pos < chars.len() {
        match chars[pos] {
            '#' => {
                pos += 1;
                1
            }
            'b' => {
                pos += 1;
                -1
            }
            _ => 0,
        }
    } else {
        0
    };

    // Octave (required)
    if pos >= chars.len() {
        return Err(LiveCodeError::UnknownNote(name.to_string()));
    }
    let octave_str: String = chars[pos..].iter().collect();
    let octave: i16 = octave_str
        .parse()
        .map_err(|_| LiveCodeError::UnknownNote(name.to_string()))?;

    // MIDI formula: (octave + 1) * 12 + semitone + accidental
    let midi = (octave + 1) * 12 + semitone + accidental;

    if !(0..=127).contains(&midi) {
        return Err(LiveCodeError::UnknownNote(format!(
            "{} (MIDI {} out of 0–127)",
            name, midi
        )));
    }

    Ok(midi as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_pattern;

    // ── note_name_to_midi ────────────────────────────────────────────

    #[test]
    fn test_midi_c4() {
        assert_eq!(note_name_to_midi("c4").unwrap(), 60);
    }

    #[test]
    fn test_midi_a4() {
        assert_eq!(note_name_to_midi("a4").unwrap(), 69);
    }

    #[test]
    fn test_midi_sharps_and_flats() {
        assert_eq!(note_name_to_midi("c#4").unwrap(), 61);
        assert_eq!(note_name_to_midi("eb4").unwrap(), 63);
        assert_eq!(note_name_to_midi("f#5").unwrap(), 78);
        assert_eq!(note_name_to_midi("bb3").unwrap(), 58);
    }

    #[test]
    fn test_midi_edge_octaves() {
        // C-1 = 0 (lowest MIDI)
        assert_eq!(note_name_to_midi("c-1").unwrap(), 0);
        // G9 = 127 (highest)
        assert_eq!(note_name_to_midi("g9").unwrap(), 127);
    }

    // ── Pattern::from_ast ────────────────────────────────────────────

    #[test]
    fn test_pattern_simple_sequence() {
        let ast = parse_pattern("c4 e4 g4").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();

        assert_eq!(pat.events.len(), 3);

        let eps = 1e-10;
        assert!((pat.events[0].start).abs() < eps);
        assert!((pat.events[0].duration - 1.0 / 3.0).abs() < eps);
        assert_eq!(pat.events[0].note, Some(60));

        assert!((pat.events[1].start - 1.0 / 3.0).abs() < eps);
        assert_eq!(pat.events[1].note, Some(64));

        assert!((pat.events[2].start - 2.0 / 3.0).abs() < eps);
        assert_eq!(pat.events[2].note, Some(67));
    }

    #[test]
    fn test_pattern_subdivision() {
        let ast = parse_pattern("c4 [e4 g4]").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();

        assert_eq!(pat.events.len(), 3);

        let eps = 1e-10;
        // c4: [0.0, 0.5)
        assert!((pat.events[0].start).abs() < eps);
        assert!((pat.events[0].duration - 0.5).abs() < eps);

        // e4: [0.5, 0.75)
        assert!((pat.events[1].start - 0.5).abs() < eps);
        assert!((pat.events[1].duration - 0.25).abs() < eps);

        // g4: [0.75, 1.0)
        assert!((pat.events[2].start - 0.75).abs() < eps);
        assert!((pat.events[2].duration - 0.25).abs() < eps);
    }

    #[test]
    fn test_pattern_with_rest() {
        let ast = parse_pattern("c4 ~ e4").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();
        // Rest produces no event
        assert_eq!(pat.events.len(), 2);
        assert_eq!(pat.events[0].note, Some(60));
        assert_eq!(pat.events[1].note, Some(64));
    }

    #[test]
    fn test_pattern_repeat() {
        let ast = parse_pattern("c4*3").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();
        assert_eq!(pat.events.len(), 3);
        for ev in &pat.events {
            assert_eq!(ev.note, Some(60));
        }
    }

    #[test]
    fn test_pattern_fast_transform() {
        let ast = parse_pattern("c4 e4 | fast 2").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();
        assert!((pat.cps - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_pattern_rev_transform() {
        let ast = parse_pattern("c4 e4 g4 | rev").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();

        assert_eq!(pat.events.len(), 3);
        // After rev, g4 should come first, then e4, then c4
        assert_eq!(pat.events[0].note_name.as_deref(), Some("g4"));
        assert_eq!(pat.events[1].note_name.as_deref(), Some("e4"));
        assert_eq!(pat.events[2].note_name.as_deref(), Some("c4"));
    }

    #[test]
    fn test_events_in_range() {
        let ast = parse_pattern("c4 e4 g4 a4").unwrap();
        let pat = Pattern::from_ast(&ast).unwrap();

        // First half: should contain c4 (0.0) and e4 (0.25)
        let first_half = pat.events_in_range(0.0, 0.5);
        assert_eq!(first_half.len(), 2);

        // Second half: g4 (0.5) and a4 (0.75)
        let second_half = pat.events_in_range(0.5, 1.0);
        assert_eq!(second_half.len(), 2);
    }
}
