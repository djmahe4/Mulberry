/// Log level for plugin messages.
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// API that the host exposes to plugins.
///
/// This is the controlled interface through which plugins interact with
/// the DAW. Plugins receive a reference to this trait during initialization.
pub trait HostApi: Send + Sync {
    /// Get the current sample rate.
    fn sample_rate(&self) -> u32;

    /// Get the current buffer size.
    fn buffer_size(&self) -> usize;

    /// Get the current tempo in BPM.
    fn tempo(&self) -> f64;

    /// Get current transport position in beats.
    fn position_beats(&self) -> f64;

    /// Check if transport is playing.
    fn is_playing(&self) -> bool;

    /// Send an event to the host's event bus.
    fn send_event(&self, event: mulberry_core::Event);

    /// Log a message from the plugin.
    fn log(&self, level: LogLevel, message: &str);
}
