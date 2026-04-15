//! # Mulberry LiveCode
//!
//! Strudel/TidalCycles-style live coding pattern engine for the Mulberry DAW.
//!
//! Provides a mini-language for expressing musical patterns that can be
//! evaluated in real-time. Patterns describe sequences of notes, samples,
//! and transformations that cycle over time.
//!
//! # Pattern Syntax
//!
//! ```text
//! "c4 e4 g4 c5"          -- Simple note sequence (one per quarter)
//! "c4 [e4 g4]"           -- Subdivision: e4 and g4 share the second quarter
//! "c4 ~ e4 ~"            -- Rests: ~ is silence
//! "c4*3 e4"              -- Repeat: c4 three times, then e4
//! "c4 e4 g4" | fast 2    -- Transform: double the speed
//! "c4 e4" | slow 2       -- Transform: half the speed
//! "c4 e4" | rev           -- Transform: reverse the pattern
//! ```
//!
//! # Design Notes
//!
//! Uses a hand-written recursive descent parser (no parser combinator library)
//! to minimize dependencies and give full control over error messages for
//! live-coding UX.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod pattern;
pub mod scheduler;

pub use ast::PatternNode;
pub use error::LiveCodeError;
pub use lexer::Lexer;
pub use parser::Parser;
pub use pattern::{Pattern, PatternEvent};
pub use scheduler::PatternScheduler;
