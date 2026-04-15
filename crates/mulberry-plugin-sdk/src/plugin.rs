use serde::{Deserialize, Serialize};

/// Category of a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    Instrument,
    Effect,
    Analyzer,
    Utility,
}

/// Information about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub category: PluginCategory,
    pub description: String,
}

/// A plugin parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

/// Trait that all Mulberry plugins must implement.
///
/// # CONTEXT7 REVIEW:
/// Issue: Plugin trait must be object-safe for dynamic dispatch
/// Resolution: All methods use &self/&mut self, no generics, no Self in return position
/// Why: Plugins are loaded as trait objects (Box<dyn MulberryPlugin>)
pub trait MulberryPlugin: Send {
    /// Get plugin information.
    fn info(&self) -> PluginInfo;

    /// Initialize the plugin with the host API.
    fn initialize(
        &mut self,
        host: &dyn crate::host::HostApi,
    ) -> Result<(), crate::error::PluginError>;

    /// Process audio samples in-place.
    /// `inputs` contains input channel data, `outputs` is where processed data should be written.
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], sample_rate: u32);

    /// Handle an event from the host.
    fn handle_event(&mut self, event: &mulberry_core::Event);

    /// Called when the plugin is being unloaded.
    fn shutdown(&mut self);

    /// Get current parameter values.
    fn get_parameters(&self) -> Vec<PluginParameter>;

    /// Set a parameter value.
    fn set_parameter(&mut self, id: &str, value: f64);
}
