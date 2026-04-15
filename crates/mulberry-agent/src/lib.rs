//! # Mulberry Agent
//!
//! AI agent orchestration for the Mulberry DAW.
//!
//! Manages a pool of AI agents that can:
//! - Generate musical patterns via LLM APIs
//! - Suggest chord progressions
//! - Assist with mixing decisions
//! - Generate live-code snippets
//!
//! Uses `tokio::task::JoinSet` for structured concurrency:
//! all agent tasks are trackable and abortable.
//!
//! # Design Notes
//!
//! Agent tasks are long-running LLM calls (network I/O bound).
//! `tokio::task::JoinSet` provides trackable, abortable task management.
//! Unlike `FuturesUnordered`, `JoinSet` integrates with tokio's task system
//! and properly handles cancellation. This is the correct pattern for
//! background I/O-bound work.

pub mod agent;
pub mod error;
pub mod orchestrator;
pub mod prompt;

pub use agent::{Agent, AgentConfig, AgentRole};
pub use error::AgentError;
pub use orchestrator::AgentOrchestrator;
