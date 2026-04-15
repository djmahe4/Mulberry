use serde::{Deserialize, Serialize};

/// Abstract syntax tree node for a pattern expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternNode {
    /// A single note event (note name string, e.g. "c4")
    Note(String),
    /// A rest (silence)
    Rest,
    /// A sequence of patterns played one after another within a cycle
    Sequence(Vec<PatternNode>),
    /// A subdivision — elements share their parent's time slot equally
    Subdivision(Vec<PatternNode>),
    /// Repeat a pattern N times
    Repeat(Box<PatternNode>, u32),
    /// Apply a transformation: (pattern, transform_name, optional argument)
    Transform {
        pattern: Box<PatternNode>,
        name: String,
        arg: Option<f64>,
    },
}
