//! # Mulberry Plugin SDK
//!
//! Plugin SDK for extending the Mulberry DAW with dynamic libraries.
//!
//! Plugins are shared libraries (.so/.dll/.dylib) that implement the
//! `MulberryPlugin` trait and export a C-compatible creation function.
//!
//! # Safety
//!
//! Plugin loading involves `unsafe` code for FFI. Each plugin runs in
//! its own isolated context with controlled access to the host API.
//!
//! # CONTEXT7 REVIEW:
//! Issue: Plugin safety and isolation
//! Resolution: Plugins communicate via a HostApi trait object with controlled capabilities.
//!   No direct access to audio thread state. Plugins submit events through the EventBus.
//! Why: Prevents plugins from corrupting audio state or causing UB in the host.
//!   All plugin API calls are bounds-checked and validated.

pub mod error;
pub mod host;
pub mod loader;
pub mod manifest;
pub mod plugin;

pub use error::PluginError;
pub use host::HostApi;
pub use loader::PluginLoader;
pub use manifest::PluginManifest;
pub use plugin::{MulberryPlugin, PluginCategory, PluginInfo};
